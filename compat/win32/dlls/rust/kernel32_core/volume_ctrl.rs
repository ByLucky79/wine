// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 volume management API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/volume_ctrl.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 volume and DOS device API: GetVolumeInformationA/W,
//   GetVolumeNameForVolumeMountPointW, GetVolumePathNamesForVolumeNameW,
//   FindFirstVolumeW, FindNextVolumeW, FindVolumeClose,
//   FindFirstVolumeMountPointA/W, FindNextVolumeMountPointA/W,
//   FindVolumeMountPointClose, SetVolumeLabelA/W,
//   GetVolumeInformationByHandleW, DefineDosDeviceW, QueryDosDeviceW.
//   Volume enumeration uses an internal mount point registry.
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
use core::cell::UnsafeCell;

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Wchar = u16;
type Handle = *mut core::ffi::c_void;
type Lpvoid = *mut core::ffi::c_void;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const MAX_PATH: usize = 260;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_FILE_NOT_FOUND: Dword = 2;
const ERROR_PATH_NOT_FOUND: Dword = 3;
const ERROR_ACCESS_DENIED: Dword = 5;
const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_INVALID_NAME: Dword = 123;
const ERROR_FILENAME_EXCED_RANGE: Dword = 206;
const ERROR_MORE_DATA: Dword = 234;
const ERROR_NO_MORE_FILES: Dword = 18;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;
const ERROR_NOT_READY: Dword = 21;

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

fn wcscat_w(dst: *mut Wchar, src: *const Wchar) {
    let dlen = lstrlen_w(dst);
    unsafe { lstrcpy_w(dst.add(dlen), src); }
}

fn wcscpy_w(dst: *mut Wchar, src: *const Wchar) {
    lstrcpy_w(dst, src);
}

fn wcsicmp(a: *const Wchar, b: *const Wchar) -> i32 {
    unsafe {
        let mut i = 0;
        loop {
            let ca = *a.add(i);
            let cb = *b.add(i);
            if ca == 0 && cb == 0 { return 0; }
            let ua = if ca >= b'A' as Wchar && ca <= b'Z' as Wchar { ca + 0x20 } else { ca };
            let ub = if cb >= b'A' as Wchar && cb <= b'Z' as Wchar { cb + 0x20 } else { cb };
            if ua != ub { return if ua < ub { -1 } else { 1 }; }
            i += 1;
        }
    }
}

// ========================================================================
// Volume registry
// ========================================================================

struct VolumeEntry {
    guid_path: Vec<Wchar>,
    dos_path: Vec<Wchar>,
    label: Vec<Wchar>,
    serial: Dword,
    fs_name: Vec<Wchar>,
    fs_flags: Dword,
    drive_type: Uint,
}

struct VolumeState {
    volumes: UnsafeCell<Vec<VolumeEntry>>,
}

unsafe impl Sync for VolumeState {}

static VOL_STATE: VolumeState = VolumeState {
    volumes: UnsafeCell::new(Vec::new()),
};

fn str_to_wchars(s: &str) -> Vec<Wchar> {
    s.encode_utf16().chain(core::iter::once(0)).collect()
}

fn init_volumes() {
    let cell = unsafe { &mut *VOL_STATE.volumes.get() };
    if !cell.is_empty() {
        return;
    }
    cell.push(VolumeEntry {
        guid_path: str_to_wchars("\\\\?\\Volume{00000000-0000-0000-0000-000000000001}\\"),
        dos_path: str_to_wchars("C:\\"),
        label: str_to_wchars("VOLUME"),
        serial: 0x12345678,
        fs_name: str_to_wchars("FAT32"),
        fs_flags: 0x000700FF,
        drive_type: DRIVE_FIXED,
    });
}

fn with_volumes<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<VolumeEntry>) -> R,
{
    init_volumes();
    let cell = unsafe { &mut *VOL_STATE.volumes.get() };
    f(cell)
}

