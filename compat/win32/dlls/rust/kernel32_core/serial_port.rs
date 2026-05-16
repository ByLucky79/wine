// SPDX-License-Identifier: GPL-3.0-only

use alloc::vec::Vec;

type Bool = i32;
type Dword = u32;
type Word = u16;
type Byte = u8;
type Long = i32;
type Uint = u32;
type Handle = u64;
type Lpcstr = *const u8;
type Lpcwstr = *const u16;
type Lpstr = *mut u8;
type Lpwstr = *mut u16;
type Lpdword = *mut Dword;
type Lpbyte = *mut Byte;
type Lpvoid = *mut u8;
type Hwnd = *mut u8;
type Hmodule = u64;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_INVALID_HANDLE: Dword = 6;

/* DCB constants */
const EVENPARITY: Byte = 2;
const MARKPARITY: Byte = 3;
const NOPARITY: Byte = 0;
const ODDPARITY: Byte = 1;
const SPACEPARITY: Byte = 4;

const ONESTOPBIT: Byte = 0;
const ONE5STOPBITS: Byte = 1;
const TWOSTOPBITS: Byte = 2;

const DTR_CONTROL_ENABLE: Dword = 1;
const DTR_CONTROL_HANDSHAKE: Dword = 2;
const RTS_CONTROL_ENABLE: Dword = 1;
const RTS_CONTROL_HANDSHAKE: Dword = 2;

/* DCB flags bits */
const DCB_BINARY: Dword = 1 << 0;
const DCB_PARITY: Dword = 1 << 1;
const DCB_OUTXCTSFLOW: Dword = 1 << 2;
const DCB_OUTXDSRFLOW: Dword = 1 << 3;
const DCB_DTRCONTROL_SHIFT: Dword = 4;
const DCB_DTRCONTROL_MASK: Dword = 3 << 4;
const DCB_DSRSENSITIVITY: Dword = 1 << 6;
const DCB_TXCONTINUEONXOFF: Dword = 1 << 7;
const DCB_OUTX: Dword = 1 << 8;
const DCB_INX: Dword = 1 << 9;
const DCB_ERRORCHAR: Dword = 1 << 10;
const DCB_NULL: Dword = 1 << 11;
const DCB_RTSCONTROL_SHIFT: Dword = 12;
const DCB_RTSCONTROL_MASK: Dword = 3 << 12;
const DCB_ABORTONERROR: Dword = 1 << 14;
const DCB_RESERVED: Dword = 1 << 15;

/* Parity constants for BuildCommDCB */
const PARITY_E: Byte = 0;
const PARITY_M: Byte = 1;
const PARITY_N: Byte = 2;
const PARITY_O: Byte = 3;
const PARITY_S: Byte = 4;

