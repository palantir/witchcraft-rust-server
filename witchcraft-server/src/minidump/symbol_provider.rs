use addr2line::fallible_iterator::FallibleIterator;
use addr2line::gimli::{Dwarf, EndianSlice, NativeEndian};
use addr2line::object::{
    File, Object as _, ObjectSection, ObjectSymbol, ObjectSymbolTable, SymbolKind, SymbolMap,
    SymbolMapName,
};
use addr2line::Context;
use async_trait::async_trait;
use cachemap2::CacheMap;
use conjure_error::Error;
use minidump::Module;
use minidump_processor::{
    FileError, FileKind, FillSymbolError, FrameSymbolizer, FrameWalker, SymbolFile, SymbolProvider,
};
use std::borrow::Cow;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use symbolic::cfi::CfiCache;
use symbolic::common::ByteView;
use symbolic::debuginfo::Object;
use witchcraft_log::warn;

// A bit jank, but there aren't any Sync arenas :(
pub struct Arena<T> {
    cache: CacheMap<usize, T>,
    next_key: AtomicUsize,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena {
            cache: CacheMap::new(),
            next_key: AtomicUsize::new(0),
        }
    }

    fn alloc(&self, value: T) -> &T {
        self.cache
            .cache(self.next_key.fetch_add(1, Ordering::Relaxed), || value)
    }
}

pub struct WitchcraftSymbolProvider<'a> {
    bufs: &'a Arena<ByteView<'a>>,
    objects: CacheMap<String, Option<ObjectState<'a>>>,
}

impl<'a> WitchcraftSymbolProvider<'a> {
    pub fn new(bufs: &'a Arena<ByteView<'a>>) -> Self {
        WitchcraftSymbolProvider {
            bufs,
            objects: CacheMap::new(),
        }
    }

    fn load_object(&self, module: &(dyn Module + Sync)) -> Option<&ObjectState<'a>> {
        self.objects
            .cache(module.code_file().to_string(), || {
                match ObjectState::new(self.bufs, module) {
                    Ok(state) => Some(state),
                    Err(e) => {
                        warn!("unable to load object for minidump processing", error: e);
                        None
                    }
                }
            })
            .as_ref()
    }
}

#[async_trait]
impl SymbolProvider for WitchcraftSymbolProvider<'_> {
    async fn fill_symbol(
        &self,
        module: &(dyn Module + Sync),
        frame: &mut (dyn FrameSymbolizer + Send),
    ) -> Result<(), FillSymbolError> {
        let Some(object) = self.load_object(module) else {
            return Err(FillSymbolError {})
        };

        object.fill_symbol(module, frame)
    }

    async fn walk_frame(
        &self,
        module: &(dyn Module + Sync),
        walker: &mut (dyn FrameWalker + Send),
    ) -> Option<()> {
        self.load_object(module)?.walk_frame(module, walker)
    }

    // This method is unused in minidump-processor
    async fn get_file_path(
        &self,
        _module: &(dyn Module + Sync),
        _file_kind: FileKind,
    ) -> Result<PathBuf, FileError> {
        Err(FileError::NotFound)
    }
}

struct ObjectState<'a> {
    cfi: SymbolFile,
    context: Option<Context<EndianSlice<'a, NativeEndian>>>,
    symbols: SymbolMap<SymbolMapName<'a>>,
}

impl<'a> ObjectState<'a> {
    fn new(bufs: &'a Arena<ByteView<'a>>, module: &(dyn Module + Sync)) -> Result<Self, Error> {
        let buf = ByteView::open(&*module.code_file()).map_err(Error::internal_safe)?;
        let buf = bufs.alloc(buf);

        let object = Object::parse(buf).map_err(Error::internal_safe)?;

        if module.code_identifier() != object.code_id() {
            return Err(Error::internal_safe("code ID mismatch for module")
                .with_safe_param("file", module.code_file())
                .with_safe_param("expected", module.code_identifier())
                .with_safe_param("actual", object.code_id()));
        }

        let cfi_cache = CfiCache::from_object(&object).map_err(Error::internal_safe)?;
        let cfi = SymbolFile::from_bytes(cfi_cache.as_slice()).map_err(Error::internal_safe)?;

        let file = File::parse(&**buf).map_err(Error::internal_safe)?;

        let dwarf = Dwarf::load(|id| {
            let buf = file
                .section_by_name(id.name())
                .and_then(|s| s.uncompressed_data().ok())
                .map(|b| &**bufs.alloc(ByteView::from_cow(b)))
                .unwrap_or(&[]);
            Ok::<_, Void>(EndianSlice::new(buf, NativeEndian))
        });

        let dwarf = match dwarf {
            Ok(dwarf) => dwarf,
            Err(void) => match void {},
        };

        let context = match Context::from_dwarf(dwarf) {
            Ok(context) => Some(context),
            Err(e) => {
                warn!(
                    "error loading debuginfo",
                    safe: { file: module.code_file() },
                    error: Error::internal_safe(e),
                );
                None
            }
        };

        let symbols = Self::build_symbol_map(&file);

        Ok(ObjectState {
            cfi,
            context,
            symbols,
        })
    }

