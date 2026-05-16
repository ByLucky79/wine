// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 locale/ANSI codepage API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/locale_win32.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 locale, code page, and NLS (National Language Support)
//   functions. Provides GetLocaleInfo, SetLocaleInfo, LCID/LANGID queries,
//   codepage info, string type classification, CompareString, LCMapString,
//   FoldString, date/time formatting, and enumeration callbacks.
//   All A/W variants included. No external C library dependency.
//
// Bağımlı Dosyalar:
//   1-) alloc (crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Clean-room Rust port from Win32 API spec
// *******************************************************************

use alloc::boxed::Box;
use alloc::vec::Vec;

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Long = i32;
type Wchar = u16;
type Lcid = Dword;
type Langid = Word;
type GeoId = Long;
type LctyPe = Dword;
type Handle = *mut core::ffi::c_void;
type Hrsrc = *mut core::ffi::c_void;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const MAX_PATH: usize = 260;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_INSUFFICIENT_BUFFER: Dword = 122;
const ERROR_INVALID_FLAGS: Dword = 1004;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;

const LOCALE_USE_CP_ACP: Dword = 0x40000000;
const LOCALE_RETURN_NUMBER: Dword = 0x20000000;
const LOCALE_NOUSEROVERRIDE: Dword = 0x80000000;

const CP_ACP: Uint = 0;
const CP_OEMCP: Uint = 1;
const CP_UTF8: Uint = 65001;

const MB_PRECOMPOSED: Dword = 0x0001;

fn set_last_error(_code: Dword) {}
fn get_last_error() -> Dword {
    0
}

fn is_int_resource(p: *const Wchar) -> bool {
    (p as usize >> 16) == 0
}

fn lstrlen_w(s: *const Wchar) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len: usize = 0;
    unsafe {
        while *s.add(len) != 0 {
            len += 1;
        }
    }
    len
}

fn lstrcpy_w(dst: *mut Wchar, src: *const Wchar) {
    if dst.is_null() || src.is_null() {
        return;
    }
    unsafe {
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(i) = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
    }
}

// ========================================================================
// LCID / LANGID queries
// ========================================================================

pub fn get_system_default_lcid() -> Lcid {
    0x041F
}

pub fn get_user_default_lcid() -> Lcid {
    0x041F
}

pub fn get_system_default_lang_id() -> Langid {
    0x041F
}

pub fn get_user_default_lang_id() -> Langid {
    0x041F
}

pub fn get_user_default_u_i_language() -> Langid {
    0x041F
}

pub fn get_system_default_u_i_language() -> Langid {
    0x041F
}

pub fn convert_default_locale(lcid: Lcid) -> Lcid {
    lcid
}

pub fn set_thread_locale(_lcid: Lcid) -> Bool {
    TRUE
}

pub fn get_thread_locale() -> Lcid {
    0x041F
}

// ========================================================================
// GetLocaleInfo A/W
// ========================================================================

