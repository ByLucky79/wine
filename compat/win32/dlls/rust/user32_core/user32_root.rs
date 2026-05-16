// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : user32.dll — Windows pencere sistemi kök crate dosyası
//                      Ortak Win32 tip tanımları, yapılar ve sabitler
// Dosya Yolu         : compat/win32/dlls/rust/user32_core/user32_root.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   user32.dll clean-room Rust implementasyonu için kök crate.
//   Tüm alt modüller buraya bağlanır. Windows pencere sistemi, mesaj
//   kuyruğu, diyalog, giriş, kaynak, pano, menü, kanca, monitör ve
//   sistem parametresi API'lerini stub olarak sağlar.
//   MSDN public spesifikasyonuna dayanır; Wine yalnızca fonksiyon
//   ismi referansı olarak kullanıldı — sıfır kod kopyası.
//
// Bağımlı Dosyalar:
//   14 alt modül (src/ dizini altında)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 83
// *******************************************************************

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate alloc;

// ── Temel C tipleri ────────────────────────────────────────────────────────
pub type BOOL      = i32;
pub type BYTE      = u8;
pub type WORD      = u16;
pub type DWORD     = u32;
pub type LONG      = i32;
pub type ULONG     = u32;
pub type UINT      = u32;
pub type INT       = i32;
pub type SHORT     = i16;
pub type USHORT    = u16;
pub type WCHAR     = u16;
pub type CHAR      = u8;
pub type LPVOID    = *mut core::ffi::c_void;
pub type LPCVOID   = *const core::ffi::c_void;
pub type LPSTR     = *mut CHAR;
pub type LPCSTR    = *const CHAR;
pub type LPWSTR    = *mut WCHAR;
pub type LPCWSTR   = *const WCHAR;
pub type HANDLE    = *mut core::ffi::c_void;
pub type HINSTANCE = HANDLE;
pub type HMODULE   = HANDLE;
pub type HWND      = HANDLE;
pub type HDC       = HANDLE;
pub type HGDIOBJ   = HANDLE;
pub type HMENU     = HANDLE;
pub type HACCEL    = HANDLE;
pub type HBITMAP   = HANDLE;
pub type HBRUSH    = HANDLE;
pub type HCURSOR   = HANDLE;
pub type HICON     = HANDLE;
pub type HFONT     = HANDLE;
pub type HPEN      = HANDLE;
pub type HRGN      = HANDLE;
pub type HMONITOR  = HANDLE;
pub type HHOOK     = HANDLE;
pub type HRAWINPUT = HANDLE;
pub type HDESK     = HANDLE;
pub type HWINSTA   = HANDLE;
pub type HKL       = HANDLE;
pub type ATOM      = WORD;
pub type WPARAM    = usize;
pub type LPARAM    = isize;
pub type LRESULT   = isize;
pub type UINT_PTR  = usize;
pub type LONG_PTR  = isize;
pub type DWORD_PTR = usize;
pub type COLORREF  = DWORD;
pub type LCID      = DWORD;
pub type LANGID    = WORD;
pub type LPDWORD   = *mut DWORD;
pub type LPBOOL    = *mut BOOL;
pub type LPLONG    = *mut LONG;
pub type LPWORD    = *mut WORD;
pub type LPINT     = *mut INT;
pub type LPBYTE    = *mut BYTE;

// ── İleri bildirimler ──────────────────────────────────────────────────────
pub type WNDPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>;
pub type HOOKPROC = Option<unsafe extern "system" fn(INT, WPARAM, LPARAM) -> LRESULT>;
pub type TIMERPROC = Option<unsafe extern "system" fn(HWND, UINT, UINT_PTR, DWORD)>;
pub type GRAYSTRINGPROC = Option<unsafe extern "system" fn(HDC, LPARAM, INT) -> BOOL>;
pub type DRAWSTATEPROC = Option<unsafe extern "system" fn(HDC, LPARAM, WPARAM, INT, INT) -> BOOL>;
pub type WNDENUMPROC = Option<unsafe extern "system" fn(HWND, LPARAM) -> BOOL>;
pub type MONITORENUMPROC = Option<unsafe extern "system" fn(HMONITOR, HDC, *mut RECT, LPARAM) -> BOOL>;
pub type SENDASYNCPROC = Option<unsafe extern "system" fn(HWND, UINT, ULONG_PTR, LRESULT)>;
pub type ULONG_PTR = usize;

// ── BOOL sabitleri ─────────────────────────────────────────────────────────
pub const TRUE:  BOOL = 1;
pub const FALSE: BOOL = 0;

// ── Genel hata sabitleri ───────────────────────────────────────────────────
pub const ERROR_SUCCESS:          DWORD = 0;
pub const ERROR_INVALID_HANDLE:   DWORD = 6;
pub const ERROR_INVALID_PARAMETER:DWORD = 87;
pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

// ── ShowWindow komutu sabitleri ────────────────────────────────────────────
pub const SW_HIDE:            INT = 0;
pub const SW_SHOWNORMAL:      INT = 1;
pub const SW_NORMAL:          INT = 1;
pub const SW_SHOWMINIMIZED:   INT = 2;
pub const SW_SHOWMAXIMIZED:   INT = 3;
pub const SW_MAXIMIZE:        INT = 3;
pub const SW_SHOWNOACTIVATE:  INT = 4;
pub const SW_SHOW:            INT = 5;
pub const SW_MINIMIZE:        INT = 6;
pub const SW_SHOWMINNOACTIVE: INT = 7;
pub const SW_SHOWNA:          INT = 8;
pub const SW_RESTORE:         INT = 9;
pub const SW_SHOWDEFAULT:     INT = 10;
pub const SW_FORCEMINIMIZE:   INT = 11;

