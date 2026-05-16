// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 path/directory/file-system API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/path_utils.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 path and volume query functions: SearchPath,
//   GetFullPathName, GetTempPath, GetTempFileName, Get/SetCurrentDirectory,
//   GetSystemDirectory, GetWindowsDirectory, GetLongPathName,
//   GetShortPathName, GetDiskFreeSpace, GetVolumeInformation, GetDriveType,
//   GetLogicalDrives, GetLogicalDriveStrings.  All A/W variants provided.
//
// Bağımlı Dosyalar:
//   1-) alloc (crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Clean-room Rust port from Win32 API spec
// *******************************************************************

use alloc::vec::Vec;
use core::ptr;

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Long = i32;
type Wchar = u16;
type Handle = *mut core::ffi::c_void;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const MAX_PATH: usize = 260;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_INSUFFICIENT_BUFFER: Dword = 122;
const ERROR_FILE_NOT_FOUND: Dword = 2;
const ERROR_PATH_NOT_FOUND: Dword = 3;
const ERROR_FILENAME_EXCED_RANGE: Dword = 206;
const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;

const INVALID_HANDLE_VALUE: isize = -1;

const DRIVE_UNKNOWN: Uint = 0;
const DRIVE_NO_ROOT_DIR: Uint = 1;
const DRIVE_REMOVABLE: Uint = 2;
const DRIVE_FIXED: Uint = 3;
const DRIVE_REMOTE: Uint = 4;
const DRIVE_CDROM: Uint = 5;
const DRIVE_RAMDISK: Uint = 6;

fn set_last_error(_code: Dword) {}

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