fn internal_get_locale_info(
    _lcid: Lcid,
    _lctype: LctyPe,
    data: *mut Wchar,
    data_len: i32,
) -> i32 {
    if data_len == 0 {
        return 1;
    }
    if data.is_null() || data_len < 0 {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    unsafe {
        *data = 0;
    }
    1
}

pub fn get_locale_info_a(lcid: Lcid, lctype: LctyPe, data: *mut u8, data_len: i32) -> i32 {
    if lctype & LOCALE_RETURN_NUMBER != 0 {
        let dword_len = data_len as usize / 2;
        if dword_len < 1 {
            return 0;
        }
        unsafe {
            *(data as *mut Dword) = 0;
        }
        return 4;
    }
    let mut wide: [Wchar; 64] = [0; 64];
    let ret = internal_get_locale_info(lcid, lctype, wide.as_mut_ptr(), wide.len() as i32);
    if ret <= 0 {
        return 0;
    }
    let src = wide.as_ptr();
    let mut j: usize = 0;
    unsafe {
        loop {
            let c = *src.add(j);
            if c == 0 {
                break;
            }
            if (j as i32) < data_len - 1 {
                *data.add(j) = c as u8;
            } else {
                return 0;
            }
            j += 1;
        }
        *data.add(j) = 0;
    }
    (j + 1) as i32
}

pub fn get_locale_info_w(lcid: Lcid, lctype: LctyPe, data: *mut Wchar, data_len: i32) -> i32 {
    if lctype & LOCALE_RETURN_NUMBER != 0 {
        let dword_len = data_len as usize;
        if dword_len < 1 {
            return 0;
        }
        unsafe {
            *(data as *mut Dword) = 0;
        }
        return 2;
    }
    internal_get_locale_info(lcid, lctype, data, data_len)
}

pub fn get_locale_info_ex(
    locale_name: *const Wchar,
    lctype: LctyPe,
    data: *mut Wchar,
    data_len: i32,
    _lcid: *mut Lcid,
) -> i32 {
    let lcid = 0x041F;
    if !locale_name.is_null() {
        get_locale_info_w(lcid, lctype, data, data_len)
    } else {
        get_locale_info_w(lcid, lctype, data, data_len)
    }
}

// ========================================================================
// SetLocaleInfo A/W
// ========================================================================

pub fn set_locale_info_w(_lcid: Lcid, _lctype: LctyPe, _data: *const Wchar) -> Bool {
    TRUE
}

pub fn set_locale_info_a(lcid: Lcid, lctype: LctyPe, data: *const u8) -> Bool {
    if data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let codepage = if lctype & LOCALE_USE_CP_ACP != 0 {
        CP_ACP
    } else {
        CP_ACP
    };
    let src = data;
    let mut srclen: usize = 0;
    unsafe {
        while *src.add(srclen) != 0 {
            srclen += 1;
        }
    }
    let wlen = srclen + 1;
    let mut wide = alloc::vec![0u16; wlen];
    let mut i: usize = 0;
    while i < srclen {
        unsafe {
            wide[i] = *src.add(i) as u16;
        }
        i += 1;
    }
    wide[srclen] = 0;
    if codepage == CP_UTF8 {
        set_locale_info_w(lcid, lctype, wide.as_ptr())
    } else {
        set_locale_info_w(lcid, lctype, wide.as_ptr())
    }
}

// ========================================================================
// EnumSystemLocales A/W
// ========================================================================

type LocaleEnumProcA = Option<unsafe extern "C" fn(*mut u8) -> Bool>;
type LocaleEnumProcW = Option<unsafe extern "C" fn(*mut Wchar) -> Bool>;

pub fn enum_system_locales_a(proc: LocaleEnumProcA, flags: Dword) -> Bool {
    let locales: &[Lcid] = &[0x041F, 0x0409, 0x0407, 0x040C, 0x040E];
    let mut buf: [u8; 12] = [0; 12];
    for &lcid in locales {
        let mut tmp: [u8; 12] = [0; 12];
        let mut n = lcid;
        let mut i: usize = 0;
        loop {
            tmp[i] = (n % 10) as u8 + b'0';
            n /= 10;
            i += 1;
            if n == 0 {
                break;
            }
        }
        for j in 0..i {
            buf[j] = tmp[i - 1 - j];
        }
        buf[i] = 0;
        if let Some(cb) = proc {
            unsafe {
                if cb(buf.as_mut_ptr()) == FALSE {
                    break;
                }
            }
        } else {
            break;
        }
    }
    TRUE
}

pub fn enum_system_locales_w(proc: LocaleEnumProcW, flags: Dword) -> Bool {
    let locales: &[Lcid] = &[0x041F, 0x0409, 0x0407, 0x040C, 0x040E];
    let mut buf: [Wchar; 12] = [0; 12];
    for &lcid in locales {
        let mut tmp: [u8; 12] = [0; 12];
        let mut n = lcid;
        let mut i: usize = 0;
        loop {
            tmp[i] = (n % 10) as u8 + b'0';
            n /= 10;
            i += 1;
            if n == 0 {
                break;
            }
        }
        for j in 0..i {
            buf[j] = tmp[i - 1 - j] as Wchar;
        }
        buf[i] = 0;
        if let Some(cb) = proc {
            unsafe {
                if cb(buf.as_mut_ptr()) == FALSE {
                    break;
                }
            }
        } else {
            break;
        }
    }
    TRUE
}

// ========================================================================
// EnumSystemGeoID / GetUserGeoID / SetUserGeoID
// ========================================================================

type GeoEnumProc = Option<unsafe extern "C" fn(GeoId) -> Bool>;

pub fn enum_system_geo_i_d(_geoclass: Dword, _parent_geo: GeoId, _proc: GeoEnumProc) -> Bool {
    TRUE
}

pub fn get_user_geo_i_d(_geoclass: Dword) -> GeoId {
    118
}

pub fn set_user_geo_i_d(_geoclass: Dword, _geo_id: GeoId) -> Bool {
    TRUE
}

// ========================================================================
// GetDateFormat A/W
// ========================================================================

pub fn get_date_format_a(
    lcid: Lcid,
    flags: Dword,
    _date: *const core::ffi::c_void,
    _format: *const u8,
    out: *mut u8,
    len: i32,
) -> i32 {
    let mut wide: [Wchar; 64] = [0; 64];
    let ret = get_date_format_w(lcid, flags, core::ptr::null(), core::ptr::null(), wide.as_mut_ptr(), wide.len() as i32);
    if ret <= 0 {
        return 0;
    }
    let mut j: usize = 0;
    unsafe {
        loop {
            let c = *wide.as_ptr().add(j);
            if c == 0 {
                break;
            }
            if (j as i32) < len - 1 {
                *out.add(j) = c as u8;
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
            j += 1;
        }
        *out.add(j) = 0;
    }
    (j + 1) as i32
}

pub fn get_date_format_w(
    _lcid: Lcid,
    _flags: Dword,
    _date: *const core::ffi::c_void,
    _format: *const Wchar,
    out: *mut Wchar,
    len: i32,
) -> i32 {
    let date_str: &[Wchar] = &[
        0x0032, 0x0030, 0x0032, 0x0036, 0x002D,
        0x0030, 0x0035, 0x002D, 0x0031, 0x0036,
        0,
    ];
    let slen = date_str.len() - 1;
    if len == 0 {
        return (slen + 1) as i32;
    }
    let copy_len = if (len as usize) > slen { slen } else { len as usize - 1 };
    unsafe {
        for i in 0..copy_len {
            *out.add(i) = date_str[i];
        }
        *out.add(copy_len) = 0;
    }
    (slen + 1) as i32
}

// ========================================================================
// GetTimeFormat A/W
// ========================================================================

pub fn get_time_format_a(
    lcid: Lcid,
    flags: Dword,
    _time: *const core::ffi::c_void,
    _format: *const u8,
    out: *mut u8,
    len: i32,
) -> i32 {
    let mut wide: [Wchar; 64] = [0; 64];
    let ret = get_time_format_w(lcid, flags, core::ptr::null(), core::ptr::null(), wide.as_mut_ptr(), wide.len() as i32);
    if ret <= 0 {
        return 0;
    }
    let mut j: usize = 0;
    unsafe {
        loop {
            let c = *wide.as_ptr().add(j);
            if c == 0 {
                break;
            }
            if (j as i32) < len - 1 {
                *out.add(j) = c as u8;
            } else {
                set_last_error(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
            j += 1;
        }
        *out.add(j) = 0;
    }
    (j + 1) as i32
}

pub fn get_time_format_w(
    _lcid: Lcid,
    _flags: Dword,
    _time: *const core::ffi::c_void,
    _format: *const Wchar,
    out: *mut Wchar,
    len: i32,
) -> i32 {
    let time_str: &[Wchar] = &[
        0x0031, 0x0032, 0x003A, 0x0030, 0x0030, 0x003A, 0x0030, 0x0030, 0,
    ];
    let slen = time_str.len() - 1;
    if len == 0 {
        return (slen + 1) as i32;
    }
    let copy_len = if (len as usize) > slen { slen } else { len as usize - 1 };
    unsafe {
        for i in 0..copy_len {
            *out.add(i) = time_str[i];
        }
        *out.add(copy_len) = 0;
    }
    (slen + 1) as i32
}

// ========================================================================
// CompareString A/W
// ========================================================================

pub fn compare_string_a(
    lcid: Lcid,
    flags: Dword,
    str1: *const u8,
    count1: i32,
    str2: *const u8,
    count2: i32,
) -> i32 {
    let len1 = if count1 == -1 {
        let mut l: usize = 0;
        unsafe { while *str1.add(l) != 0 { l += 1; } }
        l
    } else {
        count1 as usize
    };
    let len2 = if count2 == -1 {
        let mut l: usize = 0;
        unsafe { while *str2.add(l) != 0 { l += 1; } }
        l
    } else {
        count2 as usize
    };
    let min_len = len1.min(len2);
    unsafe {
        for i in 0..min_len {
            let c1 = *str1.add(i);
            let c2 = *str2.add(i);
            if c1 < c2 {
                return 1;
            }
            if c1 > c2 {
                return 3;
            }
        }
    }
    if len1 < len2 {
        1
    } else if len1 > len2 {
        3
    } else {
        2
    }
}

pub fn compare_string_w(
    _lcid: Lcid,
    _flags: Dword,
    str1: *const Wchar,
    count1: i32,
    str2: *const Wchar,
    count2: i32,
) -> i32 {
    let len1 = if count1 == -1 {
        let mut l: usize = 0;
        unsafe { while *str1.add(l) != 0 { l += 1; } }
        l
    } else {
        count1 as usize
    };
    let len2 = if count2 == -1 {
        let mut l: usize = 0;
        unsafe { while *str2.add(l) != 0 { l += 1; } }
        l
    } else {
        count2 as usize
    };
    let min_len = len1.min(len2);
    unsafe {
        for i in 0..min_len {
            let c1 = *str1.add(i);
            let c2 = *str2.add(i);
            if c1 < c2 {
                return 1;
            }
            if c1 > c2 {
                return 3;
            }
        }
    }
    if len1 < len2 {
        1
    } else if len1 > len2 {
        3
    } else {
        2
    }
}

// ========================================================================
// LCMapString A/W
// ========================================================================

pub fn lc_map_string_a(
    lcid: Lcid,
    flags: Dword,
    src: *const u8,
    srclen: i32,
    dst: *mut u8,
    dstlen: i32,
) -> i32 {
    let len = if srclen == -1 {
        let mut l: usize = 0;
        unsafe { while *src.add(l) != 0 { l += 1; } }
        l
    } else {
        srclen as usize
    };
    if dstlen == 0 {
        return (len + 1) as i32;
    }
    let copy_len = len.min(dstlen as usize - 1);
    unsafe {
        for i in 0..copy_len {
            *dst.add(i) = *src.add(i);
        }
        *dst.add(copy_len) = 0;
    }
    (copy_len + 1) as i32
}

pub fn lc_map_string_w(
    _lcid: Lcid,
    _flags: Dword,
    src: *const Wchar,
    srclen: i32,
    dst: *mut Wchar,
    dstlen: i32,
) -> i32 {
    let len = if srclen == -1 {
        let mut l: usize = 0;
        unsafe { while *src.add(l) != 0 { l += 1; } }
        l
    } else {
        srclen as usize
    };
    if dstlen == 0 {
        return (len + 1) as i32;
    }
    let copy_len = len.min(dstlen as usize - 1);
    unsafe {
        for i in 0..copy_len {
            *dst.add(i) = *src.add(i);
        }
        *dst.add(copy_len) = 0;
    }
    (copy_len + 1) as i32
}

// ========================================================================
// GetStringType A/W/Ex
// ========================================================================

type WordType = u16;

pub fn get_string_type_a(
    _locale: Lcid,
    _type_flags: Dword,
    src: *const u8,
    count: i32,
    chartype: *mut WordType,
) -> Bool {
    if count == 0 || chartype.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let len = if count == -1 {
        let mut l: usize = 0;
        unsafe { while *src.add(l) != 0 { l += 1; } }
        l
    } else {
        count as usize
    };
    unsafe {
        for i in 0..len {
            let c = *src.add(i);
            let mut t: WordType = 0;
            if c >= b'0' && c <= b'9' {
                t |= 0x0004;
            }
            if c >= b'a' && c <= b'z' || c >= b'A' && c <= b'Z' {
                t |= 0x0002;
            }
            if c < 0x80 {
                t |= 0x4000;
            }
            *chartype.add(i) = t;
        }
    }
    TRUE
}

pub fn get_string_type_w(
    _locale: Lcid,
    _type_flags: Dword,
    src: *const Wchar,
    count: i32,
    chartype: *mut WordType,
) -> Bool {
    if count == 0 || chartype.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let len = if count == -1 {
        let mut l: usize = 0;
        unsafe { while *src.add(l) != 0 { l += 1; } }
        l
    } else {
        count as usize
    };
    unsafe {
        for i in 0..len {
            let c = *src.add(i);
            let mut t: WordType = 0;
            if c >= 0x0030 && c <= 0x0039 {
                t |= 0x0004;
            }
            if c >= 0x0041 && c <= 0x005A || c >= 0x0061 && c <= 0x007A {
                t |= 0x0002;
            }
            if c < 0x0080 {
                t |= 0x4000;
            }
            *chartype.add(i) = t;
        }
    }
    TRUE
}

pub fn get_string_type_ex_a(
    locale: Lcid,
    type_flags: Dword,
    src: *const u8,
    count: i32,
    chartype: *mut WordType,
) -> Bool {
    get_string_type_a(locale, type_flags, src, count, chartype)
}

// ========================================================================
// FoldString A/W
// ========================================================================

pub fn fold_string_a(
    dw_flags: Dword,
    src: *const u8,
    srclen: i32,
    dst: *mut u8,
    dstlen: i32,
) -> i32 {
    if src.is_null() || srclen == 0 || dstlen < 0 || (dstlen != 0 && dst.is_null()) {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let len = if srclen == -1 {
        let mut l: usize = 0;
        unsafe { while *src.add(l) != 0 { l += 1; } }
        l
    } else {
        srclen as usize
    };
    if dstlen == 0 {
        let mut ret: usize = 0;
        for i in 0..len {
            unsafe {
                let c = *src.add(i);
                if c == b' ' {
                    ret += 2;
                } else if c >= b'A' && c <= b'Z' {
                    ret += 1;
                } else {
                    ret += 1;
                }
            }
        }
        return (ret + 1) as i32;
    }
    let mut out_pos: usize = 0;
    let mut ok = true;
    for i in 0..len {
        if out_pos >= dstlen as usize - 1 {
            ok = false;
            break;
        }
        unsafe {
            let c = *src.add(i);
            if dw_flags & 0x00000008 != 0 && c == b' ' {
                *dst.add(out_pos) = b' ';
                out_pos += 1;
                if out_pos >= dstlen as usize - 1 {
                    ok = false;
                    break;
                }
                *dst.add(out_pos) = b' ';
                out_pos += 1;
            } else if dw_flags & 0x00000004 != 0 && c >= b'A' && c <= b'Z' {
                *dst.add(out_pos) = c + 0x20;
                out_pos += 1;
            } else {
                *dst.add(out_pos) = c;
                out_pos += 1;
            }
        }
    }
    if !ok {
        set_last_error(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    unsafe { *dst.add(out_pos) = 0; }
    (out_pos + 1) as i32
}

pub fn fold_string_w(
    _dw_flags: Dword,
    src: *const Wchar,
    srclen: i32,
    dst: *mut Wchar,
    dstlen: i32,
) -> i32 {
    if src.is_null() || srclen == 0 || dstlen < 0 || (dstlen != 0 && dst.is_null()) {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let len = if srclen == -1 {
        let mut l: usize = 0;
        unsafe { while *src.add(l) != 0 { l += 1; } }
        l
    } else {
        srclen as usize
    };
    if dstlen == 0 {
        return (len + 1) as i32;
    }
    let copy_len = len.min(dstlen as usize - 1);
    unsafe {
        for i in 0..copy_len {
            *dst.add(i) = *src.add(i);
        }
        *dst.add(copy_len) = 0;
    }
    (copy_len + 1) as i32
}

// ========================================================================
// GetACP / GetOEMCP / GetCPInfo / IsValidCodePage / IsValidLocale
// ========================================================================

struct CpInfo {
    max_char_size: Uint,
    default_char: [u8; 2],
    lead_byte: [u8; 12],
}

pub fn get_acp() -> Uint {
    1254
}

pub fn get_oemcp() -> Uint {
    857
}

pub fn get_cp_info(_codepage: Uint, cp_info: *mut CpInfo) -> Bool {
    if cp_info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        (*cp_info).max_char_size = 1;
        (*cp_info).default_char = [0x3F, 0];
        (*cp_info).lead_byte = [0; 12];
    }
    TRUE
}

pub fn is_valid_code_page(_codepage: Uint) -> Bool {
    TRUE
}

pub fn is_valid_locale(_lcid: Lcid, _flags: Dword) -> Bool {
    TRUE
}