// ── Window style sabitleri ─────────────────────────────────────────────────
pub const WS_OVERLAPPED:   DWORD = 0x0000_0000;
pub const WS_POPUP:        DWORD = 0x8000_0000;
pub const WS_CHILD:        DWORD = 0x4000_0000;
pub const WS_MINIMIZE:     DWORD = 0x2000_0000;
pub const WS_VISIBLE:      DWORD = 0x1000_0000;
pub const WS_DISABLED:     DWORD = 0x0800_0000;
pub const WS_CLIPSIBLINGS: DWORD = 0x0400_0000;
pub const WS_CLIPCHILDREN: DWORD = 0x0200_0000;
pub const WS_MAXIMIZE:     DWORD = 0x0100_0000;
pub const WS_CAPTION:      DWORD = 0x00C0_0000;
pub const WS_BORDER:       DWORD = 0x0080_0000;
pub const WS_DLGFRAME:     DWORD = 0x0040_0000;
pub const WS_VSCROLL:      DWORD = 0x0020_0000;
pub const WS_HSCROLL:      DWORD = 0x0010_0000;
pub const WS_SYSMENU:      DWORD = 0x0008_0000;
pub const WS_THICKFRAME:   DWORD = 0x0004_0000;
pub const WS_GROUP:        DWORD = 0x0002_0000;
pub const WS_TABSTOP:      DWORD = 0x0001_0000;
pub const WS_MINIMIZEBOX:  DWORD = 0x0002_0000;
pub const WS_MAXIMIZEBOX:  DWORD = 0x0001_0000;
pub const WS_OVERLAPPEDWINDOW: DWORD =
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
pub const WS_POPUPWINDOW: DWORD = WS_POPUP | WS_BORDER | WS_SYSMENU;

// ── Extended Window style sabitleri ───────────────────────────────────────
pub const WS_EX_DLGMODALFRAME:    DWORD = 0x0000_0001;
pub const WS_EX_NOPARENTNOTIFY:   DWORD = 0x0000_0004;
pub const WS_EX_TOPMOST:          DWORD = 0x0000_0008;
pub const WS_EX_ACCEPTFILES:      DWORD = 0x0000_0010;
pub const WS_EX_TRANSPARENT:      DWORD = 0x0000_0020;
pub const WS_EX_MDICHILD:         DWORD = 0x0000_0040;
pub const WS_EX_TOOLWINDOW:       DWORD = 0x0000_0080;
pub const WS_EX_WINDOWEDGE:       DWORD = 0x0000_0100;
pub const WS_EX_CLIENTEDGE:       DWORD = 0x0000_0200;
pub const WS_EX_CONTEXTHELP:      DWORD = 0x0000_0400;
pub const WS_EX_RIGHT:            DWORD = 0x0000_1000;
pub const WS_EX_LEFT:             DWORD = 0x0000_0000;
pub const WS_EX_RTLREADING:       DWORD = 0x0000_2000;
pub const WS_EX_LTRREADING:       DWORD = 0x0000_0000;
pub const WS_EX_LEFTSCROLLBAR:    DWORD = 0x0000_4000;
pub const WS_EX_RIGHTSCROLLBAR:   DWORD = 0x0000_0000;
pub const WS_EX_CONTROLPARENT:    DWORD = 0x0001_0000;
pub const WS_EX_STATICEDGE:       DWORD = 0x0002_0000;
pub const WS_EX_APPWINDOW:        DWORD = 0x0004_0000;
pub const WS_EX_OVERLAPPEDWINDOW: DWORD = WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE;
pub const WS_EX_PALETTEWINDOW:    DWORD = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST;
pub const WS_EX_LAYERED:          DWORD = 0x0008_0000;
pub const WS_EX_NOINHERITLAYOUT:  DWORD = 0x0010_0000;
pub const WS_EX_LAYOUTRTL:        DWORD = 0x0040_0000;
pub const WS_EX_COMPOSITED:       DWORD = 0x0200_0000;
pub const WS_EX_NOACTIVATE:       DWORD = 0x0800_0000;

