// SPDX-License-Identifier: GPL-3.0-only

use core::sync::atomic::{AtomicU32, Ordering};

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Wchar = u16;

type Lpstr = *mut u8;
type Lpcstr = *const u8;
type Lpwstr = *mut u16;
type Lpcwstr = *const u16;
type Lpdword = *mut Dword;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const ERROR_BUFFER_OVERFLOW: Dword = 111;
const ERROR_MORE_DATA: Dword = 234;
const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_BAD_LENGTH: Dword = 24;
const ERROR_ACCESS_DENIED: Dword = 5;
const ERROR_NOT_SUPPORTED: Dword = 50;

const MAX_COMPUTERNAME_LENGTH: usize = 31;

const ComputerNameNetBIOS: Dword = 0;
const ComputerNameDnsHostname: Dword = 1;
const ComputerNameDnsDomain: Dword = 2;
const ComputerNameDnsFullyQualified: Dword = 3;
const ComputerNamePhysicalNetBIOS: Dword = 4;
const ComputerNamePhysicalDnsHostname: Dword = 5;
const ComputerNamePhysicalDnsDomain: Dword = 6;
const ComputerNamePhysicalDnsFullyQualified: Dword = 7;
const ComputerNameMax: Dword = 8;

fn set_last_error(_code: Dword) {}

struct RwLock {
    locked: AtomicU32,
}

impl RwLock {
    const fn new() -> Self {
        RwLock { locked: AtomicU32::new(0) }
    }

