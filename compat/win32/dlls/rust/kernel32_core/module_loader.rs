// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 module/PE loader API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/module_loader.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 PE module loading API stubs: LoadLibraryA/W/Ex,
//   GetModuleHandleA/W/Ex, GetModuleFileNameA/W, FreeLibrary,
//   FreeLibraryAndExitThread, GetProcAddress, DisableThreadLibraryCalls,
//   GetModuleInformation, GetModuleBaseNameA/W.  In the ÖZKAN-OS
//   environment these delegate to an internal module registry (stub).
//
// Bağımlı Dosyalar:
//   1-) alloc (crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Clean-room Rust port from Win32 API spec
// *******************************************************************

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::ptr;
use core::sync::atomic::{AtomicU64, Ordering};

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Wchar = u16;
type Handle = *mut core::ffi::c_void;
type Hinstance = *mut core::ffi::c_void;
type Hmodule = *mut core::ffi::c_void;
type FarProc = Option<unsafe extern "C" fn() -> i32>;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const MAX_PATH: usize = 260;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_MOD_NOT_FOUND: Dword = 126;
const ERROR_PROC_NOT_FOUND: Dword = 127;
const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;

fn set_last_error(_code: Dword) {}

fn lstrlen_w(s: *const Wchar) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len: usize = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

fn lstrcpy_w(dst: *mut Wchar, src: *const Wchar) {
    if dst.is_null() || src.is_null() {
        return;
    }
    unsafe {
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(i) = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
    }
}

fn wcsicmp(a: *const Wchar, b: *const Wchar) -> i32 {
    unsafe {
        let mut i = 0;
        loop {
            let ca = *a.add(i);
            let cb = *b.add(i);
            if ca == 0 && cb == 0 {
                return 0;
            }
            let ua = if ca >= 0x0041 && ca <= 0x005A { ca + 0x0020 } else { ca };
            let ub = if cb >= 0x0041 && cb <= 0x005A { cb + 0x0020 } else { cb };
            if ua != ub {
                return if ua < ub { -1 } else { 1 };
            }
            i += 1;
        }
    }
}

// ========================================================================
// Module registry — internal
// ========================================================================

#[derive(Clone)]
struct ModuleEntry {
    name: Vec<Wchar>,
    file_name: Vec<Wchar>,
    base: Hmodule,
    exports: BTreeMap<Vec<u8>, FarProc>,
    ref_count: u32,
}

use core::cell::UnsafeCell;

struct ModuleState {
    modules: UnsafeCell<Vec<ModuleEntry>>,
    next_base: AtomicU64,
}

unsafe impl Sync for ModuleState {}

static MODULE_STATE: ModuleState = ModuleState {
    modules: UnsafeCell::new(Vec::new()),
    next_base: AtomicU64::new(0x10000000),
};

fn with_module_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<ModuleEntry>) -> R,
{
    let cell = unsafe { &mut *MODULE_STATE.modules.get() };
    f(cell)
}

fn find_module_by_name(name: &[Wchar]) -> Option<Hmodule> {
    with_module_table(|modules| {
        for entry in modules.iter() {
            if wcsicmp(entry.name.as_ptr(), name.as_ptr()) == 0 {
                return Some(entry.base);
            }
        }
        None
    })
}

fn find_module_by_handle(handle: Hmodule) -> Option<usize> {
    with_module_table(|modules| {
        for (i, entry) in modules.iter().enumerate() {
            if entry.base == handle {
                return Some(i);
            }
        }
        None
    })
}

// ========================================================================
// LoadLibraryA / LoadLibraryW / LoadLibraryExA / LoadLibraryExW
// ========================================================================

pub fn load_library_a(lib_name: *const u8) -> Hmodule {
    let ex = LoadLibraryExFlags::default();
    load_library_ex_a(lib_name, ptr::null_mut(), ex)
}

pub fn load_library_w(lib_name: *const Wchar) -> Hmodule {
    let ex = LoadLibraryExFlags::default();
    load_library_ex_w(lib_name, ptr::null_mut(), ex)
}

struct LoadLibraryExFlags;
impl LoadLibraryExFlags {
    fn default() -> Dword {
        0
    }
    fn search_application_dir() -> Dword {
        0x00000200
    }
    fn dont_resolve_dll_refs() -> Dword {
        0x00000001
    }
    fn load_ignore_code_authz_level() -> Dword {
        0x00000010
    }
    fn load_library_as_datafile() -> Dword {
        0x00000002
    }
    fn load_library_as_datafile_exclusive() -> Dword {
        0x00000040
    }
    fn load_library_as_image_resource() -> Dword {
        0x00000020
    }
    fn load_with_altered_search_path() -> Dword {
        0x00000008
    }
}

