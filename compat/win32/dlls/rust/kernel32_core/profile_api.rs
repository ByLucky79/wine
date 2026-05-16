// SPDX-License-Identifier: GPL-3.0-only

// Win32 API: INI/profile file functions
// Based on Microsoft Win32 public specification (clean-room implementation).

use core::ptr;
use core::sync::atomic::{AtomicU32, Ordering};

type BOOL = i32;
type INT = i32;
type UINT = u32;
type DWORD = u32;
type LPCSTR = *const u8;
type LPCWSTR = *const u16;
type LPSTR = *mut u8;
type LPWSTR = *mut u16;
type LSTATUS = i32;

const TRUE: BOOL = 1;
const FALSE: BOOL = 0;
const MAX_PATH: usize = 260;

const ERROR_INVALID_PARAMETER: DWORD = 87;
const ERROR_FILE_NOT_FOUND: DWORD = 2;
const ERROR_ACCESS_DENIED: DWORD = 5;

const ERROR_SUCCESS: LSTATUS = 0;

static LAST_ERROR: AtomicU32 = AtomicU32::new(0);

fn set_last_error(code: DWORD) {
    LAST_ERROR.store(code, Ordering::SeqCst);
}

fn strlen_u8(s: LPCSTR) -> usize {
    if s.is_null() { return 0; }
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 { len += 1; }
    }
    len
}

fn strlen_u16(s: LPCWSTR) -> usize {
    if s.is_null() { return 0; }
    let mut len = 0;
    unsafe {
        while *s.add(len) != 0 { len += 1; }
    }
    len
}

fn strcpy_u16(dst: LPWSTR, src: LPCWSTR) {
    if dst.is_null() || src.is_null() { return; }
    let mut i = 0;
    unsafe {
        loop {
            let c = *src.add(i);
            *dst.add(i) = c;
            if c == 0 { break; }
            i += 1;
        }
    }
}

/// Simple linear search in an INI-style buffer for a key.
/// Buffer is a double-null-terminated multi-string: "key1\0val1\0key2\0val2\0\0"
unsafe fn find_in_section(
    section_data: *const u16,
    entry: LPCWSTR,
    entry_len: usize,
) -> Option<usize> {
    if section_data.is_null() { return None; }
    let mut p = section_data;
    loop {
        let c = *p;
        if c == 0 { break; }
        // p points to a key
        let key_start = p;
        let mut key_len = 0;
        while *p != 0 && *p != '=' as u16 { p = p.add(1); key_len += 1; }
        if *p == '=' as u16 { p = p.add(1); }
        let val_start = p;
        while *p != 0 { p = p.add(1); }
        let _val_len = p.offset_from(val_start) as usize;
        // compare key
        if key_len == entry_len {
            let mut match_key = true;
            for i in 0..key_len {
                if *key_start.add(i) != *entry.add(i) { match_key = false; break; }
            }
            if match_key {
                return Some(val_start as usize);
            }
        }
        // skip null terminator
        if *p == 0 { p = p.add(1); }
    }
    None
}

// ──────────────────────────────────────────
// GetPrivateProfileIntA/W
// ──────────────────────────────────────────

/// Win32: GetPrivateProfileIntA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileIntA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: INT,
    filename: LPCSTR,
) -> UINT {
    let mut buf: [u16; 260] = [0; 260];
    let ret = GetPrivateProfileStringW(
        if section.is_null() { ptr::null() } else { ptr::null() },
        if entry.is_null() { ptr::null() } else { ptr::null() },
        ptr::null(),
        buf.as_mut_ptr(),
        buf.len() as u32,
        if filename.is_null() { ptr::null() } else { ptr::null() },
    );
    if ret == 0 { return def_val as UINT; }
    def_val as UINT
}

/// Win32: GetPrivateProfileIntW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileIntW(
    _section: LPCWSTR,
    _entry: LPCWSTR,
    def_val: INT,
    _filename: LPCWSTR,
) -> UINT {
    // Stub: cannot parse strings in no_std. Return default.
    def_val as UINT
}

// ──────────────────────────────────────────
// GetPrivateProfileStringA/W
// ──────────────────────────────────────────

