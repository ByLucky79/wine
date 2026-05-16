// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : user32.dll — Pencere oluşturma ve yönetim API'leri
//                      (CreateWindowEx, DestroyWindow, ShowWindow, MoveWindow vb.)
// Dosya Yolu         : compat/win32/dlls/rust/user32_core/src/wnd_create.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Windows pencere yaşam döngüsü fonksiyonlarının clean-room Rust
//   implementasyonu. CreateWindowA/W, CreateWindowExA/W, DestroyWindow,
//   ShowWindow, ShowWindowAsync, MoveWindow, SetWindowPos, GetWindowRect,
//   GetClientRect, IsWindow, IsWindowEnabled, IsWindowVisible,
//   EnableWindow, SetParent, GetParent, GetAncestor, FindWindowA/W,
//   FindWindowExA/W, EnumWindows, EnumChildWindows, EnumThreadWindows,
//   GetTopWindow, GetWindow, GetDesktopWindow, GetForegroundWindow,
//   SetForegroundWindow, BringWindowToTop, GetNextWindow,
//   SetWindowLongA/W, GetWindowLongA/W, SetWindowLongPtrA/W,
//   GetWindowLongPtrA/W API'lerini içerir.
//   MSDN public spesifikasyonuna dayanır; stub implementasyon.
//
// Bağımlı Dosyalar:
//   1-) user32_root.rs (üst crate — HWND, WNDPROC, stil sabitleri)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 83
// *******************************************************************

#![allow(dead_code)]

use crate::{
    BOOL, DWORD, FALSE, HINSTANCE, HMENU, HWND, INT, LONG, LONG_PTR,
    LPCSTR, LPCWSTR, LPSTR, LPVOID, LPARAM, RECT, TRUE, UINT,
    WNDENUMPROC,
};

// SetWindowPos bayrakları (Win32 public API)
pub const SWP_NOSIZE:         UINT = 0x0001;
pub const SWP_NOMOVE:         UINT = 0x0002;
pub const SWP_NOZORDER:       UINT = 0x0004;
pub const SWP_NOREDRAW:       UINT = 0x0008;
pub const SWP_NOACTIVATE:     UINT = 0x0010;
pub const SWP_FRAMECHANGED:   UINT = 0x0020;
pub const SWP_SHOWWINDOW:     UINT = 0x0040;
pub const SWP_HIDEWINDOW:     UINT = 0x0080;
pub const SWP_NOCOPYBITS:     UINT = 0x0100;
pub const SWP_NOOWNERZORDER:  UINT = 0x0200;
pub const SWP_NOSENDCHANGING: UINT = 0x0400;
pub const SWP_DEFERERASE:     UINT = 0x2000;
pub const SWP_ASYNCWINDOWPOS: UINT = 0x4000;
pub const SWP_DRAWFRAME:      UINT = SWP_FRAMECHANGED;
pub const SWP_NOREPOSITION:   UINT = SWP_NOOWNERZORDER;

// SetWindowPos Z-sırası sabitler
pub const HWND_TOP:       HWND = 0 as HWND;
pub const HWND_BOTTOM:    HWND = 1 as HWND;
pub const HWND_TOPMOST:   HWND = (-1i32 as usize) as HWND;
pub const HWND_NOTOPMOST: HWND = (-2i32 as usize) as HWND;
pub const HWND_MESSAGE:   HWND = (-3i32 as usize) as HWND;
pub const HWND_BROADCAST: HWND = 0xFFFF as usize as HWND;
pub const HWND_DESKTOP:   HWND = 0 as HWND;

// GetAncestor bayrakları
pub const GA_PARENT:    UINT = 1;
pub const GA_ROOT:      UINT = 2;
pub const GA_ROOTOWNER: UINT = 3;

// GetWindow ilişki sabitleri
pub const GW_HWNDFIRST:  UINT = 0;
pub const GW_HWNDLAST:   UINT = 1;
pub const GW_HWNDNEXT:   UINT = 2;
pub const GW_HWNDPREV:   UINT = 3;
pub const GW_OWNER:      UINT = 4;
pub const GW_CHILD:      UINT = 5;
pub const GW_ENABLEDPOPUP: UINT = 6;

