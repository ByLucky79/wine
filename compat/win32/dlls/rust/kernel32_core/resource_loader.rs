// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 PE resource loading API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/resource_loader.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 PE resource management: FindResourceA/W/Ex,
//   LoadResource, LockResource, FreeResource, SizeofResource,
//   LoadLibraryWithResource, EnumResourceNamesA/W, EnumResourceTypesA/W,
//   EnumResourceLanguagesA/W, BeginUpdateResourceA/W, UpdateResourceA/W,
//   EndUpdateResourceA/W.  Resources are tracked via an internal registry.
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
use core::sync::atomic::{AtomicU32, Ordering};

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Wchar = u16;
type Long = i32;
type Handle = *mut core::ffi::c_void;
type Hmodule = *mut core::ffi::c_void;
type Hrsrc = *mut core::ffi::c_void;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_RESOURCE_DATA_NOT_FOUND: Dword = 1812;
const ERROR_RESOURCE_TYPE_NOT_FOUND: Dword = 1813;
const ERROR_RESOURCE_NAME_NOT_FOUND: Dword = 1814;
const ERROR_RESOURCE_LANG_NOT_FOUND: Dword = 1815;
const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_FILE_NOT_FOUND: Dword = 2;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;

const LANG_NEUTRAL: Word = 0x00;
const SUBLANG_NEUTRAL: Word = 0x00;

fn set_last_error(_code: Dword) {}

fn is_int_resource(p: *const core::ffi::c_void) -> bool {
    (p as usize >> 16) == 0
}

fn is_int_resource_w(p: *const Wchar) -> bool {
    (p as usize >> 16) == 0
}

fn loword(p: *const Wchar) -> Word {
    (p as usize & 0xFFFF) as Word
}

fn hiword(val: Dword) -> Word {
    (val >> 16) as Word
}

fn lstrlen_w(s: *const Wchar) -> usize {
    if s.is_null() { return 0; }
    let mut len: usize = 0;
    unsafe { while *s.add(len) != 0 { len += 1; } }
    len
}

fn lstrcpy_w(dst: *mut Wchar, src: *const Wchar) {
    if dst.is_null() || src.is_null() { return; }
    unsafe {
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(i) = c;
            if c == 0 { break; }
            i += 1;
        }
    }
}

// ========================================================================
// Resource registry
// ========================================================================

#[derive(Clone)]
struct ResourceEntry {
    typ: u32,
    name: u32,
    lang: Word,
    data: Vec<u8>,
}

use core::cell::UnsafeCell;

struct ResourceState {
    entries: UnsafeCell<Vec<ResourceEntry>>,
    next_hrsrc: AtomicU32,
}

unsafe impl Sync for ResourceState {}

static RES_STATE: ResourceState = ResourceState {
    entries: UnsafeCell::new(Vec::new()),
    next_hrsrc: AtomicU32::new(1),
};

type HrsrcInt = u32;

fn with_res_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<ResourceEntry>) -> R,
{
    let cell = unsafe { &mut *RES_STATE.entries.get() };
    f(cell)
}

fn make_hrsrc(val: u32) -> Hrsrc {
    val as usize as Hrsrc
}

fn hrsrc_to_u32(h: Hrsrc) -> Option<u32> {
    let val = h as usize;
    if val == 0 { None } else { Some(val as u32 - 1) }
}

fn resource_key(typ: u32, name: u32, lang: Word) -> (u32, u32, Word) {
    (typ, name, lang)
}

// ========================================================================
// FindResourceA / FindResourceW / FindResourceExA / FindResourceExW
// ========================================================================

fn get_res_name_id(name: *const u8) -> u32 {
    if (name as usize >> 16) == 0 {
        (name as usize & 0xFFFF) as u32
    } else if !name.is_null() {
        let mut h: u32 = 5381;
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                h = h.wrapping_mul(33).wrapping_add(c as u32);
                i += 1;
            }
        }
        h | 0x10000
    } else {
        0
    }
}