fn wchars_eq(a_slice: &[Wchar], b_slice: &[Wchar]) -> bool {
    let a_trim = if a_slice.last() == Some(&0) { &a_slice[..a_slice.len() - 1] } else { a_slice };
    let b_trim = if b_slice.last() == Some(&0) { &b_slice[..b_slice.len() - 1] } else { b_slice };
    if a_trim.len() != b_trim.len() {
        return false;
    }
    for (&ca, &cb) in a_trim.iter().zip(b_trim.iter()) {
        let ua = if ca >= 0x0041 && ca <= 0x005A { ca + 0x0020 } else { ca };
        let ub = if cb >= 0x0041 && cb <= 0x005B { cb + 0x0020 } else { cb };
        if ua != ub { return false; }
    }
    true
}

fn find_volume_by_path(path: &[Wchar]) -> Option<usize> {
    with_volumes(|vols| {
        for (i, vol) in vols.iter().enumerate() {
            if wchars_eq(&vol.guid_path, path) || wchars_eq(&vol.dos_path, path) {
                return Some(i);
            }
        }
        None
    })
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
    unsafe {
        if !vol_name.is_null() {
            let label = b"VOLUME\0";
            for i in 0..(label.len().min(vol_name_len as usize)) {
                *vol_name.add(i) = label[i];
            }
        }
        if !vol_serial.is_null() { *vol_serial = 0x12345678; }
        if !max_component.is_null() { *max_component = 255; }
        if !fs_flags.is_null() { *fs_flags = 0x000700FF; }
        if !fs_name.is_null() {
            let fs = b"FAT32\0";
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
    with_volumes(|vols| {
        let mut found = false;
        let root_slice = if !root.is_null() {
            let len = lstrlen_w(root);
            Some(unsafe { core::slice::from_raw_parts(root, len) })
        } else { None };

        if let Some(rs) = root_slice {
            for vol in vols.iter() {
                let d = &vol.dos_path;
                let d_trim = if d.last() == Some(&0) { &d[..d.len() - 1] } else { d.as_slice() };
                if wcsicmp(d_trim.as_ptr(), root_slice.unwrap().as_ptr()) == 0 {
                    if !vol_name.is_null() {
                        let label = &vol.label;
                        for i in 0..(label.len().min(vol_name_len as usize)) {
                            unsafe { *vol_name.add(i) = label[i]; }
                        }
                    }
                    if !vol_serial.is_null() { unsafe { *vol_serial = vol.serial; } }
                    if !max_component.is_null() { unsafe { *max_component = 255; } }
                    if !fs_flags.is_null() { unsafe { *fs_flags = vol.fs_flags; } }
                    if !fs_name.is_null() {
                        let f = &vol.fs_name;
                        for i in 0..(f.len().min(fs_name_len as usize)) {
                            unsafe { *fs_name.add(i) = f[i]; }
                        }
                    }
                    found = true;
                    break;
                }
            }
        }
        if !found {
            if !vol_name.is_null() && vol_name_len > 0 {
                unsafe { *vol_name = 0; }
            }
            if !fs_name.is_null() && fs_name_len > 0 {
                let fs = [b'F' as Wchar, b'A' as Wchar, b'T' as Wchar, b'3' as Wchar, b'2' as Wchar, 0];
                for i in 0..(fs.len().min(fs_name_len as usize)) {
                    unsafe { *fs_name.add(i) = fs[i]; }
                }
            }
            if !vol_serial.is_null() { unsafe { *vol_serial = 0; } }
            if !max_component.is_null() { unsafe { *max_component = 255; } }
            if !fs_flags.is_null() { unsafe { *fs_flags = 0x000700FF; } }
        }
    });
    TRUE
}

// ========================================================================
// GetVolumeNameForVolumeMountPointW
// ========================================================================

pub fn get_volume_name_for_volume_mount_point_w(
    path: *const Wchar,
    volume: *mut Wchar,
    size: Dword,
) -> Bool {
    if path.is_null() || volume.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    with_volumes(|vols| {
        let path_slice = {
            let len = lstrlen_w(path);
            unsafe { core::slice::from_raw_parts(path, len) }
        };
        for vol in vols.iter() {
            let d = &vol.dos_path;
            let d_trim = if d.last() == Some(&0) { &d[..d.len() - 1] } else { d.as_slice() };
            if wcsicmp(d_trim.as_ptr(), path_slice.as_ptr()) == 0 {
                let guid = &vol.guid_path;
                for i in 0..(guid.len().min(size as usize)) {
                    unsafe { *volume.add(i) = guid[i]; }
                }
                return TRUE;
            }
        }
        set_last_error(ERROR_FILE_NOT_FOUND);
        FALSE
    })
}

// ========================================================================
// GetVolumePathNamesForVolumeNameW
// ========================================================================

pub fn get_volume_path_names_for_volume_name_w(
    volume_name: *const Wchar,
    path_names: *mut Wchar,
    buf_len: Dword,
    return_len: *mut Dword,
) -> Bool {
    if volume_name.is_null() || path_names.is_null() || return_len.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    with_volumes(|vols| {
        let vn_slice = {
            let len = lstrlen_w(volume_name);
            unsafe { core::slice::from_raw_parts(volume_name, len) }
        };
        for vol in vols.iter() {
            let g = &vol.guid_path;
            let g_trim = if g.last() == Some(&0) { &g[..g.len() - 1] } else { g.as_slice() };
            if wcsicmp(g_trim.as_ptr(), vn_slice.as_ptr()) == 0 {
                let needed = (vol.dos_path.len()) as Dword;
                unsafe { *return_len = needed; }
                if needed <= buf_len {
                    for i in 0..vol.dos_path.len() {
                        unsafe { *path_names.add(i) = vol.dos_path[i]; }
                    }
                    return TRUE;
                }
                set_last_error(ERROR_MORE_DATA);
                return FALSE;
            }
        }
        set_last_error(ERROR_FILE_NOT_FOUND);
        FALSE
    })
}

// ========================================================================
// FindFirstVolumeW / FindNextVolumeW / FindVolumeClose
// ========================================================================

struct FindVolumeState {
    index: usize,
}

struct FindVolState {
    inner: UnsafeCell<BTreeMap<Handle, FindVolumeState>>,
}

unsafe impl Sync for FindVolState {}

static FIND_VOL_STATE: FindVolState = FindVolState {
    inner: UnsafeCell::new(BTreeMap::new()),
};

fn alloc_find_handle() -> Handle {
    static NEXT: AtomicU32 = AtomicU32::new(0x3000);
    NEXT.fetch_add(1, Ordering::Relaxed) as usize as Handle
}

pub fn find_first_volume_w(volume: *mut Wchar, len: Dword) -> Handle {
    if volume.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE as usize as Handle;
    }
    with_volumes(|vols| {
        if vols.is_empty() {
            set_last_error(ERROR_FILE_NOT_FOUND);
            return INVALID_HANDLE_VALUE as usize as Handle;
        }
        let guid = &vols[0].guid_path;
        if len < guid.len() as Dword {
            set_last_error(ERROR_MORE_DATA);
            return INVALID_HANDLE_VALUE as usize as Handle;
        }
        for i in 0..guid.len() {
            unsafe { *volume.add(i) = guid[i]; }
        }
        let h = alloc_find_handle();
        let cell = unsafe { &mut *FIND_VOL_STATE.inner.get() };
        cell.insert(h, FindVolumeState { index: 0 });
        h
    })
}

pub fn find_next_volume_w(handle: Handle, volume: *mut Wchar, len: Dword) -> Bool {
    if volume.is_null() || handle.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    with_volumes(|vols| {
        let cell = unsafe { &mut *FIND_VOL_STATE.inner.get() };
        if let Some(state) = cell.get_mut(&handle) {
            let next_idx = state.index + 1;
            if next_idx >= vols.len() {
                set_last_error(ERROR_NO_MORE_FILES);
                return FALSE;
            }
            let guid = &vols[next_idx].guid_path;
            if len < guid.len() as Dword {
                set_last_error(ERROR_MORE_DATA);
                return FALSE;
            }
            for i in 0..guid.len() {
                unsafe { *volume.add(i) = guid[i]; }
            }
            state.index = next_idx;
            TRUE
        } else {
            set_last_error(ERROR_INVALID_PARAMETER);
            FALSE
        }
    })
}

pub fn find_volume_close(handle: Handle) -> Bool {
    let cell = unsafe { &mut *FIND_VOL_STATE.inner.get() };
    cell.remove(&handle).is_some() as Bool
}

// ========================================================================
// FindFirstVolumeMountPointA / FindFirstVolumeMountPointW
// ========================================================================

pub fn find_first_volume_mount_point_a(
    _root: *const u8,
    _mount_point: *mut u8,
    _len: Dword,
) -> Handle {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    INVALID_HANDLE_VALUE as usize as Handle
}

pub fn find_first_volume_mount_point_w(
    _root: *const Wchar,
    _mount_point: *mut Wchar,
    _len: Dword,
) -> Handle {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    INVALID_HANDLE_VALUE as usize as Handle
}

// ========================================================================
// FindNextVolumeMountPointA / FindNextVolumeMountPointW
// ========================================================================

pub fn find_next_volume_mount_point_a(
    _handle: Handle,
    _mount_point: *mut u8,
    _len: Dword,
) -> Bool {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

pub fn find_next_volume_mount_point_w(
    _handle: Handle,
    _mount_point: *mut Wchar,
    _len: Dword,
) -> Bool {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

// ========================================================================
// FindVolumeMountPointClose
// ========================================================================

pub fn find_volume_mount_point_close(_handle: Handle) -> Bool {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

// ========================================================================
// SetVolumeLabelA / SetVolumeLabelW
// ========================================================================

pub fn set_volume_label_w(_root: *const Wchar, label: *const Wchar) -> Bool {
    if label.is_null() {
        return TRUE;
    }
    let label_slice = {
        let len = lstrlen_w(label);
        unsafe { core::slice::from_raw_parts(label, len) }
    };
    with_volumes(|vols| {
        if let Some(vol) = vols.get_mut(0) {
            vol.label = label_slice.to_vec();
            vol.label.push(0);
        }
    });
    TRUE
}

pub fn set_volume_label_a(root: *const u8, label: *const u8) -> Bool {
    let mut wide: Vec<Wchar> = Vec::new();
    if !label.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *label.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
            wide.push(0);
        }
    }
    let mut root_wide: Vec<Wchar> = Vec::new();
    if !root.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *root.add(i);
                if c == 0 { break; }
                root_wide.push(c as Wchar);
                i += 1;
            }
            root_wide.push(0);
        }
    }
    set_volume_label_w(if root.is_null() { ptr::null() } else { root_wide.as_ptr() },
                       if label.is_null() { ptr::null() } else { wide.as_ptr() })
}