// ── Pencere mesaj sabitleri (WM_*) ─────────────────────────────────────────
pub const WM_NULL:              UINT = 0x0000;
pub const WM_CREATE:            UINT = 0x0001;
pub const WM_DESTROY:           UINT = 0x0002;
pub const WM_MOVE:              UINT = 0x0003;
pub const WM_SIZE:              UINT = 0x0005;
pub const WM_ACTIVATE:          UINT = 0x0006;
pub const WM_SETFOCUS:          UINT = 0x0007;
pub const WM_KILLFOCUS:         UINT = 0x0008;
pub const WM_ENABLE:            UINT = 0x000A;
pub const WM_SETREDRAW:         UINT = 0x000B;
pub const WM_SETTEXT:           UINT = 0x000C;
pub const WM_GETTEXT:           UINT = 0x000D;
pub const WM_GETTEXTLENGTH:     UINT = 0x000E;
pub const WM_PAINT:             UINT = 0x000F;
pub const WM_CLOSE:             UINT = 0x0010;
pub const WM_QUERYENDSESSION:   UINT = 0x0011;
pub const WM_QUIT:              UINT = 0x0012;
pub const WM_QUERYOPEN:         UINT = 0x0013;
pub const WM_ERASEBKGND:        UINT = 0x0014;
pub const WM_SYSCOLORCHANGE:    UINT = 0x0015;
pub const WM_ENDSESSION:        UINT = 0x0016;
pub const WM_SHOWWINDOW:        UINT = 0x0018;
pub const WM_WININICHANGE:      UINT = 0x001A;
pub const WM_SETTINGCHANGE:     UINT = WM_WININICHANGE;
pub const WM_DEVMODECHANGE:     UINT = 0x001B;
pub const WM_ACTIVATEAPP:       UINT = 0x001C;
pub const WM_FONTCHANGE:        UINT = 0x001D;
pub const WM_TIMECHANGE:        UINT = 0x001E;
pub const WM_CANCELMODE:        UINT = 0x001F;
pub const WM_SETCURSOR:         UINT = 0x0020;
pub const WM_MOUSEACTIVATE:     UINT = 0x0021;
pub const WM_CHILDACTIVATE:     UINT = 0x0022;
pub const WM_QUEUESYNC:         UINT = 0x0023;
pub const WM_GETMINMAXINFO:     UINT = 0x0024;
pub const WM_PAINTICON:         UINT = 0x0026;
pub const WM_ICONERASEBKGND:    UINT = 0x0027;
pub const WM_NEXTDLGCTL:        UINT = 0x0028;
pub const WM_SPOOLERSTATUS:     UINT = 0x002A;
pub const WM_DRAWITEM:          UINT = 0x002B;
pub const WM_MEASUREITEM:       UINT = 0x002C;
pub const WM_DELETEITEM:        UINT = 0x002D;
pub const WM_VKEYTOITEM:        UINT = 0x002E;
pub const WM_CHARTOITEM:        UINT = 0x002F;
pub const WM_SETFONT:           UINT = 0x0030;
pub const WM_GETFONT:           UINT = 0x0031;
pub const WM_SETHOTKEY:         UINT = 0x0032;
pub const WM_GETHOTKEY:         UINT = 0x0033;
pub const WM_QUERYDRAGICON:     UINT = 0x0037;
pub const WM_COMPAREITEM:       UINT = 0x0039;
pub const WM_GETOBJECT:         UINT = 0x003D;
pub const WM_COMPACTING:        UINT = 0x0041;
pub const WM_WINDOWPOSCHANGING: UINT = 0x0046;
pub const WM_WINDOWPOSCHANGED:  UINT = 0x0047;
pub const WM_COPYDATA:          UINT = 0x004A;
pub const WM_NOTIFY:            UINT = 0x004E;
pub const WM_INPUTLANGCHANGEREQUEST: UINT = 0x0050;
pub const WM_INPUTLANGCHANGE:   UINT = 0x0051;
pub const WM_TCARD:             UINT = 0x0052;
pub const WM_HELP:              UINT = 0x0053;
pub const WM_USERCHANGED:       UINT = 0x0054;
pub const WM_NOTIFYFORMAT:      UINT = 0x0055;
pub const WM_CONTEXTMENU:       UINT = 0x007B;
pub const WM_STYLECHANGING:     UINT = 0x007C;
pub const WM_STYLECHANGED:      UINT = 0x007D;
pub const WM_DISPLAYCHANGE:     UINT = 0x007E;
pub const WM_GETICON:           UINT = 0x007F;
pub const WM_SETICON:           UINT = 0x0080;
pub const WM_NCCREATE:          UINT = 0x0081;
pub const WM_NCDESTROY:         UINT = 0x0082;
pub const WM_NCCALCSIZE:        UINT = 0x0083;
pub const WM_NCHITTEST:         UINT = 0x0084;
pub const WM_NCPAINT:           UINT = 0x0085;
pub const WM_NCACTIVATE:        UINT = 0x0086;
pub const WM_GETDLGCODE:        UINT = 0x0087;
pub const WM_SYNCPAINT:         UINT = 0x0088;
pub const WM_NCMOUSEMOVE:       UINT = 0x00A0;
pub const WM_NCLBUTTONDOWN:     UINT = 0x00A1;
pub const WM_NCLBUTTONUP:       UINT = 0x00A2;
pub const WM_NCLBUTTONDBLCLK:   UINT = 0x00A3;
pub const WM_NCRBUTTONDOWN:     UINT = 0x00A4;
pub const WM_NCRBUTTONUP:       UINT = 0x00A5;
pub const WM_NCRBUTTONDBLCLK:   UINT = 0x00A6;
pub const WM_NCMBUTTONDOWN:     UINT = 0x00A7;
pub const WM_NCMBUTTONUP:       UINT = 0x00A8;
pub const WM_NCMBUTTONDBLCLK:   UINT = 0x00A9;
pub const WM_KEYDOWN:           UINT = 0x0100;
pub const WM_KEYUP:             UINT = 0x0101;
pub const WM_CHAR:              UINT = 0x0102;
pub const WM_DEADCHAR:          UINT = 0x0103;
pub const WM_SYSKEYDOWN:        UINT = 0x0104;
pub const WM_SYSKEYUP:          UINT = 0x0105;
pub const WM_SYSCHAR:           UINT = 0x0106;
pub const WM_SYSDEADCHAR:       UINT = 0x0107;
pub const WM_UNICHAR:           UINT = 0x0109;
pub const WM_IME_STARTCOMPOSITION: UINT = 0x010D;
pub const WM_IME_ENDCOMPOSITION:   UINT = 0x010E;
pub const WM_IME_COMPOSITION:      UINT = 0x010F;
pub const WM_INITDIALOG:        UINT = 0x0110;
pub const WM_COMMAND:           UINT = 0x0111;
pub const WM_SYSCOMMAND:        UINT = 0x0112;
pub const WM_TIMER:             UINT = 0x0113;
pub const WM_HSCROLL:           UINT = 0x0114;
pub const WM_VSCROLL:           UINT = 0x0115;
pub const WM_INITMENU:          UINT = 0x0116;
pub const WM_INITMENUPOPUP:     UINT = 0x0117;
pub const WM_GESTURE:           UINT = 0x0119;
pub const WM_GESTURENOTIFY:     UINT = 0x011A;
pub const WM_MENUSELECT:        UINT = 0x011F;
pub const WM_MENUCHAR:          UINT = 0x0120;
pub const WM_ENTERIDLE:         UINT = 0x0121;
pub const WM_MENURBUTTONUP:     UINT = 0x0122;
pub const WM_MENUDRAG:          UINT = 0x0123;
pub const WM_MENUGETOBJECT:     UINT = 0x0124;
pub const WM_UNINITMENUPOPUP:   UINT = 0x0125;
pub const WM_MENUCOMMAND:       UINT = 0x0126;
pub const WM_CHANGEUISTATE:     UINT = 0x0127;
pub const WM_UPDATEUISTATE:     UINT = 0x0128;
pub const WM_QUERYUISTATE:      UINT = 0x0129;
pub const WM_CTLCOLORMSGBOX:    UINT = 0x0132;
pub const WM_CTLCOLOREDIT:      UINT = 0x0133;
pub const WM_CTLCOLORLISTBOX:   UINT = 0x0134;
pub const WM_CTLCOLORBTN:       UINT = 0x0135;
pub const WM_CTLCOLORDLG:       UINT = 0x0136;
pub const WM_CTLCOLORSCROLLBAR: UINT = 0x0137;
pub const WM_CTLCOLORSTATIC:    UINT = 0x0138;
pub const WM_MOUSEFIRST:        UINT = 0x0200;
pub const WM_MOUSEMOVE:         UINT = 0x0200;
pub const WM_LBUTTONDOWN:       UINT = 0x0201;
pub const WM_LBUTTONUP:         UINT = 0x0202;
pub const WM_LBUTTONDBLCLK:     UINT = 0x0203;
pub const WM_RBUTTONDOWN:       UINT = 0x0204;
pub const WM_RBUTTONUP:         UINT = 0x0205;
pub const WM_RBUTTONDBLCLK:     UINT = 0x0206;
pub const WM_MBUTTONDOWN:       UINT = 0x0207;
pub const WM_MBUTTONUP:         UINT = 0x0208;
pub const WM_MBUTTONDBLCLK:     UINT = 0x0209;
pub const WM_MOUSEWHEEL:        UINT = 0x020A;
pub const WM_XBUTTONDOWN:       UINT = 0x020B;
pub const WM_XBUTTONUP:         UINT = 0x020C;
pub const WM_XBUTTONDBLCLK:     UINT = 0x020D;
pub const WM_MOUSEHWHEEL:       UINT = 0x020E;
pub const WM_MOUSELAST:         UINT = 0x020E;
pub const WM_PARENTNOTIFY:      UINT = 0x0210;
pub const WM_ENTERMENULOOP:     UINT = 0x0211;
pub const WM_EXITMENULOOP:      UINT = 0x0212;
pub const WM_NEXTMENU:          UINT = 0x0213;
pub const WM_SIZING:            UINT = 0x0214;
pub const WM_CAPTURECHANGED:    UINT = 0x0215;
pub const WM_MOVING:            UINT = 0x0216;
pub const WM_POWERBROADCAST:    UINT = 0x0218;
pub const WM_DEVICECHANGE:      UINT = 0x0219;
pub const WM_MDICREATE:         UINT = 0x0220;
pub const WM_MDIDESTROY:        UINT = 0x0221;
pub const WM_MDIACTIVATE:       UINT = 0x0222;
pub const WM_MDIRESTORE:        UINT = 0x0223;
pub const WM_MDINEXT:           UINT = 0x0224;
pub const WM_MDIMAXIMIZE:       UINT = 0x0225;
pub const WM_MDITILE:           UINT = 0x0226;
pub const WM_MDICASCADE:        UINT = 0x0227;
pub const WM_MDIICONARRANGE:    UINT = 0x0228;
pub const WM_MDIGETACTIVE:      UINT = 0x0229;
pub const WM_MDISETMENU:        UINT = 0x0230;
pub const WM_ENTERSIZEMOVE:     UINT = 0x0231;
pub const WM_EXITSIZEMOVE:      UINT = 0x0232;
pub const WM_DROPFILES:         UINT = 0x0233;
pub const WM_MDIREFRESHMENU:    UINT = 0x0234;
pub const WM_IME_SETCONTEXT:    UINT = 0x0281;
pub const WM_IME_NOTIFY:        UINT = 0x0282;
pub const WM_IME_CONTROL:       UINT = 0x0283;
pub const WM_IME_COMPOSITIONFULL: UINT = 0x0284;
pub const WM_IME_SELECT:        UINT = 0x0285;
pub const WM_IME_CHAR:          UINT = 0x0286;
pub const WM_IME_REQUEST:       UINT = 0x0288;
pub const WM_IME_KEYDOWN:       UINT = 0x0290;
pub const WM_IME_KEYUP:         UINT = 0x0291;
pub const WM_MOUSEHOVER:        UINT = 0x02A1;
pub const WM_MOUSELEAVE:        UINT = 0x02A3;
pub const WM_NCMOUSEHOVER:      UINT = 0x02A0;
pub const WM_NCMOUSELEAVE:      UINT = 0x02A2;
pub const WM_WTSSESSION_CHANGE: UINT = 0x02B1;
pub const WM_CUT:               UINT = 0x0300;
pub const WM_COPY:              UINT = 0x0301;
pub const WM_PASTE:             UINT = 0x0302;
pub const WM_CLEAR:             UINT = 0x0303;
pub const WM_UNDO:              UINT = 0x0304;
pub const WM_RENDERFORMAT:      UINT = 0x0305;
pub const WM_RENDERALLFORMATS:  UINT = 0x0306;
pub const WM_DESTROYCLIPBOARD:  UINT = 0x0307;
pub const WM_DRAWCLIPBOARD:     UINT = 0x0308;
pub const WM_PAINTCLIPBOARD:    UINT = 0x0309;
pub const WM_VSCROLLCLIPBOARD:  UINT = 0x030A;
pub const WM_SIZECLIPBOARD:     UINT = 0x030B;
pub const WM_ASKCBFORMATNAME:   UINT = 0x030C;
pub const WM_CHANGECBCHAIN:     UINT = 0x030D;
pub const WM_HSCROLLCLIPBOARD:  UINT = 0x030E;
pub const WM_QUERYNEWPALETTE:   UINT = 0x030F;
pub const WM_PALETTEISCHANGING: UINT = 0x0310;
pub const WM_PALETTECHANGED:    UINT = 0x0311;
pub const WM_HOTKEY:            UINT = 0x0312;
pub const WM_PRINT:             UINT = 0x0317;
pub const WM_PRINTCLIENT:       UINT = 0x0318;
pub const WM_APPCOMMAND:        UINT = 0x0319;
pub const WM_THEMECHANGED:      UINT = 0x031A;
pub const WM_CLIPBOARDUPDATE:   UINT = 0x031D;
pub const WM_DWMCOMPOSITIONCHANGED:    UINT = 0x031E;
pub const WM_DWMNCRENDERINGCHANGED:    UINT = 0x031F;
pub const WM_DWMCOLORIZATIONCOLORCHANGED: UINT = 0x0320;
pub const WM_DWMWINDOWMAXIMIZEDCHANGE: UINT = 0x0321;
pub const WM_USER:              UINT = 0x0400;
pub const WM_APP:               UINT = 0x8000;

