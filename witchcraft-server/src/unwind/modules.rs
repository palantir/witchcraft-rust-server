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
//! Discovery of loaded modules.
use memmap2::Mmap;
use object::elf;
use object::{Object as _, ObjectSegment, SegmentFlags};
use std::ffi::{CStr, OsStr};
use std::fs::File;
use std::ops::Range;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::{fs, slice};

const PAGE_SIZE: u64 = 4096;
const NT_GNU_BUILD_ID: u32 = 3;

/// A loaded module.
pub struct Module {
    /// The name of the module's file, for display.
    pub name: String,
    /// The path to open to read the module's file, or `None` for the vDSO, which isn't backed by a file.
    pub file: Option<PathBuf>,
    /// The page-aligned range of addresses covered by the module's loadable segments.
    pub range: Range<u64>,
    /// The file offset corresponding to the start of `range`.
    pub file_offset: u64,
    /// The range of addresses covered by the module's executable segments.
    pub text: Range<u64>,
    /// The difference between runtime addresses and the addresses in the module's file.
    pub bias: u64,
    /// The build ID of the loaded module.
    pub build_id: Option<Vec<u8>>,
    /// The loaded image of the vDSO.
    pub vdso: Option<&'static [u8]>,
    /// Whether this is the main executable of the process.
    pub main: bool,
}

struct ScannedObject {
    name: Vec<u8>,
    bias: u64,
    phdrs: Vec<libc::Elf64_Phdr>,
    build_id: Option<Vec<u8>>,
}

/// Returns all modules currently loaded into the process.
pub fn scan() -> Vec<Module> {
    let mut objects = Vec::<ScannedObject>::new();
    // SAFETY: the callback matches the expected signature and the data pointer is to a live Vec<ScannedObject>.
    unsafe {
        libc::dl_iterate_phdr(
            Some(callback),
            (&mut objects as *mut Vec<ScannedObject>).cast(),
        )
    };

    // SAFETY: getauxval is always safe to call.
    let vdso = unsafe { libc::getauxval(libc::AT_SYSINFO_EHDR) };

    objects
        .iter()
        .enumerate()
        .filter_map(|(i, object)| module(i, object, vdso))
        .collect()
}

unsafe extern "C" fn callback(
    info: *mut libc::dl_phdr_info,
    _: libc::size_t,
    data: *mut libc::c_void,
) -> libc::c_int {
    let objects = &mut *data.cast::<Vec<ScannedObject>>();
    let info = &*info;

    let name = if info.dlpi_name.is_null() {
        vec![]
    } else {
        CStr::from_ptr(info.dlpi_name).to_bytes().to_vec()
    };
    let phdrs = if info.dlpi_phdr.is_null() {
        vec![]
    } else {
        slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum.into()).to_vec()
    };
    // The build ID must be read here, while the loader's lock prevents the module from being unloaded.
    let build_id = build_id(info.dlpi_addr, &phdrs);
    objects.push(ScannedObject {
        name,
        bias: info.dlpi_addr,
        phdrs,
        build_id,
    });

    0
}

/// A loadable segment of a module.
struct Segment {
    vaddr: u64,
    memsz: u64,
    offset: u64,
    executable: bool,
}

/// The runtime layout of a module.
struct Layout {
    range: Range<u64>,
    file_offset: u64,
    text: Range<u64>,
}

fn layout(bias: u64, segments: &[Segment]) -> Option<Layout> {
    let first = segments.iter().min_by_key(|s| s.vaddr)?;
    let start = bias.wrapping_add(first.vaddr);
    let aligned_start = start & !(PAGE_SIZE - 1);
    let end = segments
        .iter()
        .map(|s| bias.wrapping_add(s.vaddr).wrapping_add(s.memsz))
        .max()?;
    let aligned_end = end.checked_next_multiple_of(PAGE_SIZE)?;

    let text = || segments.iter().filter(|s| s.executable);
    let text_start = text().map(|s| bias.wrapping_add(s.vaddr)).min()?;
    let text_end = text()
        .map(|s| bias.wrapping_add(s.vaddr).wrapping_add(s.memsz))
        .max()?;

    Some(Layout {
        range: aligned_start..aligned_end,
        file_offset: first.offset - (start - aligned_start),
        text: text_start..text_end,
    })
}

