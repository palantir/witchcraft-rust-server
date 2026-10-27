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
//! DWARF unwinding and symbolization of stacks.
//!
//! Stacks can come from the current process or, given a description of its loaded modules, another one.
use crate::unwind::modules::Module;
use framehop::{CacheNative, MayAllocateDuringUnwind, ModuleSectionInfo, Unwinder, UnwinderNative};
use memmap2::Mmap;
use object::{Object, ObjectSection};
use std::collections::BTreeMap;
use std::fs::File;
use std::ops::{Deref, Range};
use std::ptr;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub mod modules;
pub mod symbolize;

/// The maximum number of frames recorded per stack.
const MAX_FRAMES: usize = 1024;

/// The minimum time between rescans of the loaded modules triggered by samples in unknown code.
const RESCAN_INTERVAL: Duration = Duration::from_secs(1);

/// The registers needed to start an unwind.
#[derive(Copy, Clone, Default)]
pub struct Regs {
    pub pc: u64,
    pub sp: u64,
    pub fp: u64,
    #[cfg(target_arch = "aarch64")]
    pub lr: u64,
}

#[derive(Clone)]
pub enum Source {
    File(Arc<Mmap>),
    Static(&'static [u8]),
}

impl Deref for Source {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Source::File(mmap) => mmap,
            Source::Static(data) => data,
        }
    }
}

/// A section of a module's file.
pub struct ModuleData {
    source: Source,
    range: Range<usize>,
}

impl Deref for ModuleData {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.source[self.range.clone()]
    }
}

pub struct LoadedModule {
    pub module: Module,
    /// The contents of the module's file, if it could be read and matches the loaded module.
    pub source: Option<Source>,
}

pub struct Unwind {
    unwinder: UnwinderNative<ModuleData, MayAllocateDuringUnwind>,
    cache: CacheNative<MayAllocateDuringUnwind>,
    // keyed by the start of the module's range
    modules: BTreeMap<u64, LoadedModule>,
    // set when unwinding the current process
    last_scan: Option<Instant>,
}

impl Unwind {
    /// Creates an unwinder with no modules.
    pub fn new() -> Self {
        Unwind {
            unwinder: UnwinderNative::new(),
            cache: CacheNative::new(),
            modules: BTreeMap::new(),
            last_scan: None,
        }
    }

    /// Creates an unwinder for the current process, which tracks the modules it loads.
    pub fn current_process() -> Self {
        let mut unwind = Unwind::new();
        unwind.scan();
        unwind
    }

    /// Updates the set of modules to match those currently loaded in the current process.
    fn scan(&mut self) {
        self.last_scan = Some(Instant::now());

        let mut old = std::mem::take(&mut self.modules);
        for module in modules::scan() {
            let start = module.range.start;
            let loaded = match old.remove(&start) {
                Some(loaded)
                    if loaded.module.range == module.range
                        && loaded.module.build_id == module.build_id =>
                {
                    loaded
                }
                Some(loaded) => {
                    self.unwinder.remove_module(loaded.module.text.start);
                    self.load(module)
                }
                None => self.load(module),
            };
            self.modules.insert(start, loaded);
        }

        for loaded in old.values() {
            self.unwinder.remove_module(loaded.module.text.start);
        }
    }

    /// Adds a module.
    pub fn add_module(&mut self, module: Module) {
        let start = module.range.start;
        let loaded = self.load(module);
        self.modules.insert(start, loaded);
    }

    fn load(&mut self, module: Module) -> LoadedModule {
        let source = load_source(&module);
        if let Some(source) = &source {
            if let Ok(object) = object::File::parse(&**source) {
                self.unwinder.add_module(framehop::Module::new(
                    module.name.clone(),
                    module.text.clone(),
                    module.bias,
                    ObjectSectionInfo {
                        object: &object,
                        source,
                    },
                ));
            }
        }

        LoadedModule { module, source }
    }

    /// Returns all modules.
    pub fn modules(&self) -> impl Iterator<Item = &LoadedModule> {
        self.modules.values()
    }

    /// Returns the module containing an address.
    pub fn module(&self, addr: u64) -> Option<&LoadedModule> {
        self.modules
            .range(..=addr)
            .next_back()
            .map(|(_, m)| m)
            .filter(|m| m.module.range.contains(&addr))
    }