/// Genişletilmiş ANSI pencere oluştur.
/// Win32: CreateWindowExA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn CreateWindowExA(
    _dw_ex_style:    DWORD,
    _lp_class_name:  LPCSTR,
    _lp_window_name: LPCSTR,
    _dw_style:       DWORD,
    _x:              INT,
    _y:              INT,
    _n_width:        INT,
    _n_height:       INT,
    _hwnd_parent:    HWND,
    _h_menu:         HMENU,
    _h_instance:     HINSTANCE,
    _lp_param:       LPVOID,
) -> HWND { core::ptr::null_mut() }

/// Genişletilmiş Unicode pencere oluştur.
/// Win32: CreateWindowExW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn CreateWindowExW(
    _dw_ex_style:    DWORD,
    _lp_class_name:  LPCWSTR,
    _lp_window_name: LPCWSTR,
    _dw_style:       DWORD,
    _x:              INT,
    _y:              INT,
    _n_width:        INT,
    _n_height:       INT,
    _hwnd_parent:    HWND,
    _h_menu:         HMENU,
    _h_instance:     HINSTANCE,
    _lp_param:       LPVOID,
) -> HWND { core::ptr::null_mut() }

/// Pencereyi yok et.
/// Win32: DestroyWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DestroyWindow(_hwnd: HWND) -> BOOL { FALSE }

/// Pencereyi göster / gizle / küçült / büyüt.
/// Win32: ShowWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ShowWindow(_hwnd: HWND, _n_cmd_show: INT) -> BOOL { FALSE }

/// Pencereyi asenkron göster.
/// Win32: ShowWindowAsync (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ShowWindowAsync(_hwnd: HWND, _n_cmd_show: INT) -> BOOL { FALSE }

/// Pencereyi taşı ve yeniden boyutlandır.
/// Win32: MoveWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn MoveWindow(
    _hwnd:    HWND,
    _x:       INT,
    _y:       INT,
    _n_width: INT,
    _n_height:INT,
    _b_repaint: BOOL,
) -> BOOL { FALSE }

/// Pencere konumunu, boyutunu ve Z-sırasını ayarla.
/// Win32: SetWindowPos (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowPos(
    _hwnd:             HWND,
    _hwnd_insert_after:HWND,
    _x:                INT,
    _y:                INT,
    _cx:               INT,
    _cy:               INT,
    _u_flags:          UINT,
) -> BOOL { FALSE }

/// Pencere ekran koordinatlarını al.
/// Win32: GetWindowRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowRect(
    _hwnd:    HWND,
    lp_rect:  *mut RECT,
) -> BOOL {
    if lp_rect.is_null() { return FALSE; }
    unsafe { *lp_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 }; }
    TRUE
}

/// İstemci alan koordinatlarını al.
/// Win32: GetClientRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetClientRect(
    _hwnd:    HWND,
    lp_rect:  *mut RECT,
) -> BOOL {
    if lp_rect.is_null() { return FALSE; }
    unsafe { *lp_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 }; }
    TRUE
}

/// Pencerenin geçerli olup olmadığını kontrol et.
/// Win32: IsWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn IsWindow(_hwnd: HWND) -> BOOL { FALSE }

/// Pencerenin etkin olup olmadığını kontrol et.
/// Win32: IsWindowEnabled (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn IsWindowEnabled(_hwnd: HWND) -> BOOL { FALSE }

/// Pencerenin görünür olup olmadığını kontrol et.
/// Win32: IsWindowVisible (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn IsWindowVisible(_hwnd: HWND) -> BOOL { FALSE }

/// Pencereyi etkinleştir / devre dışı bırak.
/// Win32: EnableWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn EnableWindow(_hwnd: HWND, _b_enable: BOOL) -> BOOL { FALSE }

/// Pencerenin üst penceresini değiştir.
/// Win32: SetParent (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetParent(_hwnd_child: HWND, _hwnd_new_parent: HWND) -> HWND {
    core::ptr::null_mut()
}

/// Üst pencereyi al.
/// Win32: GetParent (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetParent(_hwnd: HWND) -> HWND { core::ptr::null_mut() }