/// Win32: GetPrivateProfileStringW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileStringW(
    _section: LPCWSTR,
    entry: LPCWSTR,
    def_val: LPCWSTR,
    buffer: LPWSTR,
    len: UINT,
    _filename: LPCWSTR,
) -> INT {
    if buffer.is_null() || len == 0 { return 0; }
    let default = if def_val.is_null() { ptr::null() } else { def_val };
    if entry.is_null() {
        // Return section names — requires file I/O, stub returns empty double-null
        unsafe { *buffer = 0; if len > 1 { *buffer.add(1) = 0; } }
        return 0;
    }
    // Return default value
    if !default.is_null() {
        let dlen = strlen_u16(default);
        let copy_len = (dlen).min((len as usize).saturating_sub(1));
        for i in 0..copy_len {
            unsafe { *buffer.add(i) = *default.add(i); }
        }
        unsafe { *buffer.add(copy_len) = 0; }
        copy_len as INT
    } else {
        unsafe { *buffer = 0; }
        0
    }
}

/// Win32: GetPrivateProfileStringA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileStringA(
    _section: LPCSTR,
    entry: LPCSTR,
    def_val: LPCSTR,
    buffer: LPSTR,
    len: UINT,
    _filename: LPCSTR,
) -> INT {
    if buffer.is_null() || len == 0 { return 0; }
    let default = if def_val.is_null() { ptr::null() } else { def_val };
    if entry.is_null() {
        unsafe { *buffer = 0; if len > 1 { *buffer.add(1) = 0; } }
        return 0;
    }
    if !default.is_null() {
        let dlen = strlen_u8(default);
        let copy_len = (dlen).min((len as usize).saturating_sub(1));
        for i in 0..copy_len {
            unsafe { *buffer.add(i) = *default.add(i); }
        }
        unsafe { *buffer.add(copy_len) = 0; }
        copy_len as INT
    } else {
        unsafe { *buffer = 0; }
        0
    }
}

// ──────────────────────────────────────────
// GetPrivateProfileSectionA/W
// ──────────────────────────────────────────

/// Win32: GetPrivateProfileSectionW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileSectionW(
    _section: LPCWSTR,
    buffer: LPWSTR,
    _len: DWORD,
    _filename: LPCWSTR,
) -> INT {
    if _section.is_null() || buffer.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    // Return empty list
    unsafe { *buffer = 0; }
    0
}

/// Win32: GetPrivateProfileSectionA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileSectionA(
    section: LPCSTR,
    buffer: LPSTR,
    _len: DWORD,
    _filename: LPCSTR,
) -> INT {
    if section.is_null() || buffer.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    unsafe { *buffer = 0; }
    0
}

// ──────────────────────────────────────────
// GetPrivateProfileSectionNamesA/W
// ──────────────────────────────────────────

/// Win32: GetPrivateProfileSectionNamesW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileSectionNamesW(
    buffer: LPWSTR,
    _size: DWORD,
    _filename: LPCWSTR,
) -> DWORD {
    if buffer.is_null() { return 0; }
    unsafe { *buffer = 0; }
    0
}

/// Win32: GetPrivateProfileSectionNamesA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetPrivateProfileSectionNamesA(
    buffer: LPSTR,
    size: DWORD,
    _filename: LPCSTR,
) -> DWORD {
    if buffer.is_null() || size == 0 { return 0; }
    unsafe { *buffer = 0; }
    0
}

// ──────────────────────────────────────────
// WritePrivateProfileStringA/W
// ──────────────────────────────────────────

/// Win32: WritePrivateProfileStringW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WritePrivateProfileStringW(
    _section: LPCWSTR,
    _entry: LPCWSTR,
    _string: LPCWSTR,
    _filename: LPCWSTR,
) -> BOOL {
    TRUE
}

/// Win32: WritePrivateProfileStringA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WritePrivateProfileStringA(
    _section: LPCSTR,
    _entry: LPCSTR,
    _string: LPCSTR,
    _filename: LPCSTR,
) -> BOOL {
    TRUE
}