pub fn load_library_ex_a(lib_name: *const u8, _file: Handle, _flags: Dword) -> Hmodule {
    if lib_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let mut len: usize = 0;
    unsafe {
        while *lib_name.add(len) != 0 { len += 1; }
    }
    let mut wide_name: Vec<Wchar> = alloc::vec![0u16; len + 1];
    for i in 0..len {
        wide_name[i] = unsafe { *lib_name.add(i) } as Wchar;
    }
    load_library_ex_w(wide_name.as_ptr(), _file, _flags)
}

pub fn load_library_ex_w(lib_name: *const Wchar, _file: Handle, _flags: Dword) -> Hmodule {
    if lib_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let name_len = lstrlen_w(lib_name);
    let name_slice = unsafe { core::slice::from_raw_parts(lib_name, name_len) };
    if let Some(h) = find_module_by_name(name_slice) {
        return h;
    }
    let base_val = MODULE_STATE.next_base.fetch_add(0x100000, Ordering::Relaxed);
    let base = base_val as *mut core::ffi::c_void;
    let entry = ModuleEntry {
        name: name_slice.to_vec(),
        file_name: name_slice.to_vec(),
        base,
        exports: BTreeMap::new(),
        ref_count: 1,
    };
    with_module_table(|modules| {
        modules.push(entry);
    });
    base
}

// ========================================================================
// GetModuleHandleA / GetModuleHandleW / GetModuleHandleExA / GetModuleHandleExW
// ========================================================================

pub fn get_module_handle_a(module_name: *const u8) -> Hmodule {
    if module_name.is_null() {
        return with_module_table(|modules| {
            modules.first().map(|e| e.base).unwrap_or(ptr::null_mut())
        });
    }
    let mut len: usize = 0;
    unsafe {
        while *module_name.add(len) != 0 { len += 1; }
    }
    let mut wide_name: Vec<Wchar> = alloc::vec![0u16; len + 1];
    for i in 0..len {
        wide_name[i] = unsafe { *module_name.add(i) } as Wchar;
    }
    get_module_handle_w(wide_name.as_ptr())
}