    fn lock(&self) {
        while self.locked.swap(1, Ordering::Acquire) != 0 {
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) {
        self.locked.store(0, Ordering::Release);
    }
}

static LOCK: RwLock = RwLock::new();

static mut COMPUTER_NAME: [Wchar; MAX_COMPUTERNAME_LENGTH + 1] = [0; MAX_COMPUTERNAME_LENGTH + 1];
static mut COMPUTER_NAME_LEN: usize = 0;

static mut DNS_HOSTNAME: [Wchar; MAX_COMPUTERNAME_LENGTH + 1] = [0; MAX_COMPUTERNAME_LENGTH + 1];
static mut DNS_DOMAIN: [Wchar; MAX_COMPUTERNAME_LENGTH + 1] = [0; MAX_COMPUTERNAME_LENGTH + 1];
static mut DNS_FQDN: [Wchar; MAX_COMPUTERNAME_LENGTH + 1] = [0; MAX_COMPUTERNAME_LENGTH + 1];

pub fn init(name: *const u16, len: usize) {
    unsafe {
        let max = if len > MAX_COMPUTERNAME_LENGTH {
            MAX_COMPUTERNAME_LENGTH
        } else {
            len
        };
        for i in 0..max {
            COMPUTER_NAME[i] = *name.add(i);
        }
        COMPUTER_NAME[max] = 0;
        COMPUTER_NAME_LEN = max;

        for i in 0..max {
            DNS_HOSTNAME[i] = COMPUTER_NAME[i];
        }
        DNS_HOSTNAME[max] = 0;

        DNS_FQDN[0] = 0;
        DNS_DOMAIN[0] = 0;
    }
}

unsafe fn wcslen(ptr: *const u16) -> usize {
    let mut i: usize = 0;
    while *ptr.add(i) != 0 {
        i += 1;
    }
    i
}

unsafe fn wcscpy(dst: *mut u16, src: *const u16) {
    let mut i: usize = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}

pub fn get_computer_name_w(name: Lpwstr, size: Lpdword) -> Bool {
    if name.is_null() || size.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    LOCK.lock();
    unsafe {
        let needed = COMPUTER_NAME_LEN;
        let buf_size = *size as usize;

        if buf_size < needed {
            *size = needed as Dword;
            LOCK.unlock();
            set_last_error(ERROR_BUFFER_OVERFLOW);
            return FALSE;
        }

        for i in 0..=needed {
            *name.add(i) = COMPUTER_NAME[i];
        }
        *size = needed as Dword;
    }
    LOCK.unlock();
    TRUE
}

pub fn get_computer_name_a(name: Lpstr, size: Lpdword) -> Bool {
    if name.is_null() || size.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    LOCK.lock();
    unsafe {
        let src_len = COMPUTER_NAME_LEN;
        let mut dst_len: Uint = 0;
        for i in 0..src_len {
            let c = COMPUTER_NAME[i];
            if c > 127 {
                dst_len += 2;
            } else {
                dst_len += 1;
            }
        }
        let needed = dst_len;

        if (*size as Uint) < needed {
            *size = needed;
            LOCK.unlock();
            set_last_error(ERROR_BUFFER_OVERFLOW);
            return FALSE;
        }

        let mut out_pos: usize = 0;
        for i in 0..src_len {
            let c = COMPUTER_NAME[i] as u8;
            if c > 127 {
                *name.add(out_pos) = b'?';
                out_pos += 1;
            } else {
                *name.add(out_pos) = c;
                out_pos += 1;
            }
        }
        *name.add(out_pos) = 0;
        *size = needed - 1;
    }
    LOCK.unlock();
    TRUE
}

pub fn set_computer_name_w(_name: Lpcwstr) -> Bool {
    set_last_error(ERROR_ACCESS_DENIED);
    FALSE
}

pub fn set_computer_name_a(_name: Lpcstr) -> Bool {
    set_last_error(ERROR_ACCESS_DENIED);
    FALSE
}

pub fn get_computer_name_exw(name_type: Dword, buffer: Lpwstr, size: Lpdword) -> Bool {
    if buffer.is_null() || size.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    LOCK.lock();
    unsafe {
        let (src, src_len): (*const u16, usize) = match name_type {
            ComputerNameNetBIOS | ComputerNamePhysicalNetBIOS => {
                (COMPUTER_NAME.as_ptr(), COMPUTER_NAME_LEN)
            }
            ComputerNameDnsHostname | ComputerNamePhysicalDnsHostname => {
                let l = wcslen(DNS_HOSTNAME.as_ptr());
                (DNS_HOSTNAME.as_ptr(), l)
            }
            ComputerNameDnsDomain | ComputerNamePhysicalDnsDomain => {
                let l = wcslen(DNS_DOMAIN.as_ptr());
                (DNS_DOMAIN.as_ptr(), l)
            }
            ComputerNameDnsFullyQualified | ComputerNamePhysicalDnsFullyQualified => {
                let l = wcslen(DNS_FQDN.as_ptr());
                (DNS_FQDN.as_ptr(), l)
            }
            _ => {
                LOCK.unlock();
                set_last_error(ERROR_INVALID_PARAMETER);
                return FALSE;
            }
        };

        let buf_size = *size as usize;
        if buf_size <= src_len {
            *size = (src_len + 1) as Dword;
            LOCK.unlock();
            set_last_error(ERROR_MORE_DATA);
            return FALSE;
        }

        for i in 0..src_len {
            *buffer.add(i) = *src.add(i);
        }
        *buffer.add(src_len) = 0;
        *size = src_len as Dword;
    }
    LOCK.unlock();
    TRUE
}

pub fn get_computer_name_exa(name_type: Dword, buffer: Lpstr, size: Lpdword) -> Bool {
    if size.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let mut wide_buf: [Wchar; MAX_COMPUTERNAME_LENGTH + 1] = [0; MAX_COMPUTERNAME_LENGTH + 1];
    let mut wide_size: Dword = (MAX_COMPUTERNAME_LENGTH + 1) as Dword;

    if get_computer_name_exw(name_type, wide_buf.as_mut_ptr(), &mut wide_size) == FALSE {
        return FALSE;
    }

    unsafe {
        let src_len = wcslen(wide_buf.as_ptr());
        let mut dst_len: Uint = 0;
        for i in 0..src_len {
            let c = wide_buf[i];
            if c > 127 {
                dst_len += 2;
            } else {
                dst_len += 1;
            }
        }

        if (*size as Uint) <= dst_len {
            *size = dst_len + 1;
            set_last_error(ERROR_BUFFER_OVERFLOW);
            return FALSE;
        }

        let mut out_pos: usize = 0;
        for i in 0..src_len {
            let c = wide_buf[i] as u8;
            if c > 127 {
                *buffer.add(out_pos) = b'?';
                out_pos += 1;
            } else {
                *buffer.add(out_pos) = c;
                out_pos += 1;
            }
        }
        *buffer.add(out_pos) = 0;
        *size = src_len as Dword;
    }
    TRUE
}

pub fn set_computer_name_exw(_name_type: Dword, _buffer: Lpcwstr) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn set_computer_name_exa(_name_type: Dword, _buffer: Lpcstr) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn dns_hostname_to_computer_namew(hostname: Lpcwstr, computername: Lpwstr, size: Lpdword) -> Bool {
    if hostname.is_null() || size.is_null() {
        return FALSE;
    }
    LOCK.lock();
    unsafe {
        let fqdn_len = wcslen(DNS_FQDN.as_ptr());
        let host_len = wcslen(hostname);

        if fqdn_len == 0 || host_len == 0 {
            let needed = MAX_COMPUTERNAME_LENGTH + 1;
            if (*size as usize) < needed {
                *size = needed as Dword;
                LOCK.unlock();
                set_last_error(ERROR_BUFFER_OVERFLOW);
                return FALSE;
            }
            wcscpy(computername, COMPUTER_NAME.as_ptr());
            *size = COMPUTER_NAME_LEN as Dword;
            LOCK.unlock();
            return TRUE;
        }

        let needed = MAX_COMPUTERNAME_LENGTH + 1;
        if (*size as usize) < needed {
            *size = needed as Dword;
            LOCK.unlock();
            set_last_error(ERROR_BUFFER_OVERFLOW);
            return FALSE;
        }

        if !computername.is_null() {
            wcscpy(computername, COMPUTER_NAME.as_ptr());
        }
        *size = COMPUTER_NAME_LEN as Dword;
    }
    LOCK.unlock();
    TRUE
}

pub fn dns_hostname_to_computernamen(hostname: Lpcstr, computername: Lpstr, size: Lpdword) -> Bool {
    if hostname.is_null() || size.is_null() {
        return FALSE;
    }
    let mut wide_host: [Wchar; MAX_COMPUTERNAME_LENGTH + 3] = [0; MAX_COMPUTERNAME_LENGTH + 3];
    unsafe {
        let mut i: usize = 0;
        loop {
            let c = *hostname.add(i);
            if c == 0 {
                break;
            }
            if i >= MAX_COMPUTERNAME_LENGTH {
                break;
            }
            wide_host[i] = c as u16;
            i += 1;
        }
        wide_host[i] = 0;
    }

    let mut wide_result: [Wchar; MAX_COMPUTERNAME_LENGTH + 1] = [0; MAX_COMPUTERNAME_LENGTH + 1];
    let mut wide_size: Dword = (MAX_COMPUTERNAME_LENGTH + 1) as Dword;

    if dns_hostname_to_computer_namew(wide_host.as_ptr(), wide_result.as_mut_ptr(), &mut wide_size) == FALSE {
        return FALSE;
    }

    unsafe {
        let src_len = wcslen(wide_result.as_ptr());
        let mut dst_len: Uint = 0;
        for i in 0..src_len {
            let c = wide_result[i] as u8;
            if c > 127 {
                dst_len += 2;
            } else {
                dst_len += 1;
            }
        }

        if (*size as Uint) <= dst_len {
            *size = dst_len + 1;
            return TRUE;
        }

        if !computername.is_null() {
            let mut out_pos: usize = 0;
            for i in 0..src_len {
                let c = wide_result[i] as u8;
                if c > 127 {
                    *computername.add(out_pos) = b'?';
                    out_pos += 1;
                } else {
                    *computername.add(out_pos) = c;
                    out_pos += 1;
                }
            }
            *computername.add(out_pos) = 0;
        }
        *size = src_len as Dword;
    }
    TRUE
}