// ── GetWindowLong / SetWindowLong indeksleri ──────────────────────────────
pub const GWL_WNDPROC:    INT = -4;
pub const GWL_HINSTANCE:  INT = -6;
pub const GWL_HWNDPARENT: INT = -8;
pub const GWL_STYLE:      INT = -16;
pub const GWL_EXSTYLE:    INT = -20;
pub const GWL_USERDATA:   INT = -21;
pub const GWL_ID:         INT = -12;
pub const GWLP_WNDPROC:   INT = -4;
pub const GWLP_HINSTANCE: INT = -6;
pub const GWLP_HWNDPARENT:INT = -8;
pub const GWLP_USERDATA:  INT = -21;
pub const GWLP_ID:        INT = -12;
pub const DWL_MSGRESULT:  INT = 0;
pub const DWL_DLGPROC:    INT = 4;
pub const DWL_USER:       INT = 8;
pub const DWLP_MSGRESULT: INT = 0;

// ── PeekMessage bayrakları ─────────────────────────────────────────────────
pub const PM_NOREMOVE: UINT = 0x0000;
pub const PM_REMOVE:   UINT = 0x0001;
pub const PM_NOYIELD:  UINT = 0x0002;

// ── Hook türleri ───────────────────────────────────────────────────────────
pub const WH_MSGFILTER:      INT = -1;
pub const WH_JOURNALRECORD:  INT = 0;
pub const WH_JOURNALPLAYBACK:INT = 1;
pub const WH_KEYBOARD:       INT = 2;
pub const WH_GETMESSAGE:     INT = 3;
pub const WH_CALLWNDPROC:    INT = 4;
pub const WH_CBT:            INT = 5;
pub const WH_SYSMSGFILTER:   INT = 6;
pub const WH_MOUSE:          INT = 7;
pub const WH_DEBUG:          INT = 9;
pub const WH_SHELL:          INT = 10;
pub const WH_FOREGROUNDIDLE: INT = 11;
pub const WH_CALLWNDPROCRET: INT = 12;
pub const WH_KEYBOARD_LL:    INT = 13;
pub const WH_MOUSE_LL:       INT = 14;