    fn build_symbol_map(file: &File<'a, &'a [u8]>) -> SymbolMap<SymbolMapName<'a>> {
        let mut map = BTreeMap::<u64, PotentialName<'_>>::new();

        let symbols = file.symbol_table().or_else(|| file.dynamic_symbol_table());

        // Binaries commonly have multiple symbols for a given address, so we put a bit of effort
        // in here to pick the "best" one. In particular, we limit to function and data object
        // symbols, and prefer globally visible symbols over private ones.
        for symbol in symbols.into_iter().flat_map(|s| s.symbols()) {
            if !symbol.is_definition() {
                continue;
            }

            if symbol.kind() == SymbolKind::Unknown {
                continue;
            }

            let Ok(name) = symbol.name() else {
                continue;
            };

            let name = PotentialName {
                name: SymbolMapName::new(symbol.address(), name),
                global: symbol.is_global(),
            };

            match map.entry(symbol.address()) {
                Entry::Vacant(e) => {
                    e.insert(name);
                }
                Entry::Occupied(mut e) => {
                    if name.global && !e.get().global {
                        e.insert(name);
                    }
                }
            }
        }

        SymbolMap::new(map.into_values().map(|n| n.name).collect())
    }

    fn fill_symbol(
        &self,
        module: &(dyn Module + Sync),
        frame: &mut (dyn FrameSymbolizer + Send),
    ) -> Result<(), FillSymbolError> {
        let addr = frame.get_instruction() - module.base_address();

        if self.fill_symbol_dwarf(addr, frame).is_some() {
            return Ok(());
        }

        self.fill_symbol_fallback(addr, frame)
    }

    fn fill_symbol_dwarf(&self, addr: u64, frame: &mut (dyn FrameSymbolizer + Send)) -> Option<()> {
        let frames = self
            .context
            .as_ref()?
            .find_frames(addr)
            .skip_all_loads()
            .ok()?
            .collect::<Vec<_>>()
            .ok()?;

        // the dwarf frames are ordered "inside to outside", but we need to fill them in
        // "outside to inside"
        let mut it = frames.iter().rev();
        let first = it.next()?;

        frame.set_function(
            &first
                .function
                .as_ref()
                .and_then(|s| s.demangle().ok())
                .unwrap_or(Cow::Borrowed("???")),
            0,
            0,
        );
        if let Some(location) = &first.location {
            if let Some(file) = &location.file {
                frame.set_source_file(file, location.line.unwrap_or(0), 0);
            }
        }

        for inline in it {
            frame.add_inline_frame(
                &inline
                    .function
                    .as_ref()
                    .and_then(|s| s.demangle().ok())
                    .unwrap_or(Cow::Borrowed("???")),
                inline.location.as_ref().and_then(|l| l.file),
                inline.location.as_ref().and_then(|l| l.line),
            );
        }

        Some(())
    }

    fn fill_symbol_fallback(
        &self,
        addr: u64,
        frame: &mut (dyn FrameSymbolizer + Send),
    ) -> Result<(), FillSymbolError> {
        let symbol = self.symbols.get(addr);

        match symbol {
            Some(symbol) => {
                frame.set_function(
                    &rustc_demangle::demangle(symbol.name()).to_string(),
                    symbol.address(),
                    0,
                );
                Ok(())
            }
            None => Err(FillSymbolError {}),
        }
    }

    fn walk_frame(
        &self,
        module: &(dyn Module + Sync),
        walker: &mut (dyn FrameWalker + Send),
    ) -> Option<()> {
        self.cfi.walk_frame(module, walker)
    }
}

struct PotentialName<'a> {
    name: SymbolMapName<'a>,
    global: bool,
}

enum Void {}