/// Atayı al (parent / root / root owner).
/// Win32: GetAncestor (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetAncestor(_hwnd: HWND, _ga_flags: UINT) -> HWND {
    core::ptr::null_mut()
}

/// ANSI pencere arama.
/// Win32: FindWindowA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FindWindowA(
    _lp_class_name:  LPCSTR,
    _lp_window_name: LPCSTR,
) -> HWND { core::ptr::null_mut() }

/// Unicode pencere arama.
/// Win32: FindWindowW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FindWindowW(
    _lp_class_name:  LPCWSTR,
    _lp_window_name: LPCWSTR,
) -> HWND { core::ptr::null_mut() }

/// ANSI genişletilmiş pencere arama.
/// Win32: FindWindowExA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FindWindowExA(
    _hwnd_parent:    HWND,
    _hwnd_child_after: HWND,
    _lp_class_name:  LPCSTR,
    _lp_window_name: LPCSTR,
) -> HWND { core::ptr::null_mut() }

/// Unicode genişletilmiş pencere arama.
/// Win32: FindWindowExW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FindWindowExW(
    _hwnd_parent:      HWND,
    _hwnd_child_after: HWND,
    _lp_class_name:    LPCWSTR,
    _lp_window_name:   LPCWSTR,
) -> HWND { core::ptr::null_mut() }

/// Tüm üst düzey pencereleri say.
/// Win32: EnumWindows (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn EnumWindows(
    _lp_enum_func: WNDENUMPROC,
    _l_param:      LPARAM,
) -> BOOL { TRUE }

/// Alt pencereleri say.
/// Win32: EnumChildWindows (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn EnumChildWindows(
    _hwnd_parent:  HWND,
    _lp_enum_func: WNDENUMPROC,
    _l_param:      LPARAM,
) -> BOOL { TRUE }

/// İş parçacığına ait pencereleri say.
/// Win32: EnumThreadWindows (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn EnumThreadWindows(
    _dw_thread_id: DWORD,
    _lp_fn:        WNDENUMPROC,
    _l_param:      LPARAM,
) -> BOOL { TRUE }

/// En üst alt pencereyi al.
/// Win32: GetTopWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetTopWindow(_hwnd: HWND) -> HWND { core::ptr::null_mut() }

/// Z-sırası ilişkisine göre pencere al.
/// Win32: GetWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindow(_hwnd: HWND, _u_cmd: UINT) -> HWND {
    core::ptr::null_mut()
}

/// Masaüstü penceresini al.
/// Win32: GetDesktopWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetDesktopWindow() -> HWND { core::ptr::null_mut() }

/// Ön plan penceresini al.
/// Win32: GetForegroundWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetForegroundWindow() -> HWND { core::ptr::null_mut() }

/// Ön plan penceresini ayarla.
/// Win32: SetForegroundWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetForegroundWindow(_hwnd: HWND) -> BOOL { FALSE }

/// Pencereyi Z-sırasının en üstüne taşı.
/// Win32: BringWindowToTop (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn BringWindowToTop(_hwnd: HWND) -> BOOL { FALSE }

/// Ertelenmiş pencere konumlandırma başlat.
/// Win32: BeginDeferWindowPos (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn BeginDeferWindowPos(_n_num_windows: INT) -> crate::HANDLE {
    core::ptr::null_mut()
}

/// Ertelenmiş pencere konumuna pozisyon ekle.
/// Win32: DeferWindowPos (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DeferWindowPos(
    _h_win_pos_info:    crate::HANDLE,
    _hwnd:              HWND,
    _hwnd_insert_after: HWND,
    _x:                 INT,
    _y:                 INT,
    _cx:                INT,
    _cy:                INT,
    _u_flags:           UINT,
) -> crate::HANDLE { core::ptr::null_mut() }

/// Ertelenmiş pencere konumlandırmasını uygula.
/// Win32: EndDeferWindowPos (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn EndDeferWindowPos(_h_win_pos_info: crate::HANDLE) -> BOOL { FALSE }

/// ANSI pencere long değerini al.
/// Win32: GetWindowLongA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongA(_hwnd: HWND, _n_index: crate::INT) -> LONG { 0 }

