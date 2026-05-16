// SPDX-License-Identifier: GPL-3.0-only

// Win32 API: version functions
// Based on Microsoft Win32 public specification (clean-room implementation).

use core::sync::atomic::{AtomicU32, Ordering};

type BOOL = i32;
type DWORD = u32;
type ULONG = u32;
type WORD = u16;
type BYTE = u8;
type DWORDLONG = u64;
type LPCSTR = *const u8;
type LPCWSTR = *const u16;
type LPVOID = *mut core::ffi::c_void;

const TRUE: BOOL = 1;
const FALSE: BOOL = 0;
const VER_NT_WORKSTATION: u8 = 1;
const VER_SUITE_SINGLEUSERTS: WORD = 0x0100;

// Condition mask bit positions (3 bits per type)
const VER_EQUAL: u8 = 1;
const VER_GREATER: u8 = 2;
const VER_GREATER_EQUAL: u8 = 3;
const VER_LESS: u8 = 4;
const VER_LESS_EQUAL: u8 = 5;
const VER_AND: u8 = 6;
const VER_OR: u8 = 7;

const VER_PRODUCT_TYPE: DWORD = 0x0000_0080;
const VER_SUITENAME: DWORD = 0x0000_0040;
const VER_PLATFORMID: DWORD = 0x0000_0008;
const VER_BUILDNUMBER: DWORD = 0x0000_0004;
const VER_MAJORVERSION: DWORD = 0x0000_0002;
const VER_MINORVERSION: DWORD = 0x0000_0001;
const VER_SERVICEPACKMAJOR: DWORD = 0x0000_0020;
const VER_SERVICEPACKMINOR: DWORD = 0x0000_0010;

const ERROR_BAD_ARGUMENTS: DWORD = 160;
const ERROR_OLD_WIN_VERSION: DWORD = 1150;

// Simulated Windows 10 version
const HOST_MAJOR: DWORD = 10;
const HOST_MINOR: DWORD = 0;
const HOST_BUILD: DWORD = 19041;
const HOST_PLATFORM: DWORD = 2; // VER_PLATFORM_WIN32_NT
const HOST_SPMAJOR: WORD = 0;
const HOST_SPMINOR: WORD = 0;
const HOST_SUITEMASK: WORD = VER_SUITE_SINGLEUSERTS;
const HOST_PRODUCTTYPE: BYTE = VER_NT_WORKSTATION;

static LAST_ERROR: AtomicU32 = AtomicU32::new(0);

fn set_last_error(code: DWORD) {
    LAST_ERROR.store(code, Ordering::SeqCst);
}