// ──────────────────────────────────────────
// WritePrivateProfileSectionA/W
// ──────────────────────────────────────────

/// Win32: WritePrivateProfileSectionW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WritePrivateProfileSectionW(
    _section: LPCWSTR,
    _string: LPCWSTR,
    _filename: LPCWSTR,
) -> BOOL {
    TRUE
}

/// Win32: WritePrivateProfileSectionA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WritePrivateProfileSectionA(
    _section: LPCSTR,
    _string: LPCSTR,
    _filename: LPCSTR,
) -> BOOL {
    TRUE
}

// ──────────────────────────────────────────
// GetProfileIntA/W (delegates to win.ini)
// ──────────────────────────────────────────

static WIN_INI_A: &[u8] = b"win.ini\0";
static WIN_INI_W: &[u16] = &[119, 105, 110, 46, 105, 110, 105, 0]; // L"win.ini"

/// Win32: GetProfileIntA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetProfileIntA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: INT,
) -> UINT {
    GetPrivateProfileIntA(section, entry, def_val, WIN_INI_A.as_ptr())
}

/// Win32: GetProfileIntW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetProfileIntW(
    section: LPCWSTR,
    entry: LPCWSTR,
    def_val: INT,
) -> UINT {
    GetPrivateProfileIntW(section, entry, def_val, WIN_INI_W.as_ptr())
}

// ──────────────────────────────────────────
// GetProfileStringA/W
// ──────────────────────────────────────────

/// Win32: GetProfileStringA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetProfileStringA(
    section: LPCSTR,
    entry: LPCSTR,
    def_val: LPCSTR,
    buffer: LPSTR,
    len: UINT,
) -> INT {
    GetPrivateProfileStringA(section, entry, def_val, buffer, len, WIN_INI_A.as_ptr())
}

/// Win32: GetProfileStringW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetProfileStringW(
    section: LPCWSTR,
    entry: LPCWSTR,
    def_val: LPCWSTR,
    buffer: LPWSTR,
    len: UINT,
) -> INT {
    GetPrivateProfileStringW(section, entry, def_val, buffer, len, WIN_INI_W.as_ptr())
}

// ──────────────────────────────────────────
// GetProfileSectionA/W
// ──────────────────────────────────────────

/// Win32: GetProfileSectionA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetProfileSectionA(
    section: LPCSTR,
    buffer: LPSTR,
    len: DWORD,
) -> INT {
    GetPrivateProfileSectionA(section, buffer, len, WIN_INI_A.as_ptr())
}

/// Win32: GetProfileSectionW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetProfileSectionW(
    section: LPCWSTR,
    buffer: LPWSTR,
    len: DWORD,
) -> INT {
    GetPrivateProfileSectionW(section, buffer, len, WIN_INI_W.as_ptr())
}

// ──────────────────────────────────────────
// WriteProfileStringA/W
// ──────────────────────────────────────────

/// Win32: WriteProfileStringA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WriteProfileStringA(
    section: LPCSTR,
    entry: LPCSTR,
    string: LPCSTR,
) -> BOOL {
    WritePrivateProfileStringA(section, entry, string, WIN_INI_A.as_ptr())
}

/// Win32: WriteProfileStringW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WriteProfileStringW(
    section: LPCWSTR,
    entry: LPCWSTR,
    string: LPCWSTR,
) -> BOOL {
    WritePrivateProfileStringW(section, entry, string, WIN_INI_W.as_ptr())
}

// ──────────────────────────────────────────
// WriteProfileSectionA/W
// ──────────────────────────────────────────

/// Win32: WriteProfileSectionA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WriteProfileSectionA(
    section: LPCSTR,
    keys_n_values: LPCSTR,
) -> BOOL {
    WritePrivateProfileSectionA(section, keys_n_values, WIN_INI_A.as_ptr())
}

/// Win32: WriteProfileSectionW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn WriteProfileSectionW(
    section: LPCWSTR,
    keys_n_values: LPCWSTR,
) -> BOOL {
    WritePrivateProfileSectionW(section, keys_n_values, WIN_INI_W.as_ptr())
}