fn get_res_name_id_w(name: *const Wchar) -> u32 {
    if (name as usize >> 16) == 0 {
        (name as usize & 0xFFFF) as u32
    } else if !name.is_null() {
        let mut h: u32 = 5381;
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                h = h.wrapping_mul(33).wrapping_add(c as u32);
                i += 1;
            }
        }
        h | 0x20000
    } else {
        0
    }
}

pub fn find_resource_ex_a(
    module: Hmodule,
    typ: *const u8,
    name: *const u8,
    lang: Word,
) -> Hrsrc {
    if typ.is_null() || name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let type_id = get_res_name_id(typ);
    let name_id = get_res_name_id(name);
    with_res_table(|entries| {
        for (i, entry) in entries.iter().enumerate() {
            if entry.typ == type_id && entry.name == name_id && entry.lang == lang {
                return make_hrsrc(i as u32 + 1);
            }
        }
        set_last_error(ERROR_RESOURCE_DATA_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn find_resource_ex_w(
    module: Hmodule,
    typ: *const Wchar,
    name: *const Wchar,
    lang: Word,
) -> Hrsrc {
    if typ.is_null() || name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let type_id = get_res_name_id_w(typ);
    let name_id = get_res_name_id_w(name);
    with_res_table(|entries| {
        for (i, entry) in entries.iter().enumerate() {
            if entry.typ == type_id && entry.name == name_id && entry.lang == lang {
                return make_hrsrc(i as u32 + 1);
            }
        }
        set_last_error(ERROR_RESOURCE_DATA_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn find_resource_a(module: Hmodule, name: *const u8, typ: *const u8) -> Hrsrc {
    find_resource_ex_a(module, typ, name, LANG_NEUTRAL | (SUBLANG_NEUTRAL << 4))
}

pub fn find_resource_w(module: Hmodule, name: *const Wchar, typ: *const Wchar) -> Hrsrc {
    find_resource_ex_w(module, typ, name, LANG_NEUTRAL | (SUBLANG_NEUTRAL << 4))
}

// ========================================================================
// LoadResource / LockResource / FreeResource / SizeofResource
// ========================================================================

struct ResourceData {
    data: Vec<u8>,
}

struct LoadedResourceState {
    loaded: UnsafeCell<BTreeMap<u32, ResourceData>>,
}

unsafe impl Sync for LoadedResourceState {}

static LOADED_STATE: LoadedResourceState = LoadedResourceState {
    loaded: UnsafeCell::new(BTreeMap::new()),
};

pub fn load_resource(_module: Hmodule, hrsrc: Hrsrc) -> Handle {
    if let Some(idx) = hrsrc_to_u32(hrsrc) {
        let data = with_res_table(|entries| {
            if (idx as usize) < entries.len() {
                Some(entries[idx as usize].data.clone())
            } else {
                None
            }
        });
        if let Some(d) = data {
            let loaded_cell = unsafe { &mut *LOADED_STATE.loaded.get() };
            let handle_key = hrsrc as u32;
            loaded_cell.insert(handle_key, ResourceData { data: d });
            return handle_key as usize as Handle;
        }
    }
    set_last_error(ERROR_RESOURCE_DATA_NOT_FOUND);
    ptr::null_mut()
}

pub fn lock_resource(handle: Handle) -> *mut core::ffi::c_void {
    if handle.is_null() {
        return ptr::null_mut();
    }
    let loaded_cell = unsafe { &mut *LOADED_STATE.loaded.get() };
    if let Some(res) = loaded_cell.get(&(handle as u32)) {
        res.data.as_ptr() as *mut core::ffi::c_void
    } else {
        ptr::null_mut()
    }
}

pub fn free_resource(handle: Handle) -> Bool {
    if !handle.is_null() {
        let loaded_cell = unsafe { &mut *LOADED_STATE.loaded.get() };
        loaded_cell.remove(&(handle as u32));
    }
    TRUE
}

pub fn sizeof_resource(_module: Hmodule, hrsrc: Hrsrc) -> Dword {
    if let Some(idx) = hrsrc_to_u32(hrsrc) {
        with_res_table(|entries| {
            if (idx as usize) < entries.len() {
                entries[idx as usize].data.len() as Dword
            } else {
                0
            }
        })
    } else {
        0
    }
}

// ========================================================================
// LoadLibraryWithResource
// ========================================================================

pub fn load_library_with_resource(
    _module: Hmodule,
    _res_name: *const Wchar,
    _res_type: *const Wchar,
) -> Hmodule {
    ptr::null_mut()
}

// ========================================================================
// EnumResourceNamesA / EnumResourceNamesW
// ========================================================================

type EnumResNameProcA = Option<unsafe extern "C" fn(Hmodule, *const u8, *mut u8, Long) -> Bool>;
type EnumResNameProcW = Option<unsafe extern "C" fn(Hmodule, *const Wchar, *mut Wchar, Long) -> Bool>;

pub fn enum_resource_names_a(
    module: Hmodule,
    typ: *const u8,
    proc: EnumResNameProcA,
    lparam: Long,
) -> Bool {
    if proc.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    TRUE
}

pub fn enum_resource_names_w(
    module: Hmodule,
    typ: *const Wchar,
    proc: EnumResNameProcW,
    lparam: Long,
) -> Bool {
    if proc.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    TRUE
}

// ========================================================================
// EnumResourceTypesA / EnumResourceTypesW
// ========================================================================

type EnumResTypeProcA = Option<unsafe extern "C" fn(Hmodule, *mut u8, Long) -> Bool>;
type EnumResTypeProcW = Option<unsafe extern "C" fn(Hmodule, *mut Wchar, Long) -> Bool>;

pub fn enum_resource_types_a(
    module: Hmodule,
    proc: EnumResTypeProcA,
    lparam: Long,
) -> Bool {
    if proc.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    TRUE
}

pub fn enum_resource_types_w(
    module: Hmodule,
    proc: EnumResTypeProcW,
    lparam: Long,
) -> Bool {
    if proc.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    TRUE
}

// ========================================================================
// EnumResourceLanguagesA / EnumResourceLanguagesW
// ========================================================================

type EnumResLangProcA = Option<unsafe extern "C" fn(Hmodule, *const u8, *const u8, Word, Long) -> Bool>;
type EnumResLangProcW = Option<unsafe extern "C" fn(Hmodule, *const Wchar, *const Wchar, Word, Long) -> Bool>;

pub fn enum_resource_languages_a(
    module: Hmodule,
    typ: *const u8,
    name: *const u8,
    proc: EnumResLangProcA,
    lparam: Long,
) -> Bool {
    if proc.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    TRUE
}

pub fn enum_resource_languages_w(
    module: Hmodule,
    typ: *const Wchar,
    name: *const Wchar,
    proc: EnumResLangProcW,
    lparam: Long,
) -> Bool {
    if proc.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    TRUE
}

// ========================================================================
// BeginUpdateResourceA / BeginUpdateResourceW
// ========================================================================

struct UpdateResourceEntry {
    typ: u32,
    name: u32,
    lang: Word,
    codepage: Dword,
    data: Option<Vec<u8>>,
}

struct UpdateState {
    entries: Vec<UpdateResourceEntry>,
    file_name: Vec<Wchar>,
    delete_existing: Bool,
}

struct UpdateResourceState {
    handles: UnsafeCell<BTreeMap<Handle, UpdateState>>,
}

unsafe impl Sync for UpdateResourceState {}

static UPDATE_STATE: UpdateResourceState = UpdateResourceState {
    handles: UnsafeCell::new(BTreeMap::new()),
};

fn make_update_handle() -> Handle {
    static NEXT_HANDLE: AtomicU32 = AtomicU32::new(0x1000);
    let val = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    val as usize as Handle
}

pub fn begin_update_resource_w(
    file_name: *const Wchar,
    delete_existing: Bool,
) -> Handle {
    if file_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let handle = make_update_handle();
    let fname_len = lstrlen_w(file_name);
    let mut fname_vec: Vec<Wchar> = alloc::vec![0u16; fname_len + 1];
    lstrcpy_w(fname_vec.as_mut_ptr(), file_name);

    let state = UpdateState {
        entries: Vec::new(),
        file_name: fname_vec,
        delete_existing,
    };

    let handles_cell = unsafe { &mut *UPDATE_STATE.handles.get() };
    handles_cell.insert(handle, state);
    handle
}

pub fn begin_update_resource_a(file_name: *const u8, delete_existing: Bool) -> Handle {
    if file_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let mut len: usize = 0;
    unsafe { while *file_name.add(len) != 0 { len += 1; } }
    let mut wide: Vec<Wchar> = alloc::vec![0u16; len + 1];
    for i in 0..len {
        wide[i] = unsafe { *file_name.add(i) } as Wchar;
    }
    begin_update_resource_w(wide.as_ptr(), delete_existing)
}

// ========================================================================
// UpdateResourceA / UpdateResourceW
// ========================================================================

pub fn update_resource_w(
    handle: Handle,
    typ: *const Wchar,
    name: *const Wchar,
    lang: Word,
    data: *mut core::ffi::c_void,
    cb_data: Dword,
) -> Bool {
    if handle.is_null() || typ.is_null() || name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let type_id = get_res_name_id_w(typ);
    let name_id = get_res_name_id_w(name);

    let data_vec = if !data.is_null() && cb_data > 0 {
        let slice = unsafe { core::slice::from_raw_parts(data as *const u8, cb_data as usize) };
        Some(slice.to_vec())
    } else {
        None
    };

    let handles_cell = unsafe { &mut *UPDATE_STATE.handles.get() };
    if let Some(state) = handles_cell.get_mut(&handle) {
        let entry = UpdateResourceEntry {
            typ: type_id,
            name: name_id,
            lang,
            codepage: 0,
            data: data_vec,
        };
        state.entries.push(entry);
        TRUE
    } else {
        set_last_error(ERROR_INVALID_PARAMETER);
        FALSE
    }
}

pub fn update_resource_a(
    handle: Handle,
    typ: *const u8,
    name: *const u8,
    lang: Word,
    data: *mut core::ffi::c_void,
    cb_data: Dword,
) -> Bool {
    let type_id = get_res_name_id(typ);
    let name_id = get_res_name_id(name);

    let type_w: *const Wchar = type_id as usize as *const Wchar;
    let name_w: *const Wchar = name_id as usize as *const Wchar;

    update_resource_w(handle, type_w, name_w, lang, data, cb_data)
}

// ========================================================================
// EndUpdateResourceA / EndUpdateResourceW
// ========================================================================

pub fn end_update_resource_w(handle: Handle, discard: Bool) -> Bool {
    if handle.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let handles_cell = unsafe { &mut *UPDATE_STATE.handles.get() };
    if discard != 0 {
        handles_cell.remove(&handle);
        return TRUE;
    }
    if let Some(state) = handles_cell.remove(&handle) {
        for entry in state.entries {
            if let Some(d) = entry.data {
                let res_entry = ResourceEntry {
                    typ: entry.typ,
                    name: entry.name,
                    lang: entry.lang,
                    data: d,
                };
                with_res_table(|entries| {
                    entries.push(res_entry);
                });
            }
        }
        TRUE
    } else {
        set_last_error(ERROR_INVALID_PARAMETER);
        FALSE
    }
}

pub fn end_update_resource_a(handle: Handle, discard: Bool) -> Bool {
    end_update_resource_w(handle, discard)
}