fn get_last_error() -> DWORD {
    LAST_ERROR.load(Ordering::SeqCst)
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OSVERSIONINFOA {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [u8; 128],
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OSVERSIONINFOW {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [u16; 128],
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OSVERSIONINFOEXA {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [u8; 128],
    pub wServicePackMajor: WORD,
    pub wServicePackMinor: WORD,
    pub wSuiteMask: WORD,
    pub wProductType: BYTE,
    pub wReserved: BYTE,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct OSVERSIONINFOEXW {
    pub dwOSVersionInfoSize: DWORD,
    pub dwMajorVersion: DWORD,
    pub dwMinorVersion: DWORD,
    pub dwBuildNumber: DWORD,
    pub dwPlatformId: DWORD,
    pub szCSDVersion: [u16; 128],
    pub wServicePackMajor: WORD,
    pub wServicePackMinor: WORD,
    pub wSuiteMask: WORD,
    pub wProductType: BYTE,
    pub wReserved: BYTE,
}

fn verify_compare_values(left: ULONG, right: ULONG, condition: u8) -> bool {
    match condition {
        VER_EQUAL => left == right,
        VER_GREATER => left > right,
        VER_GREATER_EQUAL => left >= right,
        VER_LESS => left < right,
        VER_LESS_EQUAL => left <= right,
        _ => false,
    }
}

fn verify_update_condition(last_condition: &mut u8, condition: u8) -> u8 {
    match *last_condition {
        0 => *last_condition = condition,
        VER_EQUAL => {
            if condition >= VER_EQUAL && condition <= VER_LESS_EQUAL {
                *last_condition = condition;
                return condition;
            }
        }
        VER_GREATER | VER_GREATER_EQUAL => {
            if condition >= VER_EQUAL && condition <= VER_GREATER_EQUAL {
                return condition;
            }
        }
        VER_LESS | VER_LESS_EQUAL => {
            if condition == VER_EQUAL || (condition >= VER_LESS && condition <= VER_LESS_EQUAL) {
                return condition;
            }
        }
        _ => {}
    }
    if condition == 0 {
        *last_condition |= 0x10;
    }
    *last_condition & 0x0f
}

fn condition_from_mask(mask: DWORDLONG, shift: u32) -> u8 {
    ((mask >> (shift * 3)) & 0x07) as u8
}

/// Win32: GetVersion (KERNEL32.@)
/// Returns: MajorVersion & 0xFF, MinorVersion & 0xFF in low word.
#[no_mangle]
pub extern "C" fn GetVersion() -> DWORD {
    let major = (HOST_MAJOR & 0xFF) as DWORD;
    let minor = ((HOST_MINOR & 0xFF) as DWORD) << 8;
    major | minor
}

/// Win32: GetVersionExA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetVersionExA(info: *mut OSVERSIONINFOA) -> BOOL {
    if info.is_null() { return FALSE; }
    unsafe {
        let size = (*info).dwOSVersionInfoSize;
        if size == 0 { return FALSE; }
        (*info).dwMajorVersion = HOST_MAJOR;
        (*info).dwMinorVersion = HOST_MINOR;
        (*info).dwBuildNumber = HOST_BUILD;
        (*info).dwPlatformId = HOST_PLATFORM;
        (*info).szCSDVersion[0] = 0;
    }
    TRUE
}

/// Win32: GetVersionExW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetVersionExW(info: *mut OSVERSIONINFOW) -> BOOL {
    if info.is_null() { return FALSE; }
    unsafe {
        let size = (*info).dwOSVersionInfoSize;
        if size == 0 { return FALSE; }
        (*info).dwMajorVersion = HOST_MAJOR;
        (*info).dwMinorVersion = HOST_MINOR;
        (*info).dwBuildNumber = HOST_BUILD;
        (*info).dwPlatformId = HOST_PLATFORM;
        (*info).szCSDVersion[0] = 0;
    }
    TRUE
}

/// Win32: VerifyVersionInfoA (KERNEL32.@)
#[no_mangle]
pub extern "C" fn VerifyVersionInfoA(
    info_a: *const OSVERSIONINFOEXA,
    type_mask: DWORD,
    condition_mask: DWORDLONG,
) -> BOOL {
    if info_a.is_null() { return FALSE; }
    let info_w = OSVERSIONINFOEXW {
        dwOSVersionInfoSize: core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD,
        dwMajorVersion: unsafe { (*info_a).dwMajorVersion },
        dwMinorVersion: unsafe { (*info_a).dwMinorVersion },
        dwBuildNumber: unsafe { (*info_a).dwBuildNumber },
        dwPlatformId: unsafe { (*info_a).dwPlatformId },
        szCSDVersion: [0; 128],
        wServicePackMajor: unsafe { (*info_a).wServicePackMajor },
        wServicePackMinor: unsafe { (*info_a).wServicePackMinor },
        wSuiteMask: unsafe { (*info_a).wSuiteMask },
        wProductType: unsafe { (*info_a).wProductType },
        wReserved: unsafe { (*info_a).wReserved },
    };
    VerifyVersionInfoW(&info_w, type_mask, condition_mask)
}

/// Win32: VerifyVersionInfoW (KERNEL32.@)
#[no_mangle]
pub extern "C" fn VerifyVersionInfoW(
    info: *const OSVERSIONINFOEXW,
    type_mask: DWORD,
    condition_mask: DWORDLONG,
) -> BOOL {
    if info.is_null() || type_mask == 0 || condition_mask == 0 {
        set_last_error(ERROR_BAD_ARGUMENTS);
        return FALSE;
    }

    let ver = OSVERSIONINFOEXW {
        dwOSVersionInfoSize: core::mem::size_of::<OSVERSIONINFOEXW>() as DWORD,
        dwMajorVersion: HOST_MAJOR,
        dwMinorVersion: HOST_MINOR,
        dwBuildNumber: HOST_BUILD,
        dwPlatformId: HOST_PLATFORM,
        szCSDVersion: [0; 128],
        wServicePackMajor: HOST_SPMAJOR,
        wServicePackMinor: HOST_SPMINOR,
        wSuiteMask: HOST_SUITEMASK,
        wProductType: HOST_PRODUCTTYPE,
        wReserved: 0,
    };

    unsafe {
        if type_mask & VER_PRODUCT_TYPE != 0 {
            let cond = condition_from_mask(condition_mask, 7);
            if !verify_compare_values(ver.wProductType as ULONG, (*info).wProductType as ULONG, cond) {
                set_last_error(ERROR_OLD_WIN_VERSION);
                return FALSE;
            }
        }

        if type_mask & VER_SUITENAME != 0 {
            match condition_from_mask(condition_mask, 6) {
                VER_AND => {
                    if ((*info).wSuiteMask & ver.wSuiteMask) != (*info).wSuiteMask {
                        set_last_error(ERROR_OLD_WIN_VERSION);
                        return FALSE;
                    }
                }
                VER_OR => {
                    if ((*info).wSuiteMask & ver.wSuiteMask) == 0 && (*info).wSuiteMask != 0 {
                        set_last_error(ERROR_OLD_WIN_VERSION);
                        return FALSE;
                    }
                }
                _ => {
                    set_last_error(ERROR_BAD_ARGUMENTS);
                    return FALSE;
                }
            }
        }

        if type_mask & VER_PLATFORMID != 0 {
            let cond = condition_from_mask(condition_mask, 3);
            if !verify_compare_values(ver.dwPlatformId, (*info).dwPlatformId, cond) {
                set_last_error(ERROR_OLD_WIN_VERSION);
                return FALSE;
            }
        }

        if type_mask & VER_BUILDNUMBER != 0 {
            let cond = condition_from_mask(condition_mask, 2);
            if !verify_compare_values(ver.dwBuildNumber, (*info).dwBuildNumber, cond) {
                set_last_error(ERROR_OLD_WIN_VERSION);
                return FALSE;
            }
        }

        let composite_mask = VER_MAJORVERSION | VER_MINORVERSION
            | VER_SERVICEPACKMAJOR | VER_SERVICEPACKMINOR;
        if type_mask & composite_mask != 0 {
            let mut last_condition: u8 = 0;
            let mut succeeded = true;
            let mut do_next = true;

            if type_mask & VER_MAJORVERSION != 0 {
                let cond = verify_update_condition(&mut last_condition, condition_from_mask(condition_mask, 1));
                succeeded = verify_compare_values(ver.dwMajorVersion, (*info).dwMajorVersion, cond);
                do_next = ver.dwMajorVersion == (*info).dwMajorVersion
                    && cond >= VER_EQUAL && cond <= VER_LESS_EQUAL;
            }
            if type_mask & VER_MINORVERSION != 0 && do_next {
                let cond = verify_update_condition(&mut last_condition, condition_from_mask(condition_mask, 0));
                succeeded = verify_compare_values(ver.dwMinorVersion, (*info).dwMinorVersion, cond);
                do_next = ver.dwMinorVersion == (*info).dwMinorVersion
                    && cond >= VER_EQUAL && cond <= VER_LESS_EQUAL;
            }
            if type_mask & VER_SERVICEPACKMAJOR != 0 && do_next {
                let cond = verify_update_condition(&mut last_condition, condition_from_mask(condition_mask, 5));
                succeeded = verify_compare_values(ver.wServicePackMajor as ULONG, (*info).wServicePackMajor as ULONG, cond);
                do_next = ver.wServicePackMajor == (*info).wServicePackMajor
                    && cond >= VER_EQUAL && cond <= VER_LESS_EQUAL;
            }
            if type_mask & VER_SERVICEPACKMINOR != 0 && do_next {
                let cond = verify_update_condition(&mut last_condition, condition_from_mask(condition_mask, 4));
                succeeded = verify_compare_values(ver.wServicePackMinor as ULONG, (*info).wServicePackMinor as ULONG, cond);
            }

            if !succeeded {
                set_last_error(ERROR_OLD_WIN_VERSION);
                return FALSE;
            }
        }
    }

    TRUE
}

/// Win32: SetLastError (KERNEL32.@)
#[no_mangle]
pub extern "C" fn SetLastError(err_code: DWORD) {
    LAST_ERROR.store(err_code, Ordering::SeqCst);
}

/// Win32: GetLastError (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetLastError() -> DWORD {
    LAST_ERROR.load(Ordering::SeqCst)
}

/// Win32: SetLastErrorEx (KERNEL32.@)
#[no_mangle]
pub extern "C" fn SetLastErrorEx(err_code: DWORD, _typ: DWORD) {
    LAST_ERROR.store(err_code, Ordering::SeqCst);
}