fn strcpy_a(dst: *mut u8, src: *const u8) {
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

fn wcsrchr_w(s: *const Wchar, ch: Wchar) -> *const Wchar {
    if s.is_null() { return ptr::null(); }
    let mut last: *const Wchar = ptr::null();
    let mut i: usize = 0;
    unsafe {
        loop {
            let c = *s.add(i);
            if c == 0 { break; }
            if c == ch { last = s.add(i); }
            i += 1;
        }
    }
    last
}

fn wcscpy_w(dst: *mut Wchar, src: *const Wchar) {
    lstrcpy_w(dst, src);
}

fn wcscat_w(dst: *mut Wchar, src: *const Wchar) {
    let dlen = lstrlen_w(dst);
    unsafe {
        lstrcpy_w(dst.add(dlen), src);
    }
}

fn wcsicmp(a: *const Wchar, b: *const Wchar) -> i32 {
    unsafe {
        let mut i = 0;
        loop {
            let ca = *a.add(i);
            let cb = *b.add(i);
            if ca == 0 && cb == 0 { return 0; }
            let ua = if ca >= 0x0041 && ca <= 0x005A { ca + 0x0020 } else { ca };
            let ub = if cb >= 0x0041 && cb <= 0x005B { cb + 0x0020 } else { cb };
            if ua != ub { return if ua < ub { -1 } else { 1 }; }
            i += 1;
        }
    }
}

// ========================================================================
// SearchPathA / SearchPathW
// ========================================================================

pub fn search_path_a(
    _path: *const u8,
    file: *const u8,
    _extension: *const u8,
    buf_len: Dword,
    buf: *mut u8,
    _file_part: *mut *mut u8,
) -> Dword {
    if file.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut flen: usize = 0;
    unsafe { while *file.add(flen) != 0 { flen += 1; } }
    if flen + 1 > buf_len as usize {
        return (flen + 1) as Dword;
    }
    strcpy_a(buf, file);
    flen as Dword
}

pub fn search_path_w(
    _path: *const Wchar,
    file: *const Wchar,
    _extension: *const Wchar,
    buf_len: Dword,
    buf: *mut Wchar,
    _file_part: *mut *mut Wchar,
) -> Dword {
    if file.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let flen = lstrlen_w(file);
    if flen + 1 > buf_len as usize {
        return (flen + 1) as Dword;
    }
    lstrcpy_w(buf, file);
    flen as Dword
}

// ========================================================================
// GetFullPathNameA / GetFullPathNameW
// ========================================================================

pub fn get_full_path_name_a(
    name: *const u8,
    buf_len: Dword,
    buf: *mut u8,
    _file_part: *mut *mut u8,
) -> Dword {
    if name.is_null() || buf.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut nlen: usize = 0;
    unsafe { while *name.add(nlen) != 0 { nlen += 1; } }
    if nlen + 1 > buf_len as usize {
        return (nlen + 1) as Dword;
    }
    if nlen >= 2 && unsafe { *name.add(1) } == b':' {
        strcpy_a(buf, name);
    } else {
        unsafe { *buf.add(0) = b'C'; *buf.add(1) = b':'; *buf.add(2) = b'\\'; }
        strcpy_a(unsafe { buf.add(3) }, name);
    }
    nlen as Dword
}

pub fn get_full_path_name_w(
    name: *const Wchar,
    buf_len: Dword,
    buf: *mut Wchar,
    _file_part: *mut *mut Wchar,
) -> Dword {
    if name.is_null() || buf.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let nlen = lstrlen_w(name);
    if nlen + 1 > buf_len as usize {
        return (nlen + 1) as Dword;
    }
    if nlen >= 2 && unsafe { *name.add(1) } == b':' as Wchar {
        lstrcpy_w(buf, name);
    } else {
        unsafe {
            *buf.add(0) = b'C' as Wchar;
            *buf.add(1) = b':' as Wchar;
            *buf.add(2) = b'\\' as Wchar;
        }
        lstrcpy_w(unsafe { buf.add(3) }, name);
    }
    nlen as Dword
}

// ========================================================================
// GetTempPathA / GetTempPathW
// ========================================================================

pub fn get_temp_path_a(buf_len: Dword, buf: *mut u8) -> Dword {
    let tmp: &[u8] = b"C:\\TEMP\0";
    let len = tmp.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Dword;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = tmp[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Dword
}

pub fn get_temp_path_w(buf_len: Dword, buf: *mut Wchar) -> Dword {
    let tmp: &[Wchar] = &[
        b'C' as Wchar, b':' as Wchar, b'\\' as Wchar,
        b'T' as Wchar, b'E' as Wchar, b'M' as Wchar, b'P' as Wchar, 0,
    ];
    let len = tmp.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Dword;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = tmp[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Dword
}

// ========================================================================
// GetTempFileNameA / GetTempFileNameW
// ========================================================================

pub fn get_temp_file_name_a(
    path: *const u8,
    prefix: *const u8,
    unique: Uint,
    buf: *mut u8,
) -> Uint {
    if path.is_null() || buf.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let plen = {
        let mut l: usize = 0;
        unsafe { while *path.add(l) != 0 { l += 1; } }
        l
    };
    unsafe {
        for i in 0..plen {
            *buf.add(i) = *path.add(i);
        }
        *buf.add(plen) = b'\\';
    }
    let mut off = plen + 1;
    if !prefix.is_null() {
        unsafe {
            let mut i = 0;
            while *prefix.add(i) != 0 {
                *buf.add(off) = *prefix.add(i);
                off += 1;
                i += 1;
                if i >= 3 { break; }
            }
        }
    }
    let num = if unique == 0 { 1u32 } else { unique };
    let hex = alloc::format!("{:04X}.TMP\0", num & 0xFFFF);
    let hex_bytes = hex.as_bytes();
    for i in 0..hex_bytes.len() {
        unsafe { *buf.add(off + i) = hex_bytes[i]; }
    }
    if unique == 0 {
        let num_bytes = hex_bytes.len() - 5;
        unsafe { *buf.add(off + num_bytes) = 0; }
        (off + num_bytes) as Uint
    } else {
        unsafe { *buf.add(off + hex_bytes.len() - 1) = 0; }
        unique
    }
}

pub fn get_temp_file_name_w(
    path: *const Wchar,
    prefix: *const Wchar,
    unique: Uint,
    buf: *mut Wchar,
) -> Uint {
    if path.is_null() || buf.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let plen = lstrlen_w(path);
    unsafe {
        lstrcpy_w(buf, path);
        *buf.add(plen) = b'\\' as Wchar;
    }
    let mut off = plen + 1;
    if !prefix.is_null() {
        unsafe {
            let mut i = 0;
            while *prefix.add(i) != 0 {
                *buf.add(off) = *prefix.add(i);
                off += 1;
                i += 1;
                if i >= 3 { break; }
            }
        }
    }
    let num = if unique == 0 { 1u32 } else { unique };
    let hex = alloc::format!("{:04X}.TMP", num & 0xFFFF);
    let hex_w: Vec<Wchar> = hex.encode_utf16().chain(core::iter::once(0)).collect();
    for i in 0..hex_w.len() - 1 {
        unsafe { *buf.add(off + i) = hex_w[i]; }
    }
    unsafe { *buf.add(off + hex_w.len() - 1) = 0; }
    if unique == 0 { 0 } else { unique }
}

// ========================================================================
// GetCurrentDirectoryA / GetCurrentDirectoryW
// ========================================================================

pub fn get_current_directory_a(buf_len: Dword, buf: *mut u8) -> Dword {
    let cwd: &[u8] = b"C:\\\0";
    let len = cwd.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Dword;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = cwd[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Dword
}

pub fn get_current_directory_w(buf_len: Dword, buf: *mut Wchar) -> Dword {
    let cwd: &[Wchar] = &[
        b'C' as Wchar, b':' as Wchar, b'\\' as Wchar, 0,
    ];
    let len = cwd.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Dword;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = cwd[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Dword
}

// ========================================================================
// SetCurrentDirectoryA / SetCurrentDirectoryW
// ========================================================================

pub fn set_current_directory_a(_path: *const u8) -> Bool {
    TRUE
}

pub fn set_current_directory_w(_path: *const Wchar) -> Bool {
    TRUE
}

// ========================================================================
// GetSystemDirectoryA / GetSystemDirectoryW
// ========================================================================

pub fn get_system_directory_a(buf_len: Uint, buf: *mut u8) -> Uint {
    let sys: &[u8] = b"C:\\WINDOWS\\SYSTEM32\0";
    let len = sys.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Uint;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = sys[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Uint
}

pub fn get_system_directory_w(buf_len: Uint, buf: *mut Wchar) -> Uint {
    let sys_w: &[u16] = &[
        0x0043, 0x003A, 0x005C, 0x0057, 0x0049, 0x004E, 0x0044, 0x004F,
        0x0057, 0x0053, 0x005C, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045,
        0x004D, 0x0033, 0x0032, 0,
    ];
    let len = sys_w.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Uint;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = sys_w[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Uint
}

// ========================================================================
// GetWindowsDirectoryA / GetWindowsDirectoryW
// ========================================================================

pub fn get_windows_directory_a(buf_len: Uint, buf: *mut u8) -> Uint {
    let win: &[u8] = b"C:\\WINDOWS\0";
    let len = win.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Uint;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = win[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Uint
}

pub fn get_windows_directory_w(buf_len: Uint, buf: *mut Wchar) -> Uint {
    let win_w: &[u16] = &[
        0x0043, 0x003A, 0x005C, 0x0057, 0x0049, 0x004E, 0x0044, 0x004F,
        0x0057, 0x0053, 0,
    ];
    let len = win_w.len() - 1;
    if len + 1 > buf_len as usize {
        return (len + 1) as Uint;
    }
    for i in 0..len {
        unsafe { *buf.add(i) = win_w[i]; }
    }
    unsafe { *buf.add(len) = 0; }
    len as Uint
}

// ========================================================================
// GetLongPathNameA / GetLongPathNameW
// ========================================================================

pub fn get_long_path_name_a(
    short: *const u8,
    long_buf: *mut u8,
    buf_len: Dword,
) -> Dword {
    if short.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let mut nlen: usize = 0;
    unsafe { while *short.add(nlen) != 0 { nlen += 1; } }
    if nlen + 1 > buf_len as usize {
        return (nlen + 1) as Dword;
    }
    strcpy_a(long_buf, short);
    nlen as Dword
}

pub fn get_long_path_name_w(
    short: *const Wchar,
    long_buf: *mut Wchar,
    buf_len: Dword,
) -> Dword {
    if short.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let nlen = lstrlen_w(short);
    if nlen + 1 > buf_len as usize {
        return (nlen + 1) as Dword;
    }
    lstrcpy_w(long_buf, short);
    nlen as Dword
}

// ========================================================================
// GetShortPathNameA / GetShortPathNameW
// ========================================================================

pub fn get_short_path_name_a(
    long_path: *const u8,
    short_path: *mut u8,
    short_len: Dword,
) -> Dword {
    get_long_path_name_a(long_path, short_path, short_len)
}

pub fn get_short_path_name_w(
    long_path: *const Wchar,
    short_path: *mut Wchar,
    short_len: Dword,
) -> Dword {
    get_long_path_name_w(long_path, short_path, short_len)
}

// ========================================================================
// GetDiskFreeSpaceA / GetDiskFreeSpaceW / GetDiskFreeSpaceExA / GetDiskFreeSpaceExW
// ========================================================================

pub fn get_disk_free_space_a(
    _path: *const u8,
    sectors_per_cluster: *mut Dword,
    bytes_per_sector: *mut Dword,
    free_clusters: *mut Dword,
    total_clusters: *mut Dword,
) -> Bool {
    unsafe {
        if !sectors_per_cluster.is_null() { *sectors_per_cluster = 8; }
        if !bytes_per_sector.is_null() { *bytes_per_sector = 512; }
        if !free_clusters.is_null() { *free_clusters = 0x100000; }
        if !total_clusters.is_null() { *total_clusters = 0x200000; }
    }
    TRUE
}

pub fn get_disk_free_space_w(
    _path: *const Wchar,
    sectors_per_cluster: *mut Dword,
    bytes_per_sector: *mut Dword,
    free_clusters: *mut Dword,
    total_clusters: *mut Dword,
) -> Bool {
    unsafe {
        if !sectors_per_cluster.is_null() { *sectors_per_cluster = 8; }
        if !bytes_per_sector.is_null() { *bytes_per_sector = 512; }
        if !free_clusters.is_null() { *free_clusters = 0x100000; }
        if !total_clusters.is_null() { *total_clusters = 0x200000; }
    }
    TRUE
}

pub fn get_disk_free_space_ex_a(
    _path: *const u8,
    free_bytes: *mut u64,
    total_bytes: *mut u64,
    total_free: *mut u64,
) -> Bool {
    unsafe {
        if !free_bytes.is_null() { *free_bytes = 0x100000000; }
        if !total_bytes.is_null() { *total_bytes = 0x200000000; }
        if !total_free.is_null() { *total_free = 0x100000000; }
    }
    TRUE
}

pub fn get_disk_free_space_ex_w(
    _path: *const Wchar,
    free_bytes: *mut u64,
    total_bytes: *mut u64,
    total_free: *mut u64,
) -> Bool {
    unsafe {
        if !free_bytes.is_null() { *free_bytes = 0x100000000; }
        if !total_bytes.is_null() { *total_bytes = 0x200000000; }
        if !total_free.is_null() { *total_free = 0x100000000; }
    }
    TRUE
}

// ========================================================================
// GetVolumeInformationA / GetVolumeInformationW
// ========================================================================

pub fn get_volume_information_a(
    root: *const u8,
    vol_name: *mut u8,
    vol_name_len: Dword,
    vol_serial: *mut Dword,
    max_component: *mut Dword,
    fs_flags: *mut Dword,
    fs_name: *mut u8,
    fs_name_len: Dword,
) -> Bool {
    if root.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let vol = b"VOLUME\0";
    let fs = b"FAT32\0";
    unsafe {
        if !vol_name.is_null() {
            for i in 0..(vol.len().min(vol_name_len as usize)) {
                *vol_name.add(i) = vol[i];
            }
        }
        if !vol_serial.is_null() { *vol_serial = 0x12345678; }
        if !max_component.is_null() { *max_component = 255; }
        if !fs_flags.is_null() { *fs_flags = 0x000700FF; }
        if !fs_name.is_null() {
            for i in 0..(fs.len().min(fs_name_len as usize)) {
                *fs_name.add(i) = fs[i];
            }
        }
    }
    TRUE
}

pub fn get_volume_information_w(
    root: *const Wchar,
    vol_name: *mut Wchar,
    vol_name_len: Dword,
    vol_serial: *mut Dword,
    max_component: *mut Dword,
    fs_flags: *mut Dword,
    fs_name: *mut Wchar,
    fs_name_len: Dword,
) -> Bool {
    if root.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let vol_w: &[Wchar] = &[
        b'V' as Wchar, b'O' as Wchar, b'L' as Wchar,
        b'U' as Wchar, b'M' as Wchar, b'E' as Wchar, 0,
    ];
    let fs_w: &[Wchar] = &[
        b'F' as Wchar, b'A' as Wchar, b'T' as Wchar,
        b'3' as Wchar, b'2' as Wchar, 0,
    ];
    unsafe {
        if !vol_name.is_null() {
            for i in 0..(vol_w.len().min(vol_name_len as usize)) {
                *vol_name.add(i) = vol_w[i];
            }
        }
        if !vol_serial.is_null() { *vol_serial = 0x12345678; }
        if !max_component.is_null() { *max_component = 255; }
        if !fs_flags.is_null() { *fs_flags = 0x000700FF; }
        if !fs_name.is_null() {
            for i in 0..(fs_w.len().min(fs_name_len as usize)) {
                *fs_name.add(i) = fs_w[i];
            }
        }
    }
    TRUE
}

// ========================================================================
// GetDriveTypeA / GetDriveTypeW
// ========================================================================

pub fn get_drive_type_a(root: *const u8) -> Uint {
    if root.is_null() { return DRIVE_UNKNOWN; }
    let c = unsafe { *root.add(0) };
    if c >= b'A' && c <= b'Z' || c >= b'a' && c <= b'z' {
        DRIVE_FIXED
    } else {
        DRIVE_UNKNOWN
    }
}

pub fn get_drive_type_w(root: *const Wchar) -> Uint {
    if root.is_null() { return DRIVE_UNKNOWN; }
    let c = unsafe { *root.add(0) };
    if c >= b'A' as Wchar && c <= b'Z' as Wchar || c >= b'a' as Wchar && c <= b'z' as Wchar {
        DRIVE_FIXED
    } else {
        DRIVE_UNKNOWN
    }
}

// ========================================================================
// GetLogicalDrives
// ========================================================================

pub fn get_logical_drives() -> Dword {
    0x00000004
}

// ========================================================================
// GetLogicalDriveStringsA / GetLogicalDriveStringsW
// ========================================================================

pub fn get_logical_drive_strings_a(len: Uint, buf: *mut u8) -> Uint {
    if buf.is_null() || len < 5 {
        return 5;
    }
    unsafe {
        *buf.add(0) = b'C';
        *buf.add(1) = b':';
        *buf.add(2) = b'\\';
        *buf.add(3) = 0;
        *buf.add(4) = 0;
    }
    4
}

pub fn get_logical_drive_strings_w(len: Uint, buf: *mut Wchar) -> Uint {
    if buf.is_null() || len < 4 {
        return 4;
    }
    unsafe {
        *buf.add(0) = b'C' as Wchar;
        *buf.add(1) = b':' as Wchar;
        *buf.add(2) = b'\\' as Wchar;
        *buf.add(3) = 0;
    }
    4
}