// ── Pano biçimleri ─────────────────────────────────────────────────────────
pub const CF_TEXT:         UINT = 1;
pub const CF_BITMAP:       UINT = 2;
pub const CF_METAFILEPICT: UINT = 3;
pub const CF_SYLK:         UINT = 4;
pub const CF_DIF:          UINT = 5;
pub const CF_TIFF:         UINT = 6;
pub const CF_OEMTEXT:      UINT = 7;
pub const CF_DIB:          UINT = 8;
pub const CF_PALETTE:      UINT = 9;
pub const CF_PENDATA:      UINT = 10;
pub const CF_RIFF:         UINT = 11;
pub const CF_WAVE:         UINT = 12;
pub const CF_UNICODETEXT:  UINT = 13;
pub const CF_ENHMETAFILE:  UINT = 14;
pub const CF_HDROP:        UINT = 15;
pub const CF_LOCALE:       UINT = 16;
pub const CF_DIBV5:        UINT = 17;
pub const CF_OWNERDISPLAY: UINT = 0x0080;
pub const CF_DSPTEXT:      UINT = 0x0081;
pub const CF_DSPBITMAP:    UINT = 0x0082;
pub const CF_DSPMETAFILEPICT: UINT = 0x0083;
pub const CF_DSPENHMETAFILE:  UINT = 0x008E;