/// Unicode pencere long değerini al.
/// Win32: GetWindowLongW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongW(_hwnd: HWND, _n_index: crate::INT) -> LONG { 0 }

/// ANSI pencere long değerini ayarla.
/// Win32: SetWindowLongA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongA(
    _hwnd:        HWND,
    _n_index:     crate::INT,
    _dw_new_long: LONG,
) -> LONG { 0 }

/// Unicode pencere long değerini ayarla.
/// Win32: SetWindowLongW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongW(
    _hwnd:        HWND,
    _n_index:     crate::INT,
    _dw_new_long: LONG,
) -> LONG { 0 }

/// ANSI pencere pointer long değerini al (64-bit uyumlu).
/// Win32: GetWindowLongPtrA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongPtrA(
    _hwnd:    HWND,
    _n_index: crate::INT,
) -> LONG_PTR { 0 }

/// Unicode pencere pointer long değerini al.
/// Win32: GetWindowLongPtrW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongPtrW(
    _hwnd:    HWND,
    _n_index: crate::INT,
) -> LONG_PTR { 0 }

/// ANSI pencere pointer long değerini ayarla.
/// Win32: SetWindowLongPtrA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongPtrA(
    _hwnd:        HWND,
    _n_index:     crate::INT,
    _dw_new_long: LONG_PTR,
) -> LONG_PTR { 0 }

/// Unicode pencere pointer long değerini ayarla.
/// Win32: SetWindowLongPtrW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongPtrW(
    _hwnd:        HWND,
    _n_index:     crate::INT,
    _dw_new_long: LONG_PTR,
) -> LONG_PTR { 0 }

/// Pencerenin boyutunu ve konumunu aynı anda alır.
/// Win32: GetWindowPlacement (USER32.@)
#[repr(C)]
pub struct WINDOWPLACEMENT {
    pub length:           UINT,
    pub flags:            UINT,
    pub show_cmd:         UINT,
    pub pt_min_position:  crate::POINT,
    pub pt_max_position:  crate::POINT,
    pub rc_normal_position: RECT,
    pub rc_device:        RECT,
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowPlacement(
    _hwnd: HWND,
    lp_wnd_pl: *mut WINDOWPLACEMENT,
) -> BOOL {
    if lp_wnd_pl.is_null() { return FALSE; }
    FALSE
}

/// Pencerenin boyutunu ve konumunu aynı anda ayarla.
/// Win32: SetWindowPlacement (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowPlacement(
    _hwnd:     HWND,
    _lp_wnd_pl: *const WINDOWPLACEMENT,
) -> BOOL { FALSE }

/// ANSI pencere text konumlandırılmış çalışma zamanı yenileme.
/// Win32: SetWindowTextA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowTextA(
    _hwnd:      HWND,
    _lp_string: LPCSTR,
) -> BOOL { FALSE }

/// Unicode pencere başlığını ayarla.
/// Win32: SetWindowTextW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowTextW(
    _hwnd:      HWND,
    _lp_string: LPCWSTR,
) -> BOOL { FALSE }

/// ANSI pencere başlığını al.
/// Win32: GetWindowTextA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowTextA(
    _hwnd:      HWND,
    lp_string:  LPSTR,
    n_max_count:INT,
) -> INT {
    if lp_string.is_null() || n_max_count <= 0 { return 0; }
    0
}

/// Unicode pencere başlığını al.
/// Win32: GetWindowTextW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowTextW(
    _hwnd:       HWND,
    lp_string:   crate::LPWSTR,
    n_max_count: INT,
) -> INT {
    if lp_string.is_null() || n_max_count <= 0 { return 0; }
    0
}

/// Pencere başlığı uzunluğunu al (ANSI).
/// Win32: GetWindowTextLengthA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowTextLengthA(_hwnd: HWND) -> INT { 0 }

/// Pencere başlığı uzunluğunu al (Unicode).
/// Win32: GetWindowTextLengthW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowTextLengthW(_hwnd: HWND) -> INT { 0 }

