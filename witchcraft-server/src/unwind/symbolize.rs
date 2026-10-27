// Copyright 2026 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! Symbolization of addresses using the debug info and symbol tables of modules.
use crate::unwind::{LoadedModule, Unwind};
use addr2line::Loader;
use object::{Object, ObjectSymbol, SymbolKind};
use std::borrow::Cow;
use std::collections::HashMap;

/// A function at an address.
pub struct Frame {
    pub function: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Symbolizes addresses in the modules of an unwinder, loading each module's symbols on first use.
pub struct Symbolizer<'a> {
    unwind: &'a Unwind,
    // keyed by module start
    modules: HashMap<u64, ModuleSymbols>,
}

impl<'a> Symbolizer<'a> {
    pub fn new(unwind: &'a Unwind) -> Self {
        Symbolizer {
            unwind,
            modules: HashMap::new(),
        }
    }

    /// Returns the functions at an address, innermost first.
    ///
    /// There is more than one frame if functions were inlined into each other. Returns an empty vector if the address
    /// couldn't be symbolized.
    pub fn symbolize(&mut self, address: u64) -> Vec<Frame> {
        let Some(loaded) = self.unwind.module(address) else {
            return vec![];
        };
        let svma = address.wrapping_sub(loaded.module.bias);
        self.modules
            .entry(loaded.module.range.start)
            .or_insert_with(|| ModuleSymbols::new(loaded))
            .symbolize(svma)
    }
}

enum ModuleSymbols {
    Loader(Box<Loader>),
    // (address, size, name), sorted by address
    Symbols(Vec<(u64, u64, String)>),
    None,
}

impl ModuleSymbols {
    fn new(loaded: &LoadedModule) -> Self {
        // Only use files which match the loaded module.
        let Some(source) = &loaded.source else {
            return ModuleSymbols::None;
        };

        if let Some(path) = &loaded.module.file {
            return match Loader::new(path) {
                Ok(loader) => ModuleSymbols::Loader(Box::new(loader)),
                Err(_) => ModuleSymbols::None,
            };
        }

        // The vDSO isn't backed by a file, but does have a symbol table.
        let Ok(object) = object::File::parse(&**source) else {
            return ModuleSymbols::None;
        };
        let mut symbols = object
            .symbols()
            .chain(object.dynamic_symbols())
            .filter(|s| s.is_definition() && s.kind() == SymbolKind::Text)
            .filter_map(|s| Some((s.address(), s.size(), s.name().ok()?.to_string())))
            .collect::<Vec<_>>();
        symbols.sort_by_key(|(address, _, _)| *address);
        ModuleSymbols::Symbols(symbols)
    }

    fn symbolize(&self, svma: u64) -> Vec<Frame> {
        match self {
            ModuleSymbols::Loader(loader) => {
                let mut frames = vec![];
                if let Ok(mut it) = loader.find_frames(svma) {
                    while let Ok(Some(frame)) = it.next() {
                        let location = frame.location.as_ref();
                        frames.push(Frame {
                            function: frame
                                .function
                                .as_ref()
                                .and_then(|f| f.demangle().ok())
                                .map(Cow::into_owned),
                            file: location.and_then(|l| l.file).map(str::to_string),
                            line: location.and_then(|l| l.line),
                        });
                    }
                }

                // Without debug info, fall back to the symbol table.
                if frames.last().is_none_or(|f| f.function.is_none()) {
                    if let Some(name) = loader.find_symbol(svma) {
                        let function = demangle(name);
                        match frames.last_mut() {
                            Some(frame) => frame.function = Some(function),
                            None => frames.push(Frame {
                                function: Some(function),
                                file: None,
                                line: None,
                            }),
                        }
                    }
                }

                frames
            }
            ModuleSymbols::Symbols(symbols) => lookup_symbol(symbols, svma)
                .map(|name| Frame {
                    function: Some(demangle(name)),
                    file: None,
                    line: None,
                })
                .into_iter()
                .collect(),
            ModuleSymbols::None => vec![],
        }
    }
}

/// Looks up the symbol containing an address in a sorted list of (address, size, name).
fn lookup_symbol(symbols: &[(u64, u64, String)], svma: u64) -> Option<&str> {
    let idx = symbols.partition_point(|(address, _, _)| *address <= svma);
    let (address, size, name) = symbols.get(idx.checked_sub(1)?)?;
    // Some symbols have no size, in which case we assume they extend to the next symbol.
    if *size != 0 && svma >= address + size {
        return None;
    }
    Some(name)
}

fn demangle(name: &str) -> String {
    addr2line::demangle_auto(Cow::Borrowed(name), None).into_owned()
}
