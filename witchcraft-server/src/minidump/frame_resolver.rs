// Copyright 2022 Palantir Technologies, Inc.
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

use addr2line::gimli::{Dwarf, EndianSlice, RunTimeEndian};
use addr2line::object::{
    File, Object, ObjectSection, ObjectSymbol, ObjectSymbolTable, SymbolKind, SymbolMap,
    SymbolMapName,
};
use addr2line::Context;
use conjure_error::Error;
use minidump::{MinidumpModule, Module};
use minidump_processor::StackFrame;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};
use symbolic::common::ByteView;
use typed_arena::Arena;

pub struct Frame<'a> {
    pub name: Option<&'a str>,
    pub file: Option<&'a str>,
    pub line: Option<u32>,
}

pub struct FrameResolver<'a> {
    arena: &'a Arena<ByteView<'a>>,
    state: HashMap<String, ObjectState<'a>>,
}

impl<'a> FrameResolver<'a> {
    pub fn new(arena: &'a Arena<ByteView<'a>>) -> Self {
        FrameResolver {
            arena,
            state: HashMap::new(),
        }
    }

    pub fn resolve<F>(&mut self, stack_frame: &StackFrame, mut cb: F)
    where
        F: FnMut(&Frame),
    {
        let module = match &stack_frame.module {
            Some(module) => module,
            None => {
                cb(&Frame {
                    name: None,
                    file: None,
                    line: None,
                });
                return;
            }
        };

        let addr = stack_frame.instruction - module.base_address();

        let state = self
            .state
            .entry(module.code_file().to_string())
            .or_insert_with(|| ObjectState::new(self.arena, module).unwrap());

        let mut hit = false;
        let mut it = state.context.find_frames(addr).unwrap();
        while let Ok(Some(frame)) = it.next() {
            hit = true;
            let name = frame.function.as_ref().and_then(|f| f.demangle().ok());
            cb(&Frame {
                name: name.as_deref(),
                file: frame.location.as_ref().and_then(|l| l.file),
                line: frame.location.as_ref().and_then(|l| l.line),
            });
        }

        if hit {
            return;
        }

        let symbol = state
            .symbol_map
            .get(addr)
            .map(|s| rustc_demangle::demangle(s.name()).to_string());

        cb(&Frame {
            name: symbol.as_deref(),
            file: None,
            line: None,
        });
    }
}

struct ObjectState<'a> {
    context: Context<EndianSlice<'a, RunTimeEndian>>,
    symbol_map: SymbolMap<SymbolMapName<'a>>,
}

impl<'a> ObjectState<'a> {
    fn new(arena: &'a Arena<ByteView<'a>>, module: &MinidumpModule) -> Result<Self, Error> {
        let view = ByteView::open(&*module.code_file()).map_err(Error::internal_safe)?;
        let view = arena.alloc(view);
        let file = File::parse(&**view).map_err(Error::internal_safe)?;

        let symbol_map = Self::symbol_map(&file);

        let endian = if file.is_little_endian() {
            RunTimeEndian::Little
        } else {
            RunTimeEndian::Big
        };

        let dwarf = Dwarf::load(|id| {
            let buf = file
                .section_by_name(id.name())
                .and_then(|s| s.uncompressed_data().ok())
                .map(|b| &**arena.alloc(ByteView::from_cow(b)))
                .unwrap_or(&[]);
            Ok::<_, Void>(EndianSlice::new(buf, endian))
        });

        let dwarf = match dwarf {
            Ok(dwarf) => dwarf,
            Err(void) => match void {},
        };

        let context = Context::from_dwarf(dwarf).map_err(Error::internal_safe)?;

        Ok(ObjectState {
            context,
            symbol_map,
        })
    }

    fn symbol_map(file: &File<'a, &'a [u8]>) -> SymbolMap<SymbolMapName<'a>> {
        let mut map = BTreeMap::<u64, PotentialName<'_>>::new();

        let symbols = file
            .symbol_table()
            // https://github.com/gimli-rs/object/pull/443
            .filter(|t| t.symbols().next().is_some())
            .or_else(|| file.dynamic_symbol_table());

        // Binaries commonly have multiple symbols for a given address, so we put a bit of effort in here to pick the
        // "best" one. In particular, we limit to function and data object symbols, and prefer globally visible symbols
        // over private symbols.
        for symbol in symbols.into_iter().flat_map(|s| s.symbols()) {
            if !symbol.is_definition() {
                continue;
            }

            if symbol.kind() == SymbolKind::Unknown {
                continue;
            }

            let name = match symbol.name() {
                Ok(name) => name,
                Err(_) => continue,
            };

            let name = PotentialName {
                name: SymbolMapName::new(symbol.address(), name),
                global: symbol.is_global(),
            };

            match map.entry(symbol.address()) {
                Entry::Occupied(mut e) => {
                    if name.global && !e.get().global {
                        e.insert(name);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(name);
                }
            }
        }

        SymbolMap::new(map.into_values().map(|n| n.name).collect())
    }
}

struct PotentialName<'a> {
    name: SymbolMapName<'a>,
    global: bool,
}

enum Void {}