    /// Unwinds a thread from its registers and a copy of its stack memory starting at `stack_base`, writing its frames
    /// to `frames`.
    ///
    /// The first frame is the instruction pointer and the remainder are return addresses minus one, so every frame
    /// can be symbolized directly.
    pub fn unwind(&mut self, regs: &Regs, stack_base: u64, stack: &[u8], frames: &mut Vec<u64>) {
        frames.clear();

        // When unwinding the current process, the stack may be in a module loaded since the last scan.
        if let Some(last_scan) = self.last_scan {
            if self.module(regs.pc).is_none() && last_scan.elapsed() >= RESCAN_INTERVAL {
                self.scan();
            }
        }

        let mut read_stack = |addr: u64| {
            let offset =
                usize::try_from(addr.checked_sub(stack_base).ok_or(())?).map_err(|_| ())?;
            let bytes = stack
                .get(offset..offset.checked_add(8).ok_or(())?)
                .ok_or(())?;
            Ok(u64::from_ne_bytes(bytes.try_into().unwrap()))
        };

        let mut it = self.unwinder.iter_frames(
            regs.pc,
            self.unwind_regs(regs),
            &mut self.cache,
            &mut read_stack,
        );
        // The iterator ends with an error both on truncated stacks and in some cases of normal termination, so we just
        // keep whatever frames we got.
        while let Ok(Some(frame)) = it.next() {
            frames.push(frame.address_for_lookup());
            if frames.len() == MAX_FRAMES {
                break;
            }
        }

        if frames.is_empty() {
            frames.push(regs.pc);
        }
    }

    /// Discards cached unwind state, in case it was left inconsistent by a panic during an unwind.
    pub fn reset_cache(&mut self) {
        self.cache = CacheNative::new();
    }

    #[cfg(target_arch = "x86_64")]
    fn unwind_regs(&self, regs: &Regs) -> framehop::UnwindRegsNative {
        framehop::x86_64::UnwindRegsX86_64::new(regs.pc, regs.sp, regs.fp)
    }

    #[cfg(target_arch = "aarch64")]
    fn unwind_regs(&self, regs: &Regs) -> framehop::UnwindRegsNative {
        use framehop::aarch64::{PtrAuthMask, UnwindRegsAarch64};

        let mask = PtrAuthMask::from_max_known_address(self.unwinder.max_known_code_address());
        UnwindRegsAarch64::new_with_ptr_auth_mask(mask, regs.lr, regs.sp, regs.fp)
    }
}

/// Loads the contents of a module's file, if it matches the loaded image.
fn load_source(module: &Module) -> Option<Source> {
    if let Some(vdso) = module.vdso {
        return Some(Source::Static(vdso));
    }

    let file = File::open(module.file.as_ref()?).ok()?;
    // SAFETY: Mapped executables and libraries are not modified while in use.
    let mmap = unsafe { Mmap::map(&file) }.ok()?;
    let source = Source::File(Arc::new(mmap));

    // The file may have been replaced since it was loaded, in which case its unwind info would be wrong.
    if let Some(expected) = &module.build_id {
        let object = object::File::parse(&*source).ok()?;
        if object.build_id().ok().flatten() != Some(&**expected) {
            return None;
        }
    }

    Some(source)
}

/// Exposes an object's sections to framehop.
///
/// This is equivalent to minidump-unwind's `ObjectSectionInfo` except that section data is referenced by offset into
/// the shared file mapping rather than by a reference with an extended lifetime.
struct ObjectSectionInfo<'a> {
    object: &'a object::File<'a>,
    source: &'a Source,
}

impl ObjectSectionInfo<'_> {
    fn module_data(&self, data: &[u8]) -> Option<ModuleData> {
        let base = self.source.as_ptr() as usize;
        let start = (data.as_ptr() as usize).checked_sub(base)?;
        let end = start.checked_add(data.len())?;
        if end > self.source.len() {
            return None;
        }

        debug_assert!(ptr::eq(&self.source[start..end], data));
        Some(ModuleData {
            source: self.source.clone(),
            range: start..end,
        })
    }
}

impl ModuleSectionInfo<ModuleData> for ObjectSectionInfo<'_> {
    fn base_svma(&self) -> u64 {
        // ELF section addresses are relative to a base of zero.
        0
    }

    fn section_svma_range(&mut self, name: &[u8]) -> Option<Range<u64>> {
        let section = self.object.section_by_name_bytes(name)?;
        Some(section.address()..section.address() + section.size())
    }

    fn section_data(&mut self, name: &[u8]) -> Option<ModuleData> {
        let section = self.object.section_by_name_bytes(name)?;
        self.module_data(section.data().ok()?)
    }
}
