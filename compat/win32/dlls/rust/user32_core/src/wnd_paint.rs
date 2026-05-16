// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : user32.dll — Pencere çizim ve boyama API'leri
//                      (BeginPaint, EndPaint, InvalidateRect, UpdateWindow vb.)
// Dosya Yolu         : compat/win32/dlls/rust/user32_core/src/wnd_paint.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Windows pencere boyama ve DC fonksiyonlarının clean-room Rust
//   implementasyonu. BeginPaint, EndPaint, InvalidateRect,
//   InvalidateRgn, ValidateRect, ValidateRgn, UpdateWindow,
//   RedrawWindow, GetUpdateRect, GetUpdateRgn, ExcludeUpdateRgn,
//   GetDC, GetDCEx, ReleaseDC, GetWindowDC, DrawText, DrawTextEx,
//   TabbedTextOut, FillRect, FrameRect, InvertRect,
//   DrawFocusRect, DrawEdge, DrawFrameControl, DrawCaption,
//   DrawState, GrayString, LockWindowUpdate API'lerini içerir.
//   MSDN public spesifikasyonuna dayanır; stub implementasyon.
//
// Bağımlı Dosyalar:
//   1-) user32_root.rs (üst crate — HDC, HWND, RECT, PAINTSTRUCT)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 83
// *******************************************************************

#![allow(dead_code)]

use crate::{
    BOOL, DWORD, DRAWSTATEPROC, FALSE, GRAYSTRINGPROC,
    HBRUSH, HDC, HICON, HRGN, HWND, INT, LONG, LPARAM,
    LPCSTR, LPCWSTR, LPSTR, LPWSTR, PAINTSTRUCT, RECT, TRUE, UINT,
    WPARAM,
};

// DrawText biçimlendirme bayrakları (Win32 public API)
pub const DT_TOP:                UINT = 0x0000_0000;
pub const DT_LEFT:               UINT = 0x0000_0000;
pub const DT_CENTER:             UINT = 0x0000_0001;
pub const DT_RIGHT:              UINT = 0x0000_0002;
pub const DT_VCENTER:            UINT = 0x0000_0004;
pub const DT_BOTTOM:             UINT = 0x0000_0008;
pub const DT_WORDBREAK:          UINT = 0x0000_0010;
pub const DT_SINGLELINE:         UINT = 0x0000_0020;
pub const DT_EXPANDTABS:         UINT = 0x0000_0040;
pub const DT_TABSTOP:            UINT = 0x0000_0080;
pub const DT_NOCLIP:             UINT = 0x0000_0100;
pub const DT_EXTERNALLEADING:    UINT = 0x0000_0200;
pub const DT_CALCRECT:           UINT = 0x0000_0400;
pub const DT_NOPREFIX:           UINT = 0x0000_0800;
pub const DT_INTERNAL:           UINT = 0x0000_1000;
pub const DT_EDITCONTROL:        UINT = 0x0000_2000;
pub const DT_PATH_ELLIPSIS:      UINT = 0x0000_4000;
pub const DT_END_ELLIPSIS:       UINT = 0x0000_8000;
pub const DT_MODIFYSTRING:       UINT = 0x0001_0000;
pub const DT_RTLREADING:         UINT = 0x0002_0000;
pub const DT_WORD_ELLIPSIS:      UINT = 0x0004_0000;
pub const DT_NOFULLWIDTHCHARBREAK:UINT = 0x0008_0000;
pub const DT_HIDEPREFIX:         UINT = 0x0010_0000;
pub const DT_PREFIXONLY:         UINT = 0x0020_0000;

