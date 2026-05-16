// SPDX-License-Identifier: GPL-3.0-only

// Win32 API: power management, system time adjustment, kernel object security
// Based on Microsoft Win32 public specification (clean-room implementation).

use core::sync::atomic::{AtomicU32, Ordering};

type BOOL = i32;
type DWORD = u32;
type HANDLE = *mut core::ffi::c_void;
type LPDWORD = *mut DWORD;
type SecurityInformation = u32;
type PsecurityDescriptor = *mut core::ffi::c_void;

const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

const ERROR_CALL_NOT_IMPLEMENTED: DWORD = 120;
const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;
const ERROR_INVALID_PARAMETER: DWORD = 87;

const AC_LINE_UNKNOWN: u8 = 255;
const BATTERY_FLAG_UNKNOWN: u8 = 255;
const BATTERY_PERCENTAGE_UNKNOWN: u8 = 255;
const BATTERY_LIFE_UNKNOWN: u32 = 0xFFFF_FFFF;

static LAST_ERROR: AtomicU32 = AtomicU32::new(0);

fn set_last_error(code: DWORD) {
    LAST_ERROR.store(code, Ordering::SeqCst);
}

fn get_last_error() -> DWORD {
    LAST_ERROR.load(Ordering::SeqCst)
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_POWER_STATUS {
    pub ACLineStatus: u8,
    pub BatteryFlag: u8,
    pub BatteryLifePercent: u8,
    pub SystemStatusFlag: u8,
    pub BatteryLifeTime: u32,
    pub BatteryFullLifeTime: u32,
}

/// Puts the system into a suspended state or hibernation.
/// Win32: SetSystemPowerState (KERNEL32.@)
#[no_mangle]
pub extern "C" fn SetSystemPowerState(
    _suspend_or_hibernate: BOOL,
    _force_flag: BOOL,
) -> BOOL {
    TRUE
}

/// Gets the system power status (AC line, battery info).
/// Win32: GetSystemPowerStatus (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetSystemPowerStatus(
    ps: *mut SYSTEM_POWER_STATUS,
) -> BOOL {
    if ps.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        (*ps).ACLineStatus = AC_LINE_UNKNOWN;
        (*ps).BatteryFlag = BATTERY_FLAG_UNKNOWN;
        (*ps).BatteryLifePercent = BATTERY_PERCENTAGE_UNKNOWN;
        (*ps).SystemStatusFlag = 0;
        (*ps).BatteryLifeTime = BATTERY_LIFE_UNKNOWN;
        (*ps).BatteryFullLifeTime = BATTERY_LIFE_UNKNOWN;
    }
    TRUE
}

/// Suspends the system (higher-level wrapper).
/// Win32: SetSuspendState (KERNEL32.@)
#[no_mangle]
pub extern "C" fn SetSuspendState(
    _hibernate: BOOL,
    _force: BOOL,
    _wakeup_events_disabled: BOOL,
) -> BOOL {
    FALSE
}

/// Retrieves a copy of the security descriptor for a kernel object.
/// Win32: GetKernelObjectSecurity (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetKernelObjectSecurity(
    _handle: HANDLE,
    _requested_information: SecurityInformation,
    _security_descriptor: PsecurityDescriptor,
    _length: DWORD,
    _length_needed: LPDWORD,
) -> BOOL {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/// Sets the security descriptor for a kernel object.
/// Win32: SetKernelObjectSecurity (KERNEL32.@)
#[no_mangle]
pub extern "C" fn SetKernelObjectSecurity(
    _handle: HANDLE,
    _security_information: SecurityInformation,
    _security_descriptor: PsecurityDescriptor,
) -> BOOL {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

/// Gets the system time adjustment values.
/// Win32: GetSystemTimeAdjustment (KERNEL32.@)
#[no_mangle]
pub extern "C" fn GetSystemTimeAdjustment(
    lp_time_adjustment: LPDWORD,
    lp_time_increment: LPDWORD,
    lp_time_adjustment_disabled: LPDWORD,
) -> BOOL {
    if lp_time_adjustment.is_null() || lp_time_increment.is_null() || lp_time_adjustment_disabled.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        *lp_time_adjustment = 0;
        *lp_time_increment = 156_000;
        *lp_time_adjustment_disabled = 1;
    }
    TRUE
}

/// Enables or disables the system time adjustment.
/// Win32: SetSystemTimeAdjustment (KERNEL32.@)
#[no_mangle]
pub extern "C" fn SetSystemTimeAdjustment(
    _dw_time_adjustment: DWORD,
    _b_time_adjustment_disabled: BOOL,
) -> BOOL {
    TRUE
}
