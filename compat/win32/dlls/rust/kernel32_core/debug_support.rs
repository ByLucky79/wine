// SPDX-License-Identifier: GPL-3.0-only

use core::sync::atomic::{AtomicBool, Ordering};

type Bool = i32;
type Dword = u32;
type Handle = u64;
type Lpvoid = *mut u8;
type Lpcstr = *const u8;
type Lpcwstr = *const u16;
type Lpstr = *mut u8;
type Lpwstr = *mut u16;
type Lpdword = *mut Dword;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const DEBUG_PROCESS: Dword = 0x00000001;
const DEBUG_ONLY_THIS_PROCESS: Dword = 0x00000002;

const DBG_CONTINUE: Dword = 0x00010002;

const ERROR_ACCESS_DENIED: Dword = 5;
const ERROR_NOT_SUPPORTED: Dword = 50;

fn set_last_error(_code: Dword) {}

static DEBUGGER_ACTIVE: AtomicBool = AtomicBool::new(false);

/* Public API */

pub fn output_debug_string_a(str_: Lpcstr) {
    if str_.is_null() {
        return;
    }
    /* In kernel context, debug strings are sent via serial.
       If a debugger is attached (DBG_PRINTEXCEPTION_C), it would
       catch the exception. Since we have no SEH here, we just return. */
}

pub fn output_debug_string_w(str_: Lpcwstr) {
    if str_.is_null() {
        return;
    }
}

pub fn debug_break() {
    unsafe {
        core::arch::asm!("int3");
    }
}

pub fn is_debugger_present() -> Bool {
    if DEBUGGER_ACTIVE.load(Ordering::Relaxed) { TRUE } else { FALSE }
}

pub fn check_remote_debugger_present(_handle: Handle, debugger_present: Lpdword) -> Bool {
    if !debugger_present.is_null() {
        unsafe { *debugger_present = 0; }
    }
    TRUE
}

pub fn debug_active_process(process_id: Dword) -> Bool {
    /* Stub: debugger support not implemented */
    set_last_error(ERROR_ACCESS_DENIED);
    FALSE
}

pub fn debug_active_process_stop(_process_id: Dword) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn wait_for_debug_event(_event: *mut u8, _timeout: Dword) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn continue_debug_event(_process_id: Dword, _thread_id: Dword, _continue_status: Dword) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn debug_break_process(_process: Handle) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn debug_set_process_kill_on_exit(_kill: Bool) -> Bool {
    TRUE
}