// DrawEdge sabitleri
pub const BDR_RAISEDOUTER: UINT = 0x0001;
pub const BDR_SUNKENOUTER: UINT = 0x0002;
pub const BDR_RAISEDINNER: UINT = 0x0004;
pub const BDR_SUNKENINNER: UINT = 0x0008;
pub const BDR_OUTER:       UINT = BDR_RAISEDOUTER | BDR_SUNKENOUTER;
pub const BDR_INNER:       UINT = BDR_RAISEDINNER | BDR_SUNKENINNER;
pub const BDR_RAISED:      UINT = BDR_RAISEDOUTER | BDR_RAISEDINNER;
pub const BDR_SUNKEN:      UINT = BDR_SUNKENOUTER | BDR_SUNKENINNER;
pub const EDGE_RAISED:     UINT = BDR_RAISEDOUTER | BDR_RAISEDINNER;
pub const EDGE_SUNKEN:     UINT = BDR_SUNKENOUTER | BDR_SUNKENINNER;
pub const EDGE_ETCHED:     UINT = BDR_SUNKENOUTER | BDR_RAISEDINNER;
pub const EDGE_BUMP:       UINT = BDR_RAISEDOUTER | BDR_SUNKENINNER;
pub const BF_LEFT:   UINT = 0x0001;
pub const BF_TOP:    UINT = 0x0002;
pub const BF_RIGHT:  UINT = 0x0004;
pub const BF_BOTTOM: UINT = 0x0008;
pub const BF_RECT:   UINT = BF_LEFT | BF_TOP | BF_RIGHT | BF_BOTTOM;
pub const BF_DIAGONAL: UINT = 0x0010;
pub const BF_MIDDLE:   UINT = 0x0800;
pub const BF_SOFT:     UINT = 0x1000;
pub const BF_ADJUST:   UINT = 0x2000;
pub const BF_FLAT:     UINT = 0x4000;
pub const BF_MONO:     UINT = 0x8000;

// RedrawWindow bayrakları
pub const RDW_INVALIDATE:      UINT = 0x0001;
pub const RDW_INTERNALPAINT:   UINT = 0x0002;
pub const RDW_ERASE:           UINT = 0x0004;
pub const RDW_VALIDATE:        UINT = 0x0008;
pub const RDW_NOINTERNALPAINT: UINT = 0x0010;
pub const RDW_NOERASE:         UINT = 0x0020;
pub const RDW_NOCHILDREN:      UINT = 0x0040;
pub const RDW_ALLCHILDREN:     UINT = 0x0080;
pub const RDW_UPDATENOW:       UINT = 0x0100;
pub const RDW_ERASENOW:        UINT = 0x0200;
pub const RDW_FRAME:           UINT = 0x0400;
pub const RDW_NOFRAME:         UINT = 0x0800;

// GetDCEx bayrakları
pub const DCX_WINDOW:         DWORD = 0x0000_0001;
pub const DCX_CACHE:          DWORD = 0x0000_0002;
pub const DCX_NORESETATTRS:   DWORD = 0x0000_0004;
pub const DCX_CLIPCHILDREN:   DWORD = 0x0000_0008;
pub const DCX_CLIPSIBLINGS:   DWORD = 0x0000_0010;
pub const DCX_PARENTCLIP:     DWORD = 0x0000_0020;
pub const DCX_EXCLUDERGN:     DWORD = 0x0000_0040;
pub const DCX_INTERSECTRGN:   DWORD = 0x0000_0080;
pub const DCX_EXCLUDEUPDATE:  DWORD = 0x0000_0100;
pub const DCX_INTERSECTUPDATE:DWORD = 0x0000_0200;
pub const DCX_LOCKWINDOWUPDATE:DWORD= 0x0000_0400;
pub const DCX_VALIDATE:       DWORD = 0x0020_0000;

/// DRAWTEXTPARAMS yapısı — DrawTextEx için
#[repr(C)]
pub struct DRAWTEXTPARAMS {
    pub cb_size:        UINT,
    pub i_tab_length:   INT,
    pub i_left_margin:  INT,
    pub i_right_margin: INT,
    pub u_length_drawn: UINT,
}

/// Boyama başlat, DC al.
/// Win32: BeginPaint (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn BeginPaint(
    _hwnd:          HWND,
    lp_paint:       *mut PAINTSTRUCT,
) -> HDC {
    if lp_paint.is_null() { return core::ptr::null_mut(); }
    unsafe {
        (*lp_paint).hdc = core::ptr::null_mut();
        (*lp_paint).fErase = FALSE;
        (*lp_paint).rcPaint = crate::RECT { left: 0, top: 0, right: 0, bottom: 0 };
    }
    core::ptr::null_mut()
}

/// Boyamayı bitir, DC serbest bırak.
/// Win32: EndPaint (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn EndPaint(
    _hwnd:       HWND,
    _lp_paint:   *const PAINTSTRUCT,
) -> BOOL { TRUE }

/// Dikdörtgeni geçersiz kıl (yeniden boyama iste).
/// Win32: InvalidateRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn InvalidateRect(
    _hwnd:     HWND,
    _lp_rect:  *const RECT,
    _b_erase:  BOOL,
) -> BOOL { TRUE }

/// Bölgeyi geçersiz kıl.
/// Win32: InvalidateRgn (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn InvalidateRgn(
    _hwnd:    HWND,
    _h_rgn:   HRGN,
    _b_erase: BOOL,
) -> BOOL { TRUE }

