// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : user32.dll — Sistem parametreleri ve renk API'leri
//                      (GetSystemMetrics, SystemParametersInfo, GetSysColor vb.)
// Dosya Yolu         : compat/win32/dlls/rust/user32_core/src/system_param.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Windows sistem parametresi fonksiyonlarının clean-room Rust
//   implementasyonu. GetSystemMetrics, SystemParametersInfoA/W,
//   GetSysColor, GetSysColorBrush, SetSysColors,
//   GetSystemMetricsForDpi, SystemParametersInfoForDpi,
//   ArrangeIconicWindows, CascadeWindows, TileWindows,
//   ExitWindowsEx, ShutdownBlockReasonCreate, ShutdownBlockReasonDestroy,
//   ShutdownBlockReasonQuery, LockWorkStation API'lerini içerir.
//   MSDN public spesifikasyonuna dayanır; stub implementasyon.
//
// Bağımlı Dosyalar:
//   1-) user32_root.rs (üst crate — SM_* sabitleri, COLORREF, HBRUSH)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 83
// *******************************************************************

#![allow(dead_code)]

use crate::{
    BOOL, COLORREF, DWORD, FALSE, HBRUSH, HWND, INT, LPCWSTR, LPVOID,
    UINT,
};

// GetSysColor renk sabitleri (Win32 public API)
pub const COLOR_SCROLLBAR:             INT = 0;
pub const COLOR_BACKGROUND:            INT = 1;
pub const COLOR_ACTIVECAPTION:         INT = 2;
pub const COLOR_INACTIVECAPTION:       INT = 3;
pub const COLOR_MENU:                  INT = 4;
pub const COLOR_WINDOW:                INT = 5;
pub const COLOR_WINDOWFRAME:           INT = 6;
pub const COLOR_MENUTEXT:              INT = 7;
pub const COLOR_WINDOWTEXT:            INT = 8;
pub const COLOR_CAPTIONTEXT:           INT = 9;
pub const COLOR_ACTIVEBORDER:          INT = 10;
pub const COLOR_INACTIVEBORDER:        INT = 11;
pub const COLOR_APPWORKSPACE:          INT = 12;
pub const COLOR_HIGHLIGHT:             INT = 13;
pub const COLOR_HIGHLIGHTTEXT:         INT = 14;
pub const COLOR_BTNFACE:               INT = 15;
pub const COLOR_BTNSHADOW:             INT = 16;
pub const COLOR_GRAYTEXT:              INT = 17;
pub const COLOR_BTNTEXT:               INT = 18;
pub const COLOR_INACTIVECAPTIONTEXT:   INT = 19;
pub const COLOR_BTNHIGHLIGHT:          INT = 20;
pub const COLOR_3DDKSHADOW:            INT = 21;
pub const COLOR_3DLIGHT:               INT = 22;
pub const COLOR_INFOTEXT:              INT = 23;
pub const COLOR_INFOBK:                INT = 24;
pub const COLOR_HOTLIGHT:              INT = 26;
pub const COLOR_GRADIENTACTIVECAPTION: INT = 27;
pub const COLOR_GRADIENTINACTIVECAPTION: INT = 28;
pub const COLOR_MENUHILIGHT:           INT = 29;
pub const COLOR_MENUBAR:               INT = 30;
pub const COLOR_DESKTOP:               INT = COLOR_BACKGROUND;
pub const COLOR_3DFACE:                INT = COLOR_BTNFACE;
pub const COLOR_3DSHADOW:              INT = COLOR_BTNSHADOW;
pub const COLOR_3DHIGHLIGHT:           INT = COLOR_BTNHIGHLIGHT;
pub const COLOR_3DHILIGHT:             INT = COLOR_BTNHIGHLIGHT;
pub const COLOR_BTNHILIGHT:            INT = COLOR_BTNHIGHLIGHT;