// ── Sistem metriği sabitleri (GetSystemMetrics) ────────────────────────────
pub const SM_CXSCREEN:               INT = 0;
pub const SM_CYSCREEN:               INT = 1;
pub const SM_CXVSCROLL:              INT = 2;
pub const SM_CYHSCROLL:              INT = 3;
pub const SM_CYCAPTION:              INT = 4;
pub const SM_CXBORDER:               INT = 5;
pub const SM_CYBORDER:               INT = 6;
pub const SM_CXDLGFRAME:             INT = 7;
pub const SM_CYDLGFRAME:             INT = 8;
pub const SM_CYVTHUMB:               INT = 9;
pub const SM_CXHTHUMB:               INT = 10;
pub const SM_CXICON:                 INT = 11;
pub const SM_CYICON:                 INT = 12;
pub const SM_CXCURSOR:               INT = 13;
pub const SM_CYCURSOR:               INT = 14;
pub const SM_CYMENU:                 INT = 15;
pub const SM_CXFULLSCREEN:           INT = 16;
pub const SM_CYFULLSCREEN:           INT = 17;
pub const SM_CYKANJIWINDOW:          INT = 18;
pub const SM_MOUSEPRESENT:           INT = 19;
pub const SM_CYVSCROLL:              INT = 20;
pub const SM_CXHSCROLL:              INT = 21;
pub const SM_DEBUG:                  INT = 22;
pub const SM_SWAPBUTTON:             INT = 23;
pub const SM_CXMIN:                  INT = 28;
pub const SM_CYMIN:                  INT = 29;
pub const SM_CXSIZE:                 INT = 30;
pub const SM_CYSIZE:                 INT = 31;
pub const SM_CXFRAME:                INT = 32;
pub const SM_CYFRAME:                INT = 33;
pub const SM_CXMINTRACK:             INT = 34;
pub const SM_CYMINTRACK:             INT = 35;
pub const SM_CXDOUBLECLK:           INT = 36;
pub const SM_CYDOUBLECLK:           INT = 37;
pub const SM_CXICONSPACING:          INT = 38;
pub const SM_CYICONSPACING:          INT = 39;
pub const SM_MENUDROPALIGNMENT:      INT = 40;
pub const SM_PENWINDOWS:             INT = 41;
pub const SM_DBCSENABLED:            INT = 42;
pub const SM_CMOUSEBUTTONS:          INT = 43;
pub const SM_CXFIXEDFRAME:           INT = SM_CXDLGFRAME;
pub const SM_CYFIXEDFRAME:           INT = SM_CYDLGFRAME;
pub const SM_CXSIZEFRAME:            INT = SM_CXFRAME;
pub const SM_CYSIZEFRAME:            INT = SM_CYFRAME;
pub const SM_SECURE:                 INT = 44;
pub const SM_CXEDGE:                 INT = 45;
pub const SM_CYEDGE:                 INT = 46;
pub const SM_CXMINSPACING:           INT = 47;
pub const SM_CYMINSPACING:           INT = 48;
pub const SM_CXSMICON:               INT = 49;
pub const SM_CYSMICON:               INT = 50;
pub const SM_CYSMCAPTION:            INT = 51;
pub const SM_CXSMSIZE:               INT = 52;
pub const SM_CYSMSIZE:               INT = 53;
pub const SM_CXMENUSIZE:             INT = 54;
pub const SM_CYMENUSIZE:             INT = 55;
pub const SM_ARRANGE:                INT = 56;
pub const SM_CXMINIMIZED:            INT = 57;
pub const SM_CYMINIMIZED:            INT = 58;
pub const SM_CXMAXTRACK:             INT = 59;
pub const SM_CYMAXTRACK:             INT = 60;
pub const SM_CXMAXIMIZED:            INT = 61;
pub const SM_CYMAXIMIZED:            INT = 62;
pub const SM_NETWORK:                INT = 63;
pub const SM_CLEANBOOT:              INT = 67;
pub const SM_CXDRAG:                 INT = 68;
pub const SM_CYDRAG:                 INT = 69;
pub const SM_SHOWSOUNDS:             INT = 70;
pub const SM_CXMENUCHECK:            INT = 71;
pub const SM_CYMENUCHECK:            INT = 72;
pub const SM_SLOWMACHINE:            INT = 73;
pub const SM_MIDEASTENABLED:         INT = 74;
pub const SM_MOUSEWHEELPRESENT:      INT = 75;
pub const SM_XVIRTUALSCREEN:         INT = 76;
pub const SM_YVIRTUALSCREEN:         INT = 77;
pub const SM_CXVIRTUALSCREEN:        INT = 78;
pub const SM_CYVIRTUALSCREEN:        INT = 79;
pub const SM_CMONITORS:              INT = 80;
pub const SM_SAMEDISPLAYFORMAT:      INT = 81;
pub const SM_IMMENABLED:             INT = 82;
pub const SM_CXFOCUSBORDER:          INT = 83;
pub const SM_CYFOCUSBORDER:          INT = 84;
pub const SM_TABLETPC:               INT = 86;
pub const SM_MEDIACENTER:            INT = 87;
pub const SM_STARTER:                INT = 88;
pub const SM_SERVERR2:               INT = 89;
pub const SM_MOUSEHORIZONTALWHEELPRESENT: INT = 91;
pub const SM_CXPADDEDBORDER:         INT = 92;
pub const SM_DIGITIZER:              INT = 94;
pub const SM_MAXIMUMTOUCHES:         INT = 95;
pub const SM_REMOTESESSION:          INT = 0x1000;
pub const SM_SHUTTINGDOWN:           INT = 0x2000;
pub const SM_REMOTECONTROL:          INT = 0x2001;
pub const SM_CARETBLINKINGENABLED:   INT = 0x2002;

