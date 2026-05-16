// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Kernel32 DLL entry point and initialization
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/kernel_entry.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements KERNEL32.dll entry point (DllMain) and process initialization
//   routines. Handles DLL_PROCESS_ATTACH/DETACH, thread attach/detach
//   notifications, and one-time global state initialization for the kernel32
//   compatibility layer.
//
// Bağımlı Dosyalar:
//   1-) alloc (crate)
//   2-) core (crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu
// *******************************************************************

use core::ffi::c_void;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

type Handle = *mut c_void;
type Bool = i32;
type Dword = u32;
type Lpvoid = *mut c_void;
type Lpcstr = *const u8;
type Lpwstr = *const u16;
type Hinstance = Handle;
type Lpstr = *mut u8;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const INVALID_HANDLE_VALUE: Handle = !0usize as *mut c_void;

const DLL_PROCESS_ATTACH: Dword = 1;
const DLL_THREAD_ATTACH: Dword = 2;
const DLL_THREAD_DETACH: Dword = 3;
const DLL_PROCESS_DETACH: Dword = 0;

const DLL_PROCESS_VERIFIER: Dword = 4;

const STARTF_USESHOWWINDOW: Dword = 0x00000001;
const STARTF_USESTDHANDLES: Dword = 0x00000100;

static PROCESS_ATTACHED: AtomicBool = AtomicBool::new(false);
static STARTUP_INFO_A: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

fn set_last_error(_code: Dword) {}

fn init_console_handles() {
    let input = INVALID_HANDLE_VALUE;
    let output = INVALID_HANDLE_VALUE;
    let error = INVALID_HANDLE_VALUE;
    crate::console_io::set_std_handle(0xFFFFFFF6, input);
    crate::console_io::set_std_handle(0xFFFFFFF5, output);
    crate::console_io::set_std_handle(0xFFFFFFF4, error);
}

fn init_environment() {
    init_console_handles();
}

pub fn dll_main(
    _hinst_dll: Hinstance,
    reason: Dword,
    _reserved: Lpvoid,
) -> Bool {
    match reason {
        DLL_PROCESS_ATTACH => {
            if PROCESS_ATTACHED.swap(true, Ordering::AcqRel) {
                return TRUE;
            }
            init_environment();
            TRUE
        }
        DLL_PROCESS_DETACH => {
            PROCESS_ATTACHED.store(false, Ordering::Release);
            TRUE
        }
        DLL_THREAD_ATTACH => TRUE,
        DLL_THREAD_DETACH => TRUE,
        _ => TRUE,
    }
}

pub fn get_startup_info_a(startup_info: *mut StartupInfoA) {
    if startup_info.is_null() {
        return;
    }
    unsafe {
        ptr::write(startup_info, StartupInfoA {
            cb: core::mem::size_of::<StartupInfoA>() as Dword,
            lp_reserved: ptr::null_mut(),
            lp_desktop: ptr::null_mut(),
            lp_title: ptr::null_mut(),
            dw_x: 0,
            dw_y: 0,
            dw_x_size: 0,
            dw_y_size: 0,
            dw_x_count_chars: 0,
            dw_y_count_chars: 0,
            dw_fill_attribute: 0,
            dw_flags: STARTF_USESTDHANDLES,
            w_show_window: 0,
            cb_reserved2: 0,
            lp_reserved2: ptr::null_mut(),
            h_std_input: INVALID_HANDLE_VALUE,
            h_std_output: INVALID_HANDLE_VALUE,
            h_std_error: INVALID_HANDLE_VALUE,
        });
    }
}

pub fn get_startup_info_w(startup_info: *mut StartupInfoW) {
    if startup_info.is_null() {
        return;
    }
    unsafe {
        ptr::write(startup_info, StartupInfoW {
            cb: core::mem::size_of::<StartupInfoW>() as Dword,
            lp_reserved: ptr::null_mut(),
            lp_desktop: ptr::null_mut(),
            lp_title: ptr::null_mut(),
            dw_x: 0,
            dw_y: 0,
            dw_x_size: 0,
            dw_y_size: 0,
            dw_x_count_chars: 0,
            dw_y_count_chars: 0,
            dw_fill_attribute: 0,
            dw_flags: STARTF_USESTDHANDLES,
            w_show_window: 0,
            cb_reserved2: 0,
            lp_reserved2: ptr::null_mut(),
            h_std_input: INVALID_HANDLE_VALUE,
            h_std_output: INVALID_HANDLE_VALUE,
            h_std_error: INVALID_HANDLE_VALUE,
        });
    }
}

#[repr(C)]
struct StartupInfoA {
    cb: Dword,
    lp_reserved: Lpstr,
    lp_desktop: Lpstr,
    lp_title: Lpstr,
    dw_x: Dword,
    dw_y: Dword,
    dw_x_size: Dword,
    dw_y_size: Dword,
    dw_x_count_chars: Dword,
    dw_y_count_chars: Dword,
    dw_fill_attribute: Dword,
    dw_flags: Dword,
    w_show_window: Word,
    cb_reserved2: Word,
    lp_reserved2: *mut u8,
    h_std_input: Handle,
    h_std_output: Handle,
    h_std_error: Handle,
}

#[repr(C)]
struct StartupInfoW {
    cb: Dword,
    lp_reserved: *mut u16,
    lp_desktop: *mut u16,
    lp_title: *mut u16,
    dw_x: Dword,
    dw_y: Dword,
    dw_x_size: Dword,
    dw_y_size: Dword,
    dw_x_count_chars: Dword,
    dw_y_count_chars: Dword,
    dw_fill_attribute: Dword,
    dw_flags: Dword,
    w_show_window: Word,
    cb_reserved2: Word,
    lp_reserved2: *mut u8,
    h_std_input: Handle,
    h_std_output: Handle,
    h_std_error: Handle,
}

type Word = u16;

pub fn mul_div(
    n_multiplicand: i32,
    n_multiplier: i32,
    n_divisor: i32,
) -> i32 {
    if n_divisor == 0 {
        return -1;
    }
    let divisor = if n_divisor < 0 {
        -n_divisor
    } else {
        n_divisor
    };
    let multiplicand = if n_divisor < 0 {
        -n_multiplicand
    } else {
        n_multiplicand
    };
    let positive = (multiplicand >= 0 && n_multiplier >= 0) || (multiplicand < 0 && n_multiplier < 0);
    let product = multiplicand as i64 * n_multiplier as i64;
    let result = if positive {
        (product + (divisor as i64 / 2)) / divisor as i64
    } else {
        (product - (divisor as i64 / 2)) / divisor as i64
    };
    if result > i32::MAX as i64 || result < i32::MIN as i64 {
        return -1;
    }
    result as i32
}

pub fn get_system_registry_quota(
    quota_allowed: *mut Dword,
    quota_used: *mut Dword,
) -> Bool {
    unsafe {
        if !quota_allowed.is_null() {
            *quota_allowed = 2_000_000_000;
        }
        if !quota_used.is_null() {
            *quota_used = 100_000_000;
        }
    }
    TRUE
}

pub fn create_boundary_descriptor_a(_name: Lpcstr, _flags: Dword) -> Handle {
    ptr::null_mut()
}

pub fn create_boundary_descriptor_w(_name: Lpwstr, _flags: Dword) -> Handle {
    ptr::null_mut()
}

pub fn delete_boundary_descriptor(_boundary: Handle) {
    let _ = _boundary;
}

pub fn get_system_default_locale(_lang_id: *mut Dword, _locale: *mut Dword) -> Bool {
    TRUE
}

pub fn disable_thread_library_calls(_module: Handle) -> Bool {
    TRUE
}