fn module(index: usize, object: &ScannedObject, vdso: libc::c_ulong) -> Option<Module> {
    let segments = object
        .phdrs
        .iter()
        .filter(|p| p.p_type == libc::PT_LOAD)
        .map(|p| Segment {
            vaddr: p.p_vaddr,
            memsz: p.p_memsz,
            offset: p.p_offset,
            executable: p.p_flags & libc::PF_X != 0,
        })
        .collect::<Vec<_>>();
    let layout = layout(object.bias, &segments)?;

    let main = index == 0 && object.name.is_empty();
    let (name, file, vdso) = if vdso != 0 && layout.range.contains(&vdso) {
        // SAFETY: the vDSO is mapped readable for the lifetime of the process.
        let data = unsafe {
            slice::from_raw_parts(
                layout.range.start as *const u8,
                (layout.range.end - layout.range.start) as usize,
            )
        };
        ("[vdso]".to_string(), None, Some(data))
    } else if main {
        // The main executable. /proc/self/exe refers to the file we were executed from even if it has since been
        // replaced or deleted.
        let name = fs::read_link("/proc/self/exe")
            .map(|p| {
                p.to_string_lossy()
                    .trim_end_matches(" (deleted)")
                    .to_string()
            })
            .unwrap_or_default();
        (name, Some(PathBuf::from("/proc/self/exe")), None)
    } else if object.name.is_empty() {
        return None;
    } else {
        let path = PathBuf::from(OsStr::from_bytes(&object.name));
        (path.to_string_lossy().into_owned(), Some(path), None)
    };

    Some(Module {
        name,
        file,
        range: layout.range,
        file_offset: layout.file_offset,
        text: layout.text,
        bias: object.bias,
        build_id: object.build_id.clone(),
        vdso,
        main,
    })
}

/// Creates a module for a file which was loaded at `base`, the address of its first page.
///
/// This is used for modules of other processes, so the layout is determined from the file's program headers.
pub fn from_file(path: &Path, base: u64, build_id: Option<Vec<u8>>) -> Option<Module> {
    let file = File::open(path).ok()?;
    // SAFETY: Mapped executables and libraries are not modified while in use.
    let mmap = unsafe { Mmap::map(&file) }.ok()?;
    let object = object::File::parse(&*mmap).ok()?;

    let segments = object
        .segments()
        .map(|s| Segment {
            vaddr: s.address(),
            memsz: s.size(),
            offset: s.file_range().0,
            executable: matches!(
                s.flags(),
                SegmentFlags::Elf { p_flags, .. } if p_flags.0 & elf::PF_X.0 != 0
            ),
        })
        .collect::<Vec<_>>();
    let first = segments.iter().map(|s| s.vaddr).min()?;
    let bias = base.wrapping_sub(first & !(PAGE_SIZE - 1));
    let layout = layout(bias, &segments)?;

    Some(Module {
        name: path.to_string_lossy().into_owned(),
        file: Some(path.to_path_buf()),
        range: layout.range,
        file_offset: layout.file_offset,
        text: layout.text,
        bias,
        build_id,
        vdso: None,
        main: false,
    })
}

/// Reads the build ID of a loaded module from its notes.
///
/// # Safety
///
/// The module must be loaded.
unsafe fn build_id(bias: u64, phdrs: &[libc::Elf64_Phdr]) -> Option<Vec<u8>> {
    phdrs
        .iter()
        .filter(|p| p.p_type == libc::PT_NOTE)
        .find_map(|p| {
            // SAFETY: PT_NOTE segments are contained in a loaded segment.
            let notes = unsafe {
                slice::from_raw_parts(
                    bias.wrapping_add(p.p_vaddr) as *const u8,
                    p.p_memsz as usize,
                )
            };
            parse_build_id(notes)
        })
}

fn parse_build_id(mut notes: &[u8]) -> Option<Vec<u8>> {
    fn u32_at(b: &[u8], i: usize) -> Option<u32> {
        Some(u32::from_ne_bytes(b.get(i..i + 4)?.try_into().unwrap()))
    }

    while notes.len() >= 12 {
        let namesz = u32_at(notes, 0)? as usize;
        let descsz = u32_at(notes, 4)? as usize;
        let type_ = u32_at(notes, 8)?;

        let name_start = 12;
        let desc_start = name_start + namesz.next_multiple_of(4);
        let next = desc_start + descsz.next_multiple_of(4);

        let name = notes.get(name_start..name_start + namesz)?;
        let desc = notes.get(desc_start..desc_start + descsz)?;
        if type_ == NT_GNU_BUILD_ID && name == b"GNU\0" {
            return Some(desc.to_vec());
        }

        notes = notes.get(next..)?;
    }

    None
}