// ========================================================================
// GetVolumeInformationByHandleW
// ========================================================================

pub fn get_volume_information_by_handle_w(
    _handle: Handle,
    vol_name: *mut Wchar,
    vol_name_len: Dword,
    vol_serial: *mut Dword,
    max_component: *mut Dword,
    fs_flags: *mut Dword,
    fs_name: *mut Wchar,
    fs_name_len: Dword,
) -> Bool {
    unsafe {
        if !vol_name.is_null() && vol_name_len > 0 {
            let label: &[Wchar] = &[
                b'V' as Wchar, b'O' as Wchar, b'L' as Wchar,
                b'U' as Wchar, b'M' as Wchar, b'E' as Wchar, 0,
            ];
            for i in 0..(label.len().min(vol_name_len as usize)) {
                *vol_name.add(i) = label[i];
            }
        }
        if !vol_serial.is_null() { *vol_serial = 0x12345678; }
        if !max_component.is_null() { *max_component = 255; }
        if !fs_flags.is_null() { *fs_flags = 0x000700FF; }
        if !fs_name.is_null() && fs_name_len > 0 {
            let fs: &[Wchar] = &[
                b'F' as Wchar, b'A' as Wchar, b'T' as Wchar,
                b'3' as Wchar, b'2' as Wchar, 0,
            ];
            for i in 0..(fs.len().min(fs_name_len as usize)) {
                *fs_name.add(i) = fs[i];
            }
        }
    }
    TRUE
}