// SystemParametersInfo işlem kodları
pub const SPI_GETBEEP:                    UINT = 0x0001;
pub const SPI_SETBEEP:                    UINT = 0x0002;
pub const SPI_GETMOUSE:                   UINT = 0x0003;
pub const SPI_SETMOUSE:                   UINT = 0x0004;
pub const SPI_GETBORDER:                  UINT = 0x0005;
pub const SPI_SETBORDER:                  UINT = 0x0006;
pub const SPI_GETKEYBOARDSPEED:           UINT = 0x000A;
pub const SPI_SETKEYBOARDSPEED:           UINT = 0x000B;
pub const SPI_LANGDRIVER:                 UINT = 0x000C;
pub const SPI_ICONHORIZONTALSPACING:      UINT = 0x000D;
pub const SPI_GETSCREENSAVETIMEOUT:       UINT = 0x000E;
pub const SPI_SETSCREENSAVETIMEOUT:       UINT = 0x000F;
pub const SPI_GETSCREENSAVEACTIVE:        UINT = 0x0010;
pub const SPI_SETSCREENSAVEACTIVE:        UINT = 0x0011;
pub const SPI_GETGRIDGRANULARITY:         UINT = 0x0012;
pub const SPI_SETGRIDGRANULARITY:         UINT = 0x0013;
pub const SPI_SETDESKWALLPAPER:           UINT = 0x0014;
pub const SPI_SETDESKPATTERN:             UINT = 0x0015;
pub const SPI_GETKEYBOARDDELAY:           UINT = 0x0016;
pub const SPI_SETKEYBOARDDELAY:           UINT = 0x0017;
pub const SPI_ICONVERTICALSPACING:        UINT = 0x0018;
pub const SPI_GETICONTITLEWRAP:           UINT = 0x0019;
pub const SPI_SETICONTITLEWRAP:           UINT = 0x001A;
pub const SPI_GETMENUDROPALIGNMENT:       UINT = 0x001B;
pub const SPI_SETMENUDROPALIGNMENT:       UINT = 0x001C;
pub const SPI_SETDOUBLECLKWIDTH:          UINT = 0x001D;
pub const SPI_SETDOUBLECLKHEIGHT:         UINT = 0x001E;
pub const SPI_GETICONTITLELOGFONT:        UINT = 0x001F;
pub const SPI_SETDOUBLECLICKTIME:         UINT = 0x0020;
pub const SPI_SETMOUSEBUTTONSWAP:         UINT = 0x0021;
pub const SPI_SETICONTITLELOGFONT:        UINT = 0x0022;
pub const SPI_GETFASTTASKSWITCH:          UINT = 0x0023;
pub const SPI_SETFASTTASKSWITCH:          UINT = 0x0024;
pub const SPI_SETDRAGFULLWINDOWS:         UINT = 0x0025;
pub const SPI_GETDRAGFULLWINDOWS:         UINT = 0x0026;
pub const SPI_GETNONCLIENTMETRICS:        UINT = 0x0029;
pub const SPI_SETNONCLIENTMETRICS:        UINT = 0x002A;
pub const SPI_GETMINIMIZEDMETRICS:        UINT = 0x002B;
pub const SPI_SETMINIMIZEDMETRICS:        UINT = 0x002C;
pub const SPI_GETICONMETRICS:             UINT = 0x002D;
pub const SPI_SETICONMETRICS:             UINT = 0x002E;
pub const SPI_SETWORKAREA:                UINT = 0x002F;
pub const SPI_GETWORKAREA:                UINT = 0x0030;
pub const SPI_SETPENWINDOWS:              UINT = 0x0031;
pub const SPI_GETHIGHCONTRAST:            UINT = 0x0042;
pub const SPI_SETHIGHCONTRAST:            UINT = 0x0043;
pub const SPI_GETKEYBOARDPREF:            UINT = 0x0044;
pub const SPI_SETKEYBOARDPREF:            UINT = 0x0045;
pub const SPI_GETSCREENREADER:            UINT = 0x0046;
pub const SPI_SETSCREENREADER:            UINT = 0x0047;
pub const SPI_GETANIMATION:               UINT = 0x0048;
pub const SPI_SETANIMATION:               UINT = 0x0049;
pub const SPI_GETFONTSMOOTHING:           UINT = 0x004A;
pub const SPI_SETFONTSMOOTHING:           UINT = 0x004B;
pub const SPI_SETDRAGWIDTH:               UINT = 0x004C;
pub const SPI_SETDRAGHEIGHT:              UINT = 0x004D;
pub const SPI_SETHANDHELD:                UINT = 0x004E;
pub const SPI_GETLOWPOWERTIMEOUT:         UINT = 0x004F;
pub const SPI_GETPOWEROFFTIMEOUT:         UINT = 0x0050;
pub const SPI_SETLOWPOWERTIMEOUT:         UINT = 0x0051;
pub const SPI_SETPOWEROFFTIMEOUT:         UINT = 0x0052;
pub const SPI_GETLOWPOWERACTIVE:          UINT = 0x0053;
pub const SPI_GETPOWEROFFACTIVE:          UINT = 0x0054;
pub const SPI_SETLOWPOWERACTIVE:          UINT = 0x0055;
pub const SPI_SETPOWEROFFACTIVE:          UINT = 0x0056;
pub const SPI_SETCURSORS:                 UINT = 0x0057;
pub const SPI_SETICONS:                   UINT = 0x0058;
pub const SPI_GETDEFAULTINPUTLANG:        UINT = 0x0059;
pub const SPI_SETDEFAULTINPUTLANG:        UINT = 0x005A;
pub const SPI_SETLANGTOGGLE:              UINT = 0x005B;
pub const SPI_GETWINDOWSEXTENSION:        UINT = 0x005C;
pub const SPI_SETMOUSETRAILS:             UINT = 0x005D;
pub const SPI_GETMOUSETRAILS:             UINT = 0x005E;
pub const SPI_SETSCREENSAVERRUNNING:      UINT = 0x0061;
pub const SPI_SCREENSAVERRUNNING:         UINT = SPI_SETSCREENSAVERRUNNING;
pub const SPI_GETFILTERKEYS:              UINT = 0x0032;
pub const SPI_SETFILTERKEYS:              UINT = 0x0033;
pub const SPI_GETTOGGLEKEYS:              UINT = 0x0034;
pub const SPI_SETTOGGLEKEYS:              UINT = 0x0035;
pub const SPI_GETMOUSEKEYS:               UINT = 0x0036;
pub const SPI_SETMOUSEKEYS:               UINT = 0x0037;
pub const SPI_GETSHOWSOUNDS:              UINT = 0x0038;
pub const SPI_SETSHOWSOUNDS:              UINT = 0x0039;
pub const SPI_GETSTICKYKEYS:              UINT = 0x003A;
pub const SPI_SETSTICKYKEYS:              UINT = 0x003B;
pub const SPI_GETACCESSTIMEOUT:           UINT = 0x003C;
pub const SPI_SETACCESSTIMEOUT:           UINT = 0x003D;
pub const SPI_GETSERIALKEYS:              UINT = 0x003E;
pub const SPI_SETSERIALKEYS:              UINT = 0x003F;
pub const SPI_GETSOUNDSENTRY:             UINT = 0x0040;
pub const SPI_SETSOUNDSENTRY:             UINT = 0x0041;
pub const SPI_GETSNAPTODEFBUTTON:         UINT = 0x005F;
pub const SPI_SETSNAPTODEFBUTTON:         UINT = 0x0060;
pub const SPI_GETMOUSEHOVERWIDTH:         UINT = 0x0062;
pub const SPI_SETMOUSEHOVERWIDTH:         UINT = 0x0063;
pub const SPI_GETMOUSEHOVERHEIGHT:        UINT = 0x0064;
pub const SPI_SETMOUSEHOVERHEIGHT:        UINT = 0x0065;
pub const SPI_GETMOUSEHOVERTIME:          UINT = 0x0066;
pub const SPI_SETMOUSEHOVERTIME:          UINT = 0x0067;
pub const SPI_GETWHEELSCROLLLINES:        UINT = 0x0068;
pub const SPI_SETWHEELSCROLLLINES:        UINT = 0x0069;
pub const SPI_GETMENUSHOWDELAY:           UINT = 0x006A;
pub const SPI_SETMENUSHOWDELAY:           UINT = 0x006B;
pub const SPI_GETWHEELSCROLLCHARS:        UINT = 0x006C;
pub const SPI_SETWHEELSCROLLCHARS:        UINT = 0x006D;
pub const SPI_GETSHOWIMEUI:               UINT = 0x006E;
pub const SPI_SETSHOWIMEUI:               UINT = 0x006F;