pub fn get_module_handle_w(module_name: *const Wchar) -> Hmodule {
    if module_name.is_null() {
        return with_module_table(|modules| {
            modules.first().map(|e| e.base).unwrap_or(ptr::null_mut())
        });
    }
    let name_len = lstrlen_w(module_name);
    let name_slice = unsafe { core::slice::from_raw_parts(module_name, name_len) };
    find_module_by_name(name_slice).unwrap_or_else(|| {
        set_last_error(ERROR_MOD_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn get_module_handle_ex_a(_flags: Dword, module_name: *const u8) -> Bool {
    if module_name.is_null() {
        return TRUE;
    }
    let mut len: usize = 0;
    unsafe {
        while *module_name.add(len) != 0 { len += 1; }
    }
    let mut wide_name: Vec<Wchar> = alloc::vec![0u16; len + 1];
    for i in 0..len {
        wide_name[i] = unsafe { *module_name.add(i) } as Wchar;
    }
    get_module_handle_w(wide_name.as_ptr());
    TRUE
}

pub fn get_module_handle_ex_w(
    _flags: Dword,
    _module_name: *const Wchar,
    _out_module: *mut Hmodule,
) -> Bool {
    TRUE
}

// ========================================================================
// GetModuleFileNameA / GetModuleFileNameW
// ========================================================================

pub fn get_module_file_name_a(module: Hmodule, filename: *mut u8, size: Dword) -> Dword {
    if module.is_null() {
        let name = with_module_table(|modules| {
            modules.first().map(|e| e.file_name.clone())
        });
        if let Some(name) = name {
            let mut i: usize = 0;
            while i < (size as usize) - 1 && i < name.len() {
                unsafe { *filename.add(i) = name[i] as u8; }
                i += 1;
            }
            unsafe { *filename.add(i) = 0; }
            return i as Dword;
        }
        return 0;
    }
    if let Some(idx) = find_module_by_handle(module) {
        with_module_table(|modules| {
            let name = &modules[idx].file_name;
            let mut i: usize = 0;
            while i < (size as usize) - 1 && i < name.len() {
                unsafe { *filename.add(i) = name[i] as u8; }
                i += 1;
            }
            unsafe { *filename.add(i) = 0; }
            i as Dword
        })
    } else {
        0
    }
}

pub fn get_module_file_name_w(module: Hmodule, filename: *mut Wchar, size: Dword) -> Dword {
    if module.is_null() {
        let name = with_module_table(|modules| {
            modules.first().map(|e| e.file_name.clone())
        });
        if let Some(name) = name {
            let copy_len = (size as usize).min(name.len() + 1);
            unsafe {
                for i in 0..copy_len.min(name.len()) {
                    *filename.add(i) = name[i];
                }
                *filename.add(copy_len - 1) = 0;
            }
            return (copy_len - 1) as Dword;
        }
        return 0;
    }
    if let Some(idx) = find_module_by_handle(module) {
        with_module_table(|modules| {
            let name = &modules[idx].file_name;
            let copy_len = (size as usize).min(name.len() + 1);
            unsafe {
                for i in 0..copy_len.min(name.len()) {
                    *filename.add(i) = name[i];
                }
                *filename.add(copy_len - 1) = 0;
            }
            (copy_len - 1) as Dword
        })
    } else {
        0
    }
}

// ========================================================================
// FreeLibrary / FreeLibraryAndExitThread
// ========================================================================

pub fn free_library(module: Hmodule) -> Bool {
    if let Some(idx) = find_module_by_handle(module) {
        with_module_table(|modules| {
            let entry = &mut modules[idx];
            if entry.ref_count > 1 {
                entry.ref_count -= 1;
            } else {
                modules.remove(idx);
            }
        });
        TRUE
    } else {
        FALSE
    }
}

pub fn free_library_and_exit_thread(module: Hmodule, exit_code: Dword) {
    free_library(module);
    exit_thread(exit_code);
}

pub fn exit_thread(_exit_code: Dword) {}

// ========================================================================
// GetProcAddress
// ========================================================================

pub fn get_proc_address(module: Hmodule, func_name: *const u8) -> FarProc {
    let mod_handle = if module.is_null() {
        with_module_table(|modules| {
            modules.first().map(|e| e.base).unwrap_or(ptr::null_mut())
        })
    } else {
        module
    };
    if mod_handle.is_null() {
        set_last_error(ERROR_MOD_NOT_FOUND);
        return None;
    }
    if func_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return None;
    }
    if let Some(idx) = find_module_by_handle(mod_handle) {
        with_module_table(|modules| {
            let entry = &modules[idx];
            let mut name_vec: Vec<u8> = Vec::new();
            unsafe {
                let mut i = 0;
                loop {
                    let c = *func_name.add(i);
                    if c == 0 { break; }
                    name_vec.push(c);
                    i += 1;
                }
            }
            if let Some(&proc) = entry.exports.get(&name_vec) {
                proc
            } else {
                set_last_error(ERROR_PROC_NOT_FOUND);
                None
            }
        })
    } else {
        set_last_error(ERROR_MOD_NOT_FOUND);
        None
    }
}

// ========================================================================
// DisableThreadLibraryCalls
// ========================================================================

pub fn disable_thread_library_calls(_module: Hmodule) -> Bool {
    TRUE
}

// ========================================================================
// GetModuleInformation / GetModuleBaseNameA / GetModuleBaseNameW (psapi)
// ========================================================================

struct ModuleInfo {
    base_of_dll: *mut core::ffi::c_void,
    size_of_image: Dword,
    entry_point: *mut core::ffi::c_void,
}

pub fn get_module_information(
    _process: Handle,
    _module: Hmodule,
    _modinfo: *mut ModuleInfo,
    _cb: Dword,
) -> Bool {
    TRUE
}

pub fn get_module_base_name_a(
    _process: Handle,
    module: Hmodule,
    base_name: *mut u8,
    size: Dword,
) -> Dword {
    let handle = if module.is_null() {
        with_module_table(|modules| {
            modules.first().map(|e| e.base).unwrap_or(ptr::null_mut())
        })
    } else {
        module
    };
    if handle.is_null() {
        return 0;
    }
    if let Some(idx) = find_module_by_handle(handle) {
        with_module_table(|modules| {
            let name = &modules[idx].name;
            let mut i: usize = 0;
            while i < (size as usize) - 1 && i < name.len() {
                unsafe { *base_name.add(i) = name[i] as u8; }
                i += 1;
            }
            unsafe { *base_name.add(i) = 0; }
            i as Dword
        })
    } else {
        0
    }
}

pub fn get_module_base_name_w(
    _process: Handle,
    module: Hmodule,
    base_name: *mut Wchar,
    size: Dword,
) -> Dword {
    let handle = if module.is_null() {
        with_module_table(|modules| {
            modules.first().map(|e| e.base).unwrap_or(ptr::null_mut())
        })
    } else {
        module
    };
    if handle.is_null() {
        return 0;
    }
    if let Some(idx) = find_module_by_handle(handle) {
        with_module_table(|modules| {
            let name = &modules[idx].name;
            let copy_len = (size as usize).min(name.len() + 1);
            unsafe {
                for i in 0..copy_len.min(name.len()) {
                    *base_name.add(i) = name[i];
                }
                *base_name.add(copy_len - 1) = 0;
            }
            (copy_len - 1) as Dword
        })
    } else {
        0
    }
}