// ========================================================================
// DefineDosDeviceW
// ========================================================================

pub fn define_dos_device_w(
    _flags: Dword,
    dev_name: *const Wchar,
    target_path: *const Wchar,
) -> Bool {
    if dev_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let dev_slice = {
        let len = lstrlen_w(dev_name);
        unsafe { core::slice::from_raw_parts(dev_name, len) }
    };
    with_volumes(|vols| {
        let mut path: Vec<Wchar> = Vec::new();
        if !target_path.is_null() {
            let len = lstrlen_w(target_path);
            let sl = unsafe { core::slice::from_raw_parts(target_path, len) };
            path.extend_from_slice(sl);
        }
        path.push(0);

        let mut dev_vec: Vec<Wchar> = Vec::new();
        dev_vec.extend_from_slice(dev_slice);
        dev_vec.push(0);

        vols.push(VolumeEntry {
            guid_path: {
                let s = alloc::format!("\\\\?\\Volume{{{:04X}{:04X}-0000-0000-0000-000000000000}}\\", 0, vols.len() + 1);
                s.encode_utf16().chain(core::iter::once(0)).collect()
            },
            dos_path: dev_vec.clone(),
            label: {
                let s = "DOSDEV";
                s.encode_utf16().chain(core::iter::once(0)).collect()
            },
            serial: 0,
            fs_name: {
                let s = "NTFS";
                s.encode_utf16().chain(core::iter::once(0)).collect()
            },
            fs_flags: 0x000700FF,
            drive_type: DRIVE_REMOTE,
        });
        TRUE
    })
}