/// Dikdörtgeni geçerli kıl (boyama iptal).
/// Win32: ValidateRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ValidateRect(
    _hwnd:    HWND,
    _lp_rect: *const RECT,
) -> BOOL { TRUE }

/// Bölgeyi geçerli kıl.
/// Win32: ValidateRgn (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ValidateRgn(_hwnd: HWND, _h_rgn: HRGN) -> BOOL { TRUE }

/// Pencereyi hemen güncelle.
/// Win32: UpdateWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn UpdateWindow(_hwnd: HWND) -> BOOL { TRUE }

/// Pencereyi yeniden çiz.
/// Win32: RedrawWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn RedrawWindow(
    _hwnd:        HWND,
    _lp_rect_update: *const RECT,
    _h_rgn_update:   HRGN,
    _flags:          UINT,
) -> BOOL { TRUE }

/// Güncelleme dikdörtgenini al.
/// Win32: GetUpdateRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetUpdateRect(
    _hwnd:    HWND,
    lp_rect:  *mut RECT,
    _b_erase: BOOL,
) -> BOOL {
    if !lp_rect.is_null() {
        unsafe { *lp_rect = crate::RECT { left: 0, top: 0, right: 0, bottom: 0 }; }
    }
    FALSE
}

/// Güncelleme bölgesini al.
/// Win32: GetUpdateRgn (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetUpdateRgn(
    _hwnd:    HWND,
    _h_rgn:   HRGN,
    _b_erase: BOOL,
) -> INT { 1 /* NULLREGION */ }

/// Güncelleme bölgesini DC'den çıkar.
/// Win32: ExcludeUpdateRgn (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ExcludeUpdateRgn(_hdc: HDC, _hwnd: HWND) -> INT { 1 }

/// Pencere DC'sini al.
/// Win32: GetDC (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetDC(_hwnd: HWND) -> HDC { core::ptr::null_mut() }

/// Genişletilmiş pencere DC'si al.
/// Win32: GetDCEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetDCEx(
    _hwnd:    HWND,
    _h_rgn_clip: HRGN,
    _flags:   DWORD,
) -> HDC { core::ptr::null_mut() }

/// DC'yi serbest bırak.
/// Win32: ReleaseDC (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ReleaseDC(_hwnd: HWND, _hdc: HDC) -> INT { 1 }

/// Pencere çerçevesi dahil DC al.
/// Win32: GetWindowDC (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetWindowDC(_hwnd: HWND) -> HDC { core::ptr::null_mut() }

/// ANSI metin çiz.
/// Win32: DrawTextA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawTextA(
    _hdc:     HDC,
    lp_ch_text: LPCSTR,
    _c_ch_text: INT,
    lp_rect:    *mut RECT,
    _format:    UINT,
) -> INT {
    if lp_ch_text.is_null() || lp_rect.is_null() { return 0; }
    0
}

/// Unicode metin çiz.
/// Win32: DrawTextW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawTextW(
    _hdc:       HDC,
    lp_ch_text: LPCWSTR,
    _c_ch_text: INT,
    lp_rect:    *mut RECT,
    _format:    UINT,
) -> INT {
    if lp_ch_text.is_null() || lp_rect.is_null() { return 0; }
    0
}

/// Gelişmiş ANSI metin çiz.
/// Win32: DrawTextExA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawTextExA(
    _hdc:       HDC,
    _lp_ch_text:LPSTR,
    _cch_text:  INT,
    lp_rect:    *mut RECT,
    _dw_dt_fmt: UINT,
    _lp_dt_params: *const DRAWTEXTPARAMS,
) -> INT {
    if lp_rect.is_null() { return 0; }
    0
}

/// Gelişmiş Unicode metin çiz.
/// Win32: DrawTextExW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawTextExW(
    _hdc:          HDC,
    _lp_ch_text:   LPWSTR,
    _cch_text:     INT,
    lp_rect:       *mut RECT,
    _dw_dt_fmt:    UINT,
    _lp_dt_params: *const DRAWTEXTPARAMS,
) -> INT {
    if lp_rect.is_null() { return 0; }
    0
}

/// Dikdörtgeni doldur.
/// Win32: FillRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FillRect(
    _hdc:    HDC,
    lp_rc:   *const RECT,
    _h_brush:HBRUSH,
) -> INT {
    if lp_rc.is_null() { return 0; }
    1
}