// ── Yapılar ────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct POINTS {
    pub x: SHORT,
    pub y: SHORT,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RECT {
    pub left:   LONG,
    pub top:    LONG,
    pub right:  LONG,
    pub bottom: LONG,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SIZE {
    pub cx: LONG,
    pub cy: LONG,
}

#[repr(C)]
pub struct MSG {
    pub hwnd:    HWND,
    pub message: UINT,
    pub wParam:  WPARAM,
    pub lParam:  LPARAM,
    pub time:    DWORD,
    pub pt:      POINT,
    pub lPrivate:DWORD,
}

#[repr(C)]
pub struct WNDCLASSA {
    pub style:         UINT,
    pub lpfnWndProc:   WNDPROC,
    pub cbClsExtra:    INT,
    pub cbWndExtra:    INT,
    pub hInstance:     HINSTANCE,
    pub hIcon:         HICON,
    pub hCursor:       HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName:  LPCSTR,
    pub lpszClassName: LPCSTR,
}

#[repr(C)]
pub struct WNDCLASSW {
    pub style:         UINT,
    pub lpfnWndProc:   WNDPROC,
    pub cbClsExtra:    INT,
    pub cbWndExtra:    INT,
    pub hInstance:     HINSTANCE,
    pub hIcon:         HICON,
    pub hCursor:       HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName:  LPCWSTR,
    pub lpszClassName: LPCWSTR,
}

#[repr(C)]
pub struct WNDCLASSEXA {
    pub cbSize:        UINT,
    pub style:         UINT,
    pub lpfnWndProc:   WNDPROC,
    pub cbClsExtra:    INT,
    pub cbWndExtra:    INT,
    pub hInstance:     HINSTANCE,
    pub hIcon:         HICON,
    pub hCursor:       HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName:  LPCSTR,
    pub lpszClassName: LPCSTR,
    pub hIconSm:       HICON,
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize:        UINT,
    pub style:         UINT,
    pub lpfnWndProc:   WNDPROC,
    pub cbClsExtra:    INT,
    pub cbWndExtra:    INT,
    pub hInstance:     HINSTANCE,
    pub hIcon:         HICON,
    pub hCursor:       HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName:  LPCWSTR,
    pub lpszClassName: LPCWSTR,
    pub hIconSm:       HICON,
}

#[repr(C)]
pub struct CREATESTRUCTA {
    pub lpCreateParams: LPVOID,
    pub hInstance:      HINSTANCE,
    pub hMenu:          HMENU,
    pub hwndParent:     HWND,
    pub cy:             INT,
    pub cx:             INT,
    pub y:              INT,
    pub x:              INT,
    pub style:          LONG,
    pub lpszName:       LPCSTR,
    pub lpszClass:      LPCSTR,
    pub dwExStyle:      DWORD,
}

#[repr(C)]
pub struct CREATESTRUCTW {
    pub lpCreateParams: LPVOID,
    pub hInstance:      HINSTANCE,
    pub hMenu:          HMENU,
    pub hwndParent:     HWND,
    pub cy:             INT,
    pub cx:             INT,
    pub y:              INT,
    pub x:              INT,
    pub style:          LONG,
    pub lpszName:       LPCWSTR,
    pub lpszClass:      LPCWSTR,
    pub dwExStyle:      DWORD,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc:          HDC,
    pub fErase:       BOOL,
    pub rcPaint:      RECT,
    pub fRestore:     BOOL,
    pub fIncUpdate:   BOOL,
    pub rgbReserved:  [BYTE; 32],
}

#[repr(C)]
pub struct WINDOWPOS {
    pub hwnd:            HWND,
    pub hwndInsertAfter: HWND,
    pub x:               INT,
    pub y:               INT,
    pub cx:              INT,
    pub cy:              INT,
    pub flags:           UINT,
}

#[repr(C)]
pub struct MINMAXINFO {
    pub ptReserved:     POINT,
    pub ptMaxSize:      POINT,
    pub ptMaxPosition:  POINT,
    pub ptMinTrackSize: POINT,
    pub ptMaxTrackSize: POINT,
}

#[repr(C)]
pub struct COPYDATASTRUCT {
    pub dwData: ULONG_PTR,
    pub cbData: DWORD,
    pub lpData: LPVOID,
}

#[repr(C)]
pub struct SCROLLINFO {
    pub cbSize:  UINT,
    pub fMask:   UINT,
    pub nMin:    INT,
    pub nMax:    INT,
    pub nPage:   UINT,
    pub nPos:    INT,
    pub nTrackPos: INT,
}

#[repr(C)]
pub struct MONITORINFO {
    pub cbSize:    DWORD,
    pub rcMonitor: RECT,
    pub rcWork:    RECT,
    pub dwFlags:   DWORD,
}

#[repr(C)]
pub struct MONITORINFOEXA {
    pub cbSize:    DWORD,
    pub rcMonitor: RECT,
    pub rcWork:    RECT,
    pub dwFlags:   DWORD,
    pub szDevice:  [CHAR; 32],
}

#[repr(C)]
pub struct MONITORINFOEXW {
    pub cbSize:    DWORD,
    pub rcMonitor: RECT,
    pub rcWork:    RECT,
    pub dwFlags:   DWORD,
    pub szDevice:  [WCHAR; 32],
}

#[repr(C)]
pub struct TRACKMOUSEEVENT {
    pub cbSize:      DWORD,
    pub dwFlags:     DWORD,
    pub hwndTrack:   HWND,
    pub dwHoverTime: DWORD,
}

#[repr(C)]
pub struct MOUSEINPUT {
    pub dx:          LONG,
    pub dy:          LONG,
    pub mouseData:   DWORD,
    pub dwFlags:     DWORD,
    pub time:        DWORD,
    pub dwExtraInfo: ULONG_PTR,
}

#[repr(C)]
pub struct KEYBDINPUT {
    pub wVk:         WORD,
    pub wScan:       WORD,
    pub dwFlags:     DWORD,
    pub time:        DWORD,
    pub dwExtraInfo: ULONG_PTR,
}

#[repr(C)]
pub struct HARDWAREINPUT {
    pub uMsg:    DWORD,
    pub wParamL: WORD,
    pub wParamH: WORD,
}

#[repr(C)]
pub union INPUT_UNION {
    pub mi: core::mem::ManuallyDrop<MOUSEINPUT>,
    pub ki: core::mem::ManuallyDrop<KEYBDINPUT>,
    pub hi: core::mem::ManuallyDrop<HARDWAREINPUT>,
}

#[repr(C)]
pub struct INPUT {
    pub r#type: DWORD,
    pub u: INPUT_UNION,
}

#[repr(C)]
pub struct GUITHREADINFO {
    pub cbSize:        DWORD,
    pub flags:         DWORD,
    pub hwndActive:    HWND,
    pub hwndFocus:     HWND,
    pub hwndCapture:   HWND,
    pub hwndMenuOwner: HWND,
    pub hwndMoveSize:  HWND,
    pub hwndCaret:     HWND,
    pub rcCaret:       RECT,
}

#[repr(C)]
pub struct MENUITEMINFOA {
    pub cbSize:        UINT,
    pub fMask:         UINT,
    pub fType:         UINT,
    pub fState:        UINT,
    pub wID:           UINT,
    pub hSubMenu:      HMENU,
    pub hbmpChecked:   HBITMAP,
    pub hbmpUnchecked: HBITMAP,
    pub dwItemData:    ULONG_PTR,
    pub dwTypeData:    LPSTR,
    pub cch:           UINT,
    pub hbmpItem:      HBITMAP,
}

#[repr(C)]
pub struct MENUITEMINFOW {
    pub cbSize:        UINT,
    pub fMask:         UINT,
    pub fType:         UINT,
    pub fState:        UINT,
    pub wID:           UINT,
    pub hSubMenu:      HMENU,
    pub hbmpChecked:   HBITMAP,
    pub hbmpUnchecked: HBITMAP,
    pub dwItemData:    ULONG_PTR,
    pub dwTypeData:    LPWSTR,
    pub cch:           UINT,
    pub hbmpItem:      HBITMAP,
}

#[repr(C)]
pub struct ACCEL {
    pub fVirt: BYTE,
    pub key:   WORD,
    pub cmd:   WORD,
}

#[repr(C)]
pub struct NCCALCSIZE_PARAMS {
    pub rgrc:  [RECT; 3],
    pub lppos: *mut WINDOWPOS,
}

#[repr(C)]
pub struct DRAWITEMSTRUCT {
    pub CtlType:    UINT,
    pub CtlID:      UINT,
    pub itemID:     UINT,
    pub itemAction: UINT,
    pub itemState:  UINT,
    pub hwndItem:   HWND,
    pub hDC:        HDC,
    pub rcItem:     RECT,
    pub itemData:   ULONG_PTR,
}

#[repr(C)]
pub struct HELPINFO {
    pub cbSize:       UINT,
    pub iContextType: INT,
    pub iCtrlId:      INT,
    pub hItemHandle:  HANDLE,
    pub dwContextId:  DWORD_PTR,
    pub MousePos:     POINT,
}

// ── Alt modüller ────────────────────────────────────────────────────────────
#[path = "src/wnd_class.rs"]   pub mod wnd_class;
#[path = "src/wnd_create.rs"]  pub mod wnd_create;
#[path = "src/wnd_message.rs"] pub mod wnd_message;
#[path = "src/wnd_paint.rs"]   pub mod wnd_paint;
#[path = "src/wnd_prop.rs"]    pub mod wnd_prop;
#[path = "src/dialog.rs"]      pub mod dialog;
#[path = "src/input_kbd.rs"]   pub mod input_kbd;
#[path = "src/input_mouse.rs"] pub mod input_mouse;
#[path = "src/resources.rs"]   pub mod resources;
#[path = "src/clipboard.rs"]   pub mod clipboard;
#[path = "src/menu.rs"]        pub mod menu;
#[path = "src/system_param.rs"]pub mod system_param;
#[path = "src/hook.rs"]        pub mod hook;
#[path = "src/monitor.rs"]     pub mod monitor;
