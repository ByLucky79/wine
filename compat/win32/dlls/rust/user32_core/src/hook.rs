// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : user32.dll — Pencere kanca (hook) API'leri
//                      (SetWindowsHookEx, CallNextHookEx, UnhookWindowsHookEx vb.)
// Dosya Yolu         : compat/win32/dlls/rust/user32_core/src/hook.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Windows pencere kanca (hook) fonksiyonlarının clean-room Rust
//   implementasyonu. SetWindowsHookA/W, SetWindowsHookExA/W,
//   UnhookWindowsHook, UnhookWindowsHookEx, CallNextHookEx,
//   CallMsgFilterA/W API'lerini içerir.
//   MSDN public spesifikasyonuna dayanır; stub implementasyon.
//
// Bağımlı Dosyalar:
//   1-) user32_root.rs (üst crate — WH_* sabitleri, HHOOK, HOOKPROC)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 83
// *******************************************************************

#![allow(dead_code)]

use crate::{
    BOOL, FALSE, HHOOK, HINSTANCE, HOOKPROC, INT, LPARAM, LRESULT, MSG,
    UINT, WPARAM,
};

/// ANSI kanca yükle (eski API, SetWindowsHookExA kullan).
/// Win32: SetWindowsHookA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookA(
    _id_hook:   INT,
    _lp_fn:     HOOKPROC,
) -> HHOOK { core::ptr::null_mut() }

/// Unicode kanca yükle (eski API).
/// Win32: SetWindowsHookW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookW(
    _id_hook: INT,
    _lp_fn:   HOOKPROC,
) -> HHOOK { core::ptr::null_mut() }

/// ANSI kanca yükle (tercih edilen API).
/// Win32: SetWindowsHookExA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookExA(
    _id_hook:    INT,
    _lp_fn:      HOOKPROC,
    _h_mod:      HINSTANCE,
    _dw_thread_id: UINT,
) -> HHOOK { core::ptr::null_mut() }

/// Unicode kanca yükle (tercih edilen API).
/// Win32: SetWindowsHookExW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookExW(
    _id_hook:      INT,
    _lp_fn:        HOOKPROC,
    _h_mod:        HINSTANCE,
    _dw_thread_id: UINT,
) -> HHOOK { core::ptr::null_mut() }

/// Eski kanca API'sini kaldır.
/// Win32: UnhookWindowsHook (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn UnhookWindowsHook(_n_code: INT, _pfn_filter_proc: HOOKPROC) -> BOOL {
    FALSE
}

/// Kancayı kaldır.
/// Win32: UnhookWindowsHookEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn UnhookWindowsHookEx(_hhk: HHOOK) -> BOOL { FALSE }

/// Bir sonraki kancayı çağır.
/// Win32: CallNextHookEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn CallNextHookEx(
    _hhk:     HHOOK,
    _n_code:  INT,
    _w_param: WPARAM,
    _l_param: LPARAM,
) -> LRESULT { 0 }

/// ANSI mesaj filtresini çağır.
/// Win32: CallMsgFilterA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn CallMsgFilterA(
    lp_msg:  *const MSG,
    _n_code: INT,
) -> BOOL {
    if lp_msg.is_null() { return FALSE; }
    FALSE
}

/// Unicode mesaj filtresini çağır.
/// Win32: CallMsgFilterW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn CallMsgFilterW(
    lp_msg:  *const MSG,
    _n_code: INT,
) -> BOOL {
    if lp_msg.is_null() { return FALSE; }
    FALSE
}