/// Dikdörtgen çerçevesi çiz.
/// Win32: FrameRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FrameRect(
    _hdc:    HDC,
    lp_rc:   *const RECT,
    _h_brush:HBRUSH,
) -> INT {
    if lp_rc.is_null() { return 0; }
    1
}

/// Dikdörtgeni ters çevir (XOR).
/// Win32: InvertRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn InvertRect(
    _hdc:  HDC,
    lp_rc: *const RECT,
) -> BOOL {
    if lp_rc.is_null() { return FALSE; }
    TRUE
}

/// Odak dikdörtgeni çiz.
/// Win32: DrawFocusRect (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawFocusRect(
    _hdc:  HDC,
    lp_rc: *const RECT,
) -> BOOL {
    if lp_rc.is_null() { return FALSE; }
    TRUE
}

/// 3D kenar çiz.
/// Win32: DrawEdge (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawEdge(
    _hdc:       HDC,
    qrc:        *mut RECT,
    _edge:      UINT,
    _grfFlags:  UINT,
) -> BOOL {
    if qrc.is_null() { return FALSE; }
    TRUE
}

/// Çerçeve kontrolü çiz.
/// Win32: DrawFrameControl (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawFrameControl(
    _hdc:    HDC,
    lp_rect: *const RECT,
    _u_type: UINT,
    _u_state:UINT,
) -> BOOL {
    if lp_rect.is_null() { return FALSE; }
    TRUE
}

/// Pencere başlık çubuğu çiz.
/// Win32: DrawCaption (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawCaption(
    _hwnd:   HWND,
    _hdc:    HDC,
    lp_rect: *const RECT,
    _u_flags:UINT,
) -> BOOL {
    if lp_rect.is_null() { return FALSE; }
    TRUE
}

/// DrawState ile renk/resim/metin çiz.
/// Win32: DrawStateA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawStateA(
    _hdc:       HDC,
    _h_brush:   HBRUSH,
    _lp_output_func: DRAWSTATEPROC,
    _l_data:    LPARAM,
    _w_data:    WPARAM,
    _x:         INT,
    _y:         INT,
    _cx:        INT,
    _cy:        INT,
    _fu_flags:  UINT,
) -> BOOL { TRUE }

/// Win32: DrawStateW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawStateW(
    _hdc:            HDC,
    _h_brush:        HBRUSH,
    _lp_output_func: DRAWSTATEPROC,
    _l_data:         LPARAM,
    _w_data:         WPARAM,
    _x:              INT,
    _y:              INT,
    _cx:             INT,
    _cy:             INT,
    _fu_flags:       UINT,
) -> BOOL { TRUE }

/// Soluk ANSI metin çiz.
/// Win32: GrayStringA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GrayStringA(
    _hdc:          HDC,
    _h_brush:      HBRUSH,
    _lp_output_func: GRAYSTRINGPROC,
    _lp_data:      LPARAM,
    _n_count:      INT,
    _x:            INT,
    _y:            INT,
    _n_width:      INT,
    _n_height:     INT,
) -> BOOL { TRUE }

/// Soluk Unicode metin çiz.
/// Win32: GrayStringW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GrayStringW(
    _hdc:            HDC,
    _h_brush:        HBRUSH,
    _lp_output_func: GRAYSTRINGPROC,
    _lp_data:        LPARAM,
    _n_count:        INT,
    _x:              INT,
    _y:              INT,
    _n_width:        INT,
    _n_height:       INT,
) -> BOOL { TRUE }

/// Tüm boyamaları kilitle.
/// Win32: LockWindowUpdate (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn LockWindowUpdate(_hwnd_lock: HWND) -> BOOL { TRUE }

/// İkon çiz.
/// Win32: DrawIcon (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawIcon(
    _hdc:    HDC,
    _x:      INT,
    _y:      INT,
    _h_icon: HICON,
) -> BOOL { TRUE }

/// İkonu gelişmiş seçeneklerle çiz.
/// Win32: DrawIconEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DrawIconEx(
    _hdc:          HDC,
    _x_left:       INT,
    _y_top:        INT,
    _h_icon:       HICON,
    _cx_width:     INT,
    _cy_width:     INT,
    _i_step_if_anim_cursor: UINT,
    _h_brush_flicker: HBRUSH,
    _flags:        UINT,
) -> BOOL { TRUE }