#[repr(C)]
#[derive(Clone, Copy)]
struct Dcb {
    dcb_length: Dword,
    baud_rate: Dword,
    flags: Dword,
    w_reserved: Word,
    xon_lim: Word,
    xoff_lim: Word,
    byte_size: Byte,
    parity: Byte,
    stop_bits: Byte,
    xon_char: Byte,
    xoff_char: Byte,
    error_char: Byte,
    eof_char: Byte,
    evt_char: Byte,
    w_reserved1: Word,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CommTimeouts {
    read_interval_timeout: Dword,
    read_total_timeout_multiplier: Dword,
    read_total_timeout_constant: Dword,
    write_total_timeout_multiplier: Dword,
    write_total_timeout_constant: Dword,
}

#[repr(C)]
struct CommConfig {
    cb_size: Dword,
    version: Word,
    reserved: Word,
    dcb: Dcb,
}

fn set_last_error(_code: Dword) {}

fn wcslen(ptr: Lpcwstr) -> usize {
    let mut len: usize = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    len
}

fn wcschr(ptr: Lpcwstr, c: u16) -> Option<usize> {
    let mut i: usize = 0;
    unsafe {
        loop {
            let ch = *ptr.add(i);
            if ch == c {
                return Some(i);
            }
            if ch == 0 {
                return None;
            }
            i += 1;
        }
    }
}

fn wcsnicmp(mut a: Lpcwstr, b: &[u16], n: usize) -> Bool {
    for i in 0..n {
        if i >= b.len() {
            return -1;
        }
        unsafe {
            let ca = *a.add(i);
            let cb = b[i];
            let la = if ca >= b'A' as u16 && ca <= b'Z' as u16 { ca + 32 } else { ca };
            let lb = if cb >= b'A' as u16 && cb <= b'Z' as u16 { cb + 32 } else { cb };
            if la != lb {
                if la < lb { return -1; }
                return 1;
            }
            if ca == 0 {
                if i + 1 == n {
                    return 0;
                }
                return -1;
            }
        }
    }
    0
}

fn wcsncmp(a: Lpcwstr, b: &[u16], n: usize) -> Bool {
    for i in 0..n {
        if i >= b.len() {
            return -1;
        }
        unsafe {
            let ca = *a.add(i);
            let cb = b[i];
            if ca != cb {
                if ca < cb {
                    return -1;
                }
                return 1;
            }
            if ca == 0 {
                if i + 1 == n {
                    return 0;
                }
                return -1;
            }
        }
    }
    0
}

fn wcstoul(ptr: Lpcwstr) -> (Dword, usize) {
    let mut val: Dword = 0;
    let mut i: usize = 0;
    unsafe {
        loop {
            let c = *ptr.add(i);
            if c >= b'0' as u16 && c <= b'9' as u16 {
                val = val.wrapping_mul(10).wrapping_add((c - b'0' as u16) as Dword);
                i += 1;
            } else {
                break;
            }
        }
    }
    (val, i)
}

fn comm_parse_start(ptr: Lpcwstr) -> Option<Lpcwstr> {
    let saved = ptr;
    unsafe {
        if wcsnicmp(ptr, &[b'C' as u16, b'O' as u16, b'M' as u16], 3) == 0 {
            let mut p = ptr.add(3);
            let c = *p;
            if c < b'1' as u16 || c > b'9' as u16 {
                return None;
            }
            while *p >= b'0' as u16 && *p <= b'9' as u16 {
                p = p.add(1);
            }
            if *p != b':' as u16 && *p != b' ' as u16 {
                return None;
            }
            while *p == b' ' as u16 {
                p = p.add(1);
            }
            if *p == b':' as u16 {
                p = p.add(1);
                while *p == b' ' as u16 {
                    p = p.add(1);
                }
            }
            return Some(p);
        }
        if *ptr == b' ' as u16 {
            return None;
        }
    }
    Some(ptr)
}

fn comm_parse_number(ptr: Lpcwstr, number: &mut Dword) -> Option<Lpcwstr> {
    unsafe {
        let c = *ptr;
        if c < b'0' as u16 || c > b'9' as u16 {
            return None;
        }
    }
    let (val, consumed) = wcstoul(ptr);
    *number = val;
    unsafe { Some(ptr.add(consumed)) }
}

fn comm_parse_parity(ptr: Lpcwstr, parity: &mut Byte) -> Option<Lpcwstr> {
    unsafe {
        let c = *ptr;
        if c == b'E' as u16 || c == b'e' as u16 { *parity = EVENPARITY; }
        else if c == b'M' as u16 || c == b'm' as u16 { *parity = MARKPARITY; }
        else if c == b'N' as u16 || c == b'n' as u16 { *parity = NOPARITY; }
        else if c == b'O' as u16 || c == b'o' as u16 { *parity = ODDPARITY; }
        else if c == b'S' as u16 || c == b's' as u16 { *parity = SPACEPARITY; }
        else { return None; }
        Some(ptr.add(1))
    }
}

fn comm_parse_byte_size(ptr: Lpcwstr, byte_size: &mut Byte) -> Option<Lpcwstr> {
    let mut temp: Dword = 0;
    let ptr = comm_parse_number(ptr, &mut temp)?;
    if temp >= 5 && temp <= 8 {
        *byte_size = temp as Byte;
        Some(ptr)
    } else {
        None
    }
}

fn comm_parse_stop_bits(ptr: Lpcwstr, stop_bits: &mut Byte) -> Option<Lpcwstr> {
    unsafe {
        if wcsncmp(ptr, &[b'1' as u16, b'.' as u16, b'5' as u16], 3) == 0 {
            *stop_bits = ONE5STOPBITS;
            Some(ptr.add(3))
        } else {
            let mut temp: Dword = 0;
            let ptr = comm_parse_number(ptr, &mut temp)?;
            if temp == 1 {
                *stop_bits = ONESTOPBIT;
            } else if temp == 2 {
                *stop_bits = TWOSTOPBITS;
            } else {
                return None;
            }
            Some(ptr)
        }
    }
}

fn comm_parse_on_off(ptr: Lpcwstr, on_off: &mut Dword) -> Option<Lpcwstr> {
    unsafe {
        if wcsnicmp(ptr, &[b'O' as u16, b'N' as u16], 2) == 0 {
            *on_off = 1;
            Some(ptr.add(2))
        } else if wcsnicmp(ptr, &[b'O' as u16, b'F' as u16, b'F' as u16], 3) == 0 {
            *on_off = 0;
            Some(ptr.add(3))
        } else {
            None
        }
    }
}

fn skip_spaces(ptr: Lpcwstr) -> Lpcwstr {
    let mut p = ptr;
    unsafe {
        while *p == b' ' as u16 {
            p = p.add(1);
        }
    }
    p
}

fn comm_build_old_dcb(device: Lpcwstr, dcb: &mut Dcb) -> Bool {
    let mut temp: Dword = 0;
    let device = match comm_parse_number(device, &mut temp) {
        Some(p) => p,
        None => return FALSE,
    };
    dcb.baud_rate = match temp {
        11 | 30 | 60 => temp * 10,
        12 | 24 | 48 | 96 => temp * 100,
        19 => 19200,
        _ => temp,
    };

    let device = skip_spaces(device);
    unsafe {
        if *device != b',' as u16 {
            return FALSE;
        }
    }
    let device = skip_spaces(unsafe { device.add(1) });

    let device = match comm_parse_parity(device, &mut dcb.parity) {
        Some(p) => p,
        None => return FALSE,
    };

    let device = skip_spaces(device);
    unsafe {
        if *device != b',' as u16 {
            return FALSE;
        }
    }
    let device = skip_spaces(unsafe { device.add(1) });

    let device = match comm_parse_byte_size(device, &mut dcb.byte_size) {
        Some(p) => p,
        None => return FALSE,
    };

    let device = skip_spaces(device);
    unsafe {
        if *device != b',' as u16 {
            return FALSE;
        }
    }
    let device = skip_spaces(unsafe { device.add(1) });

    let device = match comm_parse_stop_bits(device, &mut dcb.stop_bits) {
        Some(p) => p,
        None => return FALSE,
    };

    let device = skip_spaces(device);
    let mut last: u16 = 0;
    unsafe {
        if *device == b',' as u16 {
            let device = device.add(1);
            let device = skip_spaces(device);
            if *device != 0 {
                last = *device;
            }
        }
    }

    if last == 0 {
        dcb.flags &= !(DCB_OUTX | DCB_INX | DCB_OUTXCTSFLOW | DCB_OUTXDSRFLOW);
        dcb.flags |= DCB_DTRCONTROL_MASK & (DTR_CONTROL_ENABLE << DCB_DTRCONTROL_SHIFT);
        dcb.flags |= DCB_RTSCONTROL_MASK & (RTS_CONTROL_ENABLE << DCB_RTSCONTROL_SHIFT);
    } else if last == b'X' as u16 || last == b'x' as u16 {
        dcb.flags |= DCB_OUTX | DCB_INX;
        dcb.flags &= !(DCB_OUTXCTSFLOW | DCB_OUTXDSRFLOW);
        dcb.flags |= DCB_DTRCONTROL_MASK & (DTR_CONTROL_ENABLE << DCB_DTRCONTROL_SHIFT);
        dcb.flags |= DCB_RTSCONTROL_MASK & (RTS_CONTROL_ENABLE << DCB_RTSCONTROL_SHIFT);
    } else if last == b'P' as u16 || last == b'p' as u16 {
        dcb.flags &= !(DCB_OUTX | DCB_INX);
        dcb.flags |= DCB_OUTXCTSFLOW | DCB_OUTXDSRFLOW;
        dcb.flags |= DCB_DTRCONTROL_MASK & (DTR_CONTROL_HANDSHAKE << DCB_DTRCONTROL_SHIFT);
        dcb.flags |= DCB_RTSCONTROL_MASK & (RTS_CONTROL_HANDSHAKE << DCB_RTSCONTROL_SHIFT);
    } else {
        return FALSE;
    }

    unsafe {
        if *device != 0 {
            return FALSE;
        }
    }

    TRUE
}

fn comm_build_new_dcb(device: Lpcwstr, dcb: &mut Dcb, timeouts: &mut CommTimeouts) -> Bool {
    let mut temp: Dword = 0;
    let mut baud_set: Bool = FALSE;
    let mut stop_set: Bool = FALSE;

    let mut ptr = device;
    unsafe {
        while *ptr != 0 {
            ptr = skip_spaces(ptr);

            if wcsnicmp(ptr, &[b'b' as u16, b'a' as u16, b'u' as u16, b'd' as u16, b'=' as u16], 5) == 0 {
                baud_set = TRUE;
                ptr = match comm_parse_number(ptr.add(5), &mut dcb.baud_rate) {
                    Some(p) => p,
                    None => return FALSE,
                };
            } else if wcsnicmp(ptr, &[b'p' as u16, b'a' as u16, b'r' as u16, b'i' as u16, b't' as u16, b'y' as u16, b'=' as u16], 7) == 0 {
                ptr = match comm_parse_parity(ptr.add(7), &mut dcb.parity) {
                    Some(p) => p,
                    None => return FALSE,
                };
            } else if wcsnicmp(ptr, &[b'd' as u16, b'a' as u16, b't' as u16, b'a' as u16, b'=' as u16], 5) == 0 {
                ptr = match comm_parse_byte_size(ptr.add(5), &mut dcb.byte_size) {
                    Some(p) => p,
                    None => return FALSE,
                };
            } else if wcsnicmp(ptr, &[b's' as u16, b't' as u16, b'o' as u16, b'p' as u16, b'=' as u16], 5) == 0 {
                stop_set = TRUE;
                ptr = match comm_parse_stop_bits(ptr.add(5), &mut dcb.stop_bits) {
                    Some(p) => p,
                    None => return FALSE,
                };
            } else if wcsnicmp(ptr, &[b't' as u16, b'o' as u16, b'=' as u16], 3) == 0 {
                ptr = match comm_parse_on_off(ptr.add(3), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                timeouts.read_interval_timeout = 0;
                timeouts.read_total_timeout_multiplier = 0;
                timeouts.read_total_timeout_constant = 0;
                timeouts.write_total_timeout_multiplier = 0;
                timeouts.write_total_timeout_constant = if temp != 0 { 60000 } else { 0 };
            } else if wcsnicmp(ptr, &[b'x' as u16, b'o' as u16, b'n' as u16, b'=' as u16], 4) == 0 {
                ptr = match comm_parse_on_off(ptr.add(4), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                if temp != 0 {
                    dcb.flags |= DCB_OUTX | DCB_INX;
                } else {
                    dcb.flags &= !(DCB_OUTX | DCB_INX);
                }
            } else if wcsnicmp(ptr, &[b'o' as u16, b'd' as u16, b's' as u16, b'r' as u16, b'=' as u16], 5) == 0 {
                ptr = match comm_parse_on_off(ptr.add(5), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                if temp != 0 {
                    dcb.flags |= DCB_OUTXDSRFLOW;
                } else {
                    dcb.flags &= !DCB_OUTXDSRFLOW;
                }
            } else if wcsnicmp(ptr, &[b'o' as u16, b'c' as u16, b't' as u16, b's' as u16, b'=' as u16], 5) == 0 {
                ptr = match comm_parse_on_off(ptr.add(5), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                if temp != 0 {
                    dcb.flags |= DCB_OUTXCTSFLOW;
                } else {
                    dcb.flags &= !DCB_OUTXCTSFLOW;
                }
            } else if wcsnicmp(ptr, &[b'd' as u16, b't' as u16, b'r' as u16, b'=' as u16], 4) == 0 {
                ptr = match comm_parse_on_off(ptr.add(4), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                dcb.flags &= !DCB_DTRCONTROL_MASK;
                dcb.flags |= (temp << DCB_DTRCONTROL_SHIFT) & DCB_DTRCONTROL_MASK;
            } else if wcsnicmp(ptr, &[b'r' as u16, b't' as u16, b's' as u16, b'=' as u16], 4) == 0 {
                ptr = match comm_parse_on_off(ptr.add(4), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                dcb.flags &= !DCB_RTSCONTROL_MASK;
                dcb.flags |= (temp << DCB_RTSCONTROL_SHIFT) & DCB_RTSCONTROL_MASK;
            } else if wcsnicmp(ptr, &[b'i' as u16, b'd' as u16, b's' as u16, b'r' as u16, b'=' as u16], 5) == 0 {
                ptr = match comm_parse_on_off(ptr.add(5), &mut temp) {
                    Some(p) => p,
                    None => return FALSE,
                };
                if temp != 0 {
                    dcb.flags |= DCB_DSRSENSITIVITY;
                } else {
                    dcb.flags &= !DCB_DSRSENSITIVITY;
                }
            } else {
                return FALSE;
            }

            if *ptr != 0 && *ptr != b' ' as u16 {
                return FALSE;
            }
        }
    }

    if stop_set == FALSE {
        if baud_set != FALSE && dcb.baud_rate == 110 {
            dcb.stop_bits = TWOSTOPBITS;
        } else {
            dcb.stop_bits = ONESTOPBIT;
        }
    }

    TRUE
}

fn build_comm_dcb_and_timeouts_w(devid: Lpcwstr, lpdcb: *mut Dcb, lptimeouts: *mut CommTimeouts) -> Bool {
    let mut dcb: Dcb;
    let mut timeouts: CommTimeouts;
    unsafe {
        dcb = *lpdcb;
        if !lptimeouts.is_null() {
            timeouts = *lptimeouts;
        } else {
            timeouts = CommTimeouts {
                read_interval_timeout: 0,
                read_total_timeout_multiplier: 0,
                read_total_timeout_constant: 0,
                write_total_timeout_multiplier: 0,
                write_total_timeout_constant: 0,
            };
        }
    }
    dcb.dcb_length = core::mem::size_of::<Dcb>() as Dword;

    let ptr = match comm_parse_start(devid) {
        Some(p) => p,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    };

    let result = match wcschr(ptr, b',' as u16) {
        Some(_) => comm_build_old_dcb(ptr, &mut dcb),
        None => comm_build_new_dcb(ptr, &mut dcb, &mut timeouts),
    };

    if result != FALSE {
        unsafe {
            *lpdcb = dcb;
            if !lptimeouts.is_null() {
                *lptimeouts = timeouts;
            }
        }
        TRUE
    } else {
        set_last_error(ERROR_INVALID_PARAMETER);
        FALSE
    }
}

fn build_comm_dcb_and_timeouts_a(device: Lpcstr, lpdcb: *mut Dcb, lptimeouts: *mut CommTimeouts) -> Bool {
    if device.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let mut wide: Vec<u16> = Vec::new();
    unsafe {
        let mut i: usize = 0;
        loop {
            let c = *device.add(i) as u16;
            if c == 0 {
                break;
            }
            wide.push(c);
            i += 1;
        }
    }
    wide.push(0);
    build_comm_dcb_and_timeouts_w(wide.as_ptr(), lpdcb, lptimeouts)
}

/* Exported public API */
pub fn build_comm_dcba(device: Lpcstr, lpdcb: *mut Dcb) -> Bool {
    build_comm_dcb_and_timeouts_a(device, lpdcb, core::ptr::null_mut())
}

pub fn build_comm_dcbw(devid: Lpcwstr, lpdcb: *mut Dcb) -> Bool {
    build_comm_dcb_and_timeouts_w(devid, lpdcb, core::ptr::null_mut())
}

pub fn build_comm_dcb_and_timeoutsa(device: Lpcstr, lpdcb: *mut Dcb, lptimeouts: *mut CommTimeouts) -> Bool {
    build_comm_dcb_and_timeouts_a(device, lpdcb, lptimeouts)
}

pub fn build_comm_dcb_and_timeoutsw(devid: Lpcwstr, lpdcb: *mut Dcb, lptimeouts: *mut CommTimeouts) -> Bool {
    build_comm_dcb_and_timeouts_w(devid, lpdcb, lptimeouts)
}

pub fn set_comm_state(_handle: Handle, _lpdcb: *const Dcb) -> Bool {
    FALSE
}

pub fn get_comm_state(_handle: Handle, _lpdcb: *mut Dcb) -> Bool {
    FALSE
}

pub fn set_comm_timeouts(_handle: Handle, _lptimeouts: *const CommTimeouts) -> Bool {
    FALSE
}

pub fn get_comm_timeouts(_handle: Handle, _lptimeouts: *mut CommTimeouts) -> Bool {
    FALSE
}

pub fn clear_comm_error(_handle: Handle, lp_errors: Lpdword, _lp_stat: *mut u8) -> Bool {
    if !lp_errors.is_null() {
        unsafe { *lp_errors = 0; }
    }
    TRUE
}

pub fn purge_comm(_handle: Handle, _flags: Dword) -> Bool {
    TRUE
}

pub fn set_comm_mask(_handle: Handle, _mask: Dword) -> Bool {
    TRUE
}

pub fn wait_comm_event(_handle: Handle, _mask: *mut Dword, _overlapped: *mut u8) -> Bool {
    set_last_error(ERROR_INVALID_PARAMETER);
    FALSE
}

pub fn setup_comm(_handle: Handle, _in_queue: Dword, _out_queue: Dword) -> Bool {
    TRUE
}

pub fn get_comm_modem_status(_handle: Handle, lp_modem_stat: Lpdword) -> Bool {
    if !lp_modem_stat.is_null() {
        unsafe { *lp_modem_stat = 0; }
    }
    TRUE
}

pub fn escape_comm_function(_handle: Handle, _func: Dword) -> Bool {
    TRUE
}

pub fn transmit_comm_char(_handle: Handle, _char: Byte) -> Bool {
    TRUE
}

pub fn comm_config_dialoga(lpsz_device: Lpcstr, hwnd: Hwnd, lpcc: *mut CommConfig) -> Bool {
    if lpsz_device.is_null() {
        return comm_config_dialogw(core::ptr::null(), hwnd, lpcc);
    }
    let mut wide: Vec<u16> = Vec::new();
    unsafe {
        let mut i: usize = 0;
        loop {
            let c = *lpsz_device.add(i) as u16;
            if c == 0 {
                break;
            }
            wide.push(c);
            i += 1;
        }
    }
    wide.push(0);
    comm_config_dialogw(wide.as_ptr(), hwnd, lpcc)
}

pub fn comm_config_dialogw(_lpsz_device: Lpcwstr, _hwnd: Hwnd, _lpcc: *mut CommConfig) -> Bool {
    TRUE
}

pub fn get_default_comm_configw(_lpsz_name: Lpcwstr, _lpcc: *mut CommConfig, _lpdw_size: Lpdword) -> Bool {
    FALSE
}

pub fn get_default_comm_configa(lpsz_name: Lpcstr, lpcc: *mut CommConfig, lpdw_size: Lpdword) -> Bool {
    if lpsz_name.is_null() {
        return get_default_comm_configw(core::ptr::null(), lpcc, lpdw_size);
    }
    let mut wide: Vec<u16> = Vec::new();
    unsafe {
        let mut i: usize = 0;
        loop {
            let c = *lpsz_name.add(i) as u16;
            if c == 0 {
                break;
            }
            wide.push(c);
            i += 1;
        }
    }
    wide.push(0);
    get_default_comm_configw(wide.as_ptr(), lpcc, lpdw_size)
}

pub fn set_default_comm_configw(_lpsz_name: Lpcwstr, _lpcc: *const CommConfig, _dw_size: Dword) -> Bool {
    FALSE
}

pub fn set_default_comm_configa(lpsz_name: Lpcstr, lpcc: *const CommConfig, dw_size: Dword) -> Bool {
    if lpsz_name.is_null() {
        return set_default_comm_configw(core::ptr::null(), lpcc, dw_size);
    }
    let mut wide: Vec<u16> = Vec::new();
    unsafe {
        let mut i: usize = 0;
        loop {
            let c = *lpsz_name.add(i) as u16;
            if c == 0 {
                break;
            }
            wide.push(c);
            i += 1;
        }
    }
    wide.push(0);
    set_default_comm_configw(wide.as_ptr(), lpcc, dw_size)
}