// ========================================================================
// QueryDosDeviceW
// ========================================================================

pub fn query_dos_device_w(
    dev_name: *const Wchar,
    target: *mut Wchar,
    buf_size: Dword,
) -> Dword {
    if target.is_null() || buf_size == 0 || dev_name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let dev_slice = {
        let len = lstrlen_w(dev_name);
        unsafe { core::slice::from_raw_parts(dev_name, len) }
    };
    with_volumes(|vols| {
        for vol in vols.iter() {
            let d = &vol.dos_path;
            let d_trim = if d.last() == Some(&0) { &d[..d.len() - 1] } else { d.as_slice() };
            if wcsicmp(d_trim.as_ptr(), dev_slice.as_ptr()) == 0 {
                let nt_path: Vec<Wchar> = {
                    let s = alloc::format!("\\??\\C:\\");
                    s.encode_utf16().chain(core::iter::once(0)).collect()
                };
                let needed = nt_path.len() as Dword;
                if needed > buf_size {
                    set_last_error(ERROR_MORE_DATA);
                    return needed;
                }
                for i in 0..nt_path.len() {
                    unsafe { *target.add(i) = nt_path[i]; }
                }
                return (nt_path.len() - 1) as Dword;
            }
        }
        let global_path: Vec<Wchar> = {
            let s = alloc::format!("\\??\\C:\\");
            s.encode_utf16().chain(core::iter::once(0)).collect()
        };
        let needed = global_path.len() as Dword;
        if needed > buf_size {
            set_last_error(ERROR_MORE_DATA);
            return needed;
        }
        for i in 0..global_path.len() {
            unsafe { *target.add(i) = global_path[i]; }
        }
        (global_path.len() - 1) as Dword
    })
}