/// Sekme karakterleri de dahil ANSI metin çiz.
/// Win32: TabbedTextOutA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn TabbedTextOutA(
    _hdc:              HDC,
    _x:                INT,
    _y:                INT,
    _lp_string:        LPCSTR,
    _n_count:          INT,
    _n_tab_positions:  INT,
    _lp_n_tab_stop_positions: *const INT,
    _n_tab_origin:     INT,
) -> LONG { 0 }

/// Win32: TabbedTextOutW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn TabbedTextOutW(
    _hdc:                    HDC,
    _x:                      INT,
    _y:                      INT,
    _lp_string:              LPCWSTR,
    _n_count:                INT,
    _n_tab_positions:        INT,
    _lp_n_tab_stop_positions:*const INT,
    _n_tab_origin:           INT,
) -> LONG { 0 }

/// Scrollbar'ı çiz.
/// Win32: DrawScrollBar (USER32.@) — iç kullanım
#[no_mangle]
pub unsafe extern "system" fn DrawScrollBar(
    _hwnd:   HWND,
    _hdc:    HDC,
    _b_vert: BOOL,
    _b_arrows: BOOL,
) {}

/// Çizim DC'si için clip bölgesi al.
/// Win32: GetClipBox (USER32.@) — USER32'deki sarmalayıcı
#[no_mangle]
pub unsafe extern "system" fn GetClipBox_USER(
    _hdc:    HDC,
    lp_rect: *mut RECT,
) -> INT {
    if lp_rect.is_null() { return 0; /* ERROR */ }
    unsafe { *lp_rect = crate::RECT { left: 0, top: 0, right: 0, bottom: 0 }; }
    1 /* NULLREGION */
}

/// Kaydırma çubuğu bilgisini al.
/// Win32: GetScrollInfo (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetScrollInfo(
    _hwnd:     HWND,
    _n_bar:    INT,
    lp_si:     *mut crate::SCROLLINFO,
) -> BOOL {
    if lp_si.is_null() { return FALSE; }
    FALSE
}

/// Kaydırma çubuğu bilgisini ayarla.
/// Win32: SetScrollInfo (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetScrollInfo(
    _hwnd:    HWND,
    _n_bar:   INT,
    lp_si:    *const crate::SCROLLINFO,
    _b_redraw:BOOL,
) -> INT {
    if lp_si.is_null() { return 0; }
    0
}

/// Kaydırma pozisyonunu al.
/// Win32: GetScrollPos (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetScrollPos(_hwnd: HWND, _n_bar: INT) -> INT { 0 }

/// Kaydırma pozisyonunu ayarla.
/// Win32: SetScrollPos (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetScrollPos(
    _hwnd:     HWND,
    _n_bar:    INT,
    _n_pos:    INT,
    _b_redraw: BOOL,
) -> INT { 0 }

/// Kaydırma aralığını al.
/// Win32: GetScrollRange (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn GetScrollRange(
    _hwnd:       HWND,
    _n_bar:      INT,
    lp_min_pos:  *mut INT,
    lp_max_pos:  *mut INT,
) -> BOOL {
    if !lp_min_pos.is_null() { unsafe { *lp_min_pos = 0; } }
    if !lp_max_pos.is_null() { unsafe { *lp_max_pos = 0; } }
    TRUE
}

/// Kaydırma aralığını ayarla.
/// Win32: SetScrollRange (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn SetScrollRange(
    _hwnd:    HWND,
    _n_bar:   INT,
    _n_min_pos:INT,
    _n_max_pos:INT,
    _b_redraw: BOOL,
) -> BOOL { TRUE }

/// Kaydırma çubuğunu göster / gizle.
/// Win32: ShowScrollBar (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ShowScrollBar(
    _hwnd:  HWND,
    _w_bar: UINT,
    _b_show:BOOL,
) -> BOOL { TRUE }

/// Pencereyi kaydır.
/// Win32: ScrollWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ScrollWindow(
    _hwnd:          HWND,
    _x_amount:      INT,
    _y_amount:      INT,
    _lp_rect:       *const RECT,
    _lp_clip_rect:  *const RECT,
) -> BOOL { TRUE }

/// Gelişmiş pencere kaydırma.
/// Win32: ScrollWindowEx (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ScrollWindowEx(
    _hwnd:         HWND,
    _dx:           INT,
    _dy:           INT,
    _prc_scroll:   *const RECT,
    _prc_clip:     *const RECT,
    _h_rgn_update: HRGN,
    _prc_update:   *mut RECT,
    _flags:        UINT,
) -> INT { 0 /* NULLREGION */ }