/// Pencereyi pencere konumlandırma yapısına göre hizala.
/// Win32: AdjustWindowRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn AdjustWindowRect(
    lp_rect: *mut RECT,
    _dw_style:  DWORD,
    _b_menu:    BOOL,
) -> BOOL {
    if lp_rect.is_null() { return FALSE; }
    TRUE
}

/// Genişletilmiş stil ile pencere konumunu ayarla.
/// Win32: AdjustWindowRectEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn AdjustWindowRectEx(
    lp_rect:     *mut RECT,
    _dw_style:   DWORD,
    _b_menu:     BOOL,
    _dw_ex_style:DWORD,
) -> BOOL {
    if lp_rect.is_null() { return FALSE; }
    TRUE
}

/// Ekran koordinatlarını istemci koordinatlarına çevir.
/// Win32: ScreenToClient (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ScreenToClient(_hwnd: HWND, lp_point: *mut crate::POINT) -> BOOL {
    if lp_point.is_null() { return FALSE; }
    TRUE
}

/// İstemci koordinatlarını ekran koordinatlarına çevir.
/// Win32: ClientToScreen (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ClientToScreen(_hwnd: HWND, lp_point: *mut crate::POINT) -> BOOL {
    if lp_point.is_null() { return FALSE; }
    TRUE
}

/// Noktanın hangi pencereye ait olduğunu bul.
/// Win32: WindowFromPoint (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn WindowFromPoint(_point: crate::POINT) -> HWND {
    core::ptr::null_mut()
}

/// İstemci noktasının hangi alt pencereye ait olduğunu bul.
/// Win32: ChildWindowFromPoint (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ChildWindowFromPoint(
    _hwnd_parent: HWND,
    _point:       crate::POINT,
) -> HWND { core::ptr::null_mut() }

/// Bayraklı istemci noktası pencere arama.
/// Win32: ChildWindowFromPointEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ChildWindowFromPointEx(
    _hwnd_parent: HWND,
    _pt:          crate::POINT,
    _u_flags:     UINT,
) -> HWND { core::ptr::null_mut() }

/// Pencerenin iç içe durumunu kontrol et.
/// Win32: IsChild (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn IsChild(_hwnd_parent: HWND, _hwnd: HWND) -> BOOL { FALSE }

/// Pencere durumunu maximize/minimize sorgula.
/// Win32: IsZoomed (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn IsZoomed(_hwnd: HWND) -> BOOL { FALSE }

/// Win32: IsIconic (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn IsIconic(_hwnd: HWND) -> BOOL { FALSE }

/// Pencereyi öne getir.
/// Win32: OpenIcon (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn OpenIcon(_hwnd: HWND) -> BOOL { FALSE }

/// Pencereyi kapatma kutusundan kapat.
/// Win32: CloseWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn CloseWindow(_hwnd: HWND) -> BOOL { FALSE }

/// Pencere word değerini al (eski API).
/// Win32: GetWindowWord (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowWord(_hwnd: HWND, _n_index: INT) -> crate::WORD { 0 }

/// Pencere word değerini ayarla (eski API).
/// Win32: SetWindowWord (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowWord(
    _hwnd:       HWND,
    _n_index:    INT,
    _w_new_word: crate::WORD,
) -> crate::WORD { 0 }

/// Aktif pencereyi al.
/// Win32: GetActiveWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetActiveWindow() -> HWND { core::ptr::null_mut() }

/// Aktif pencereyi ayarla.
/// Win32: SetActiveWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetActiveWindow(_hwnd: HWND) -> HWND { core::ptr::null_mut() }

/// Odak penceresini al.
/// Win32: GetFocus (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetFocus() -> HWND { core::ptr::null_mut() }

/// Odağı ayarla.
/// Win32: SetFocus (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetFocus(_hwnd: HWND) -> HWND { core::ptr::null_mut() }

/// Pencereye bölge ata.
/// Win32: SetWindowRgn (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetWindowRgn(
    _hwnd:        HWND,
    _h_rgn:       crate::HRGN,
    _b_redraw:    BOOL,
) -> INT { 0 }

/// Pencere bölgesini al.
/// Win32: GetWindowRgn (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowRgn(
    _hwnd:  HWND,
    _h_rgn: crate::HRGN,
) -> INT { 0 /* NULLREGION */ }