// ExitWindowsEx bayrakları
pub const EWX_LOGOFF:    UINT = 0x0000_0000;
pub const EWX_SHUTDOWN:  UINT = 0x0000_0001;
pub const EWX_REBOOT:    UINT = 0x0000_0002;
pub const EWX_FORCE:     UINT = 0x0000_0004;
pub const EWX_POWEROFF:  UINT = 0x0000_0008;
pub const EWX_FORCEIFHUNG: UINT = 0x0000_0010;
pub const EWX_QUICKRESOLVE: UINT = 0x0000_0020;
pub const EWX_RESTARTAPPS:  UINT = 0x0000_0040;
pub const EWX_HYBRID_SHUTDOWN: UINT = 0x0040_0000;
pub const EWX_BOOTOPTIONS:  UINT = 0x0100_0000;

/// Sistem metriği değerini al.
/// Win32: GetSystemMetrics (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetSystemMetrics(n_index: INT) -> INT {
    match n_index {
        crate::SM_CXSCREEN => 1280,
        crate::SM_CYSCREEN => 720,
        crate::SM_CMOUSEBUTTONS => 3,
        crate::SM_MOUSEPRESENT => 1,
        _ => 0,
    }
}

/// DPI'ya göre sistem metriği değerini al.
/// Win32: GetSystemMetricsForDpi (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetSystemMetricsForDpi(n_index: INT, _dpi: UINT) -> INT {
    GetSystemMetrics(n_index)
}

/// ANSI sistem parametresini al / ayarla.
/// Win32: SystemParametersInfoA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SystemParametersInfoA(
    _u_action:    UINT,
    _u_param:     UINT,
    _pv_param:    LPVOID,
    _f_win_ini:   UINT,
) -> BOOL { FALSE }

/// Unicode sistem parametresini al / ayarla.
/// Win32: SystemParametersInfoW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SystemParametersInfoW(
    _u_action:  UINT,
    _u_param:   UINT,
    _pv_param:  LPVOID,
    _f_win_ini: UINT,
) -> BOOL { FALSE }

/// DPI'ya göre sistem parametresini al.
/// Win32: SystemParametersInfoForDpi (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SystemParametersInfoForDpi(
    _u_action: UINT,
    _u_param:  UINT,
    _pv_param: LPVOID,
    _f_win_ini:UINT,
    _dpi:      UINT,
) -> BOOL { FALSE }

/// Sistem rengi al.
/// Win32: GetSysColor (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetSysColor(n_index: INT) -> COLORREF {
    match n_index {
        COLOR_WINDOW     => 0x00FF_FFFF,
        COLOR_WINDOWTEXT => 0x0000_0000,
        COLOR_BTNFACE    => 0x00F0_F0F0,
        COLOR_HIGHLIGHT  => 0x0000_78D7,
        COLOR_HIGHLIGHTTEXT => 0x00FF_FFFF,
        _ => 0x00C0_C0C0,
    }
}

/// Sistem rengi fırçası al.
/// Win32: GetSysColorBrush (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetSysColorBrush(_n_index: INT) -> HBRUSH { core::ptr::null_mut() }

/// Sistem renklerini ayarla.
/// Win32: SetSysColors (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetSysColors(
    _c_elements:          INT,
    _lp_a_elements:       *const INT,
    _lp_a_rgb_values:     *const COLORREF,
) -> BOOL { FALSE }

/// Simge haline getirilmiş pencereleri düzenle.
/// Win32: ArrangeIconicWindows (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ArrangeIconicWindows(_hwnd: HWND) -> UINT { 0 }

/// Windows'u kapat veya yeniden başlat.
/// Win32: ExitWindowsEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ExitWindowsEx(_u_flags: UINT, _dw_reason: DWORD) -> BOOL { FALSE }

/// İş istasyonunu kilitle.
/// Win32: LockWorkStation (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn LockWorkStation() -> BOOL { FALSE }

/// Kapatma engelleme nedeni oluştur.
/// Win32: ShutdownBlockReasonCreate (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ShutdownBlockReasonCreate(
    _hwnd:   HWND,
    _pwsz_reason: LPCWSTR,
) -> BOOL { FALSE }

/// Kapatma engelleme nedenini yok et.
/// Win32: ShutdownBlockReasonDestroy (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ShutdownBlockReasonDestroy(_hwnd: HWND) -> BOOL { FALSE }

/// Kapatma engelleme nedenini sorgula.
/// Win32: ShutdownBlockReasonQuery (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ShutdownBlockReasonQuery(
    _hwnd:      HWND,
    pwsz_buf:   crate::LPWSTR,
    pcch_buf:   *mut DWORD,
) -> BOOL {
    if !pcch_buf.is_null() { unsafe { *pcch_buf = 0; } }
    if !pwsz_buf.is_null() { return FALSE; }
    FALSE
}

/// Güncel DPI'yı al.
/// Win32: GetDpiForSystem (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetDpiForSystem() -> UINT { 96 }

/// Pencere DPI'sini al.
/// Win32: GetDpiForWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetDpiForWindow(_hwnd: HWND) -> UINT { 96 }

/// İş parçacığı DPI farkındalık bağlamı al.
/// Win32: GetThreadDpiAwarenessContext (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetThreadDpiAwarenessContext() -> crate::HANDLE {
    core::ptr::null_mut()
}

/// İş parçacığı DPI farkındalık bağlamını ayarla.
/// Win32: SetThreadDpiAwarenessContext (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetThreadDpiAwarenessContext(
    _ctx: crate::HANDLE,
) -> crate::HANDLE { core::ptr::null_mut() }
