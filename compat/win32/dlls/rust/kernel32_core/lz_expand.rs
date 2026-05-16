// SPDX-License-Identifier: GPL-3.0-only

use alloc::vec::Vec;
use core::ptr;

type Bool = i32;
type Byte = u8;
type Int = i32;
type Uint = u32;
type Dword = u32;
type Word = u16;
type Long = i32;
type Hfile = i32;
type Lpstr = *mut u8;
type Lpcstr = *const u8;
type Lpwstr = *mut u16;
type Lpcwstr = *const u16;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const LZERROR_BADINHANDLE: Int = -1;
const LZERROR_BADOUTHANDLE: Int = -2;
const LZERROR_READ: Int = -3;
const LZERROR_WRITE: Int = -4;
const LZERROR_GLOBALLOC: Int = -5;
const LZERROR_GLOBLOCK: Int = -6;
const LZERROR_BADVALUE: Int = -7;
const LZERROR_UNKNOWNALG: Int = -8;

const LZ_MAGIC_LEN: usize = 8;
const LZ_HEADER_LEN: usize = 14;
const LZ_TABLE_SIZE: usize = 0x1000;
const LZ_MIN_HANDLE: i32 = 0x400;
const MAX_LZSTATES: usize = 16;
const GETLEN: usize = 2048;

const LZ_MAGIC: [Byte; 8] = [b'S', b'Z', b'D', b'D', 0x88, 0xf0, 0x27, 0x33];

fn set_last_error(_code: Dword) {}

#[derive(Clone, Copy)]
#[repr(C)]
struct LzFileHeader {
    magic: [Byte; 8],
    compression_type: Byte,
    last_char: Byte,
    real_length: Dword,
}

#[repr(C)]
struct LzState {
    real_fd: Hfile,
    last_char: Byte,
    real_length: Dword,
    real_current: Dword,
    real_wanted: Dword,
    table: [Byte; LZ_TABLE_SIZE],
    cur_tab_ent: Uint,
    string_len: Byte,
    string_pos: Dword,
    byte_type: Word,
    get_buf: [Byte; GETLEN],
    get_cur: Dword,
    get_len: Dword,
}

static mut LZ_STATES: [*mut LzState; MAX_LZSTATES] = [
    ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
    ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
    ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
    ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
];

fn is_lz_handle(fd: Hfile) -> bool {
    fd >= LZ_MIN_HANDLE && (fd as usize) < LZ_MIN_HANDLE as usize + MAX_LZSTATES
}

fn get_lz_state(fd: Hfile) -> Option<&'static mut LzState> {
    if is_lz_handle(fd) {
        let idx = (fd - LZ_MIN_HANDLE) as usize;
        unsafe {
            let ptr = LZ_STATES[idx];
            if !ptr.is_null() {
                Some(&mut *ptr)
            } else {
                None
            }
        }
    } else {
        None
    }
}

fn _lzget(lzs: &mut LzState, b: &mut Byte) -> Int {
    if lzs.get_cur < lzs.get_len {
        *b = lzs.get_buf[lzs.get_cur as usize];
        lzs.get_cur += 1;
        1
    } else {
        let ret = _lread(lzs.real_fd, &mut lzs.get_buf, GETLEN);
        if ret < 0 {
            return ret;
        }
        if ret == 0 {
            return 0;
        }
        lzs.get_len = ret as Dword;
        lzs.get_cur = 1;
        *b = lzs.get_buf[0];
        1
    }
}

fn _lzget_flush(lzs: &mut LzState) {
    lzs.get_cur = lzs.get_len;
}

fn read_header(fd: Hfile, head: &mut LzFileHeader) -> Int {
    let mut buf: [Byte; LZ_HEADER_LEN] = [0; LZ_HEADER_LEN];

    if _llseek(fd, 0, 0) < 0 {
        return LZERROR_BADINHANDLE;
    }

    if _lread(fd, &mut buf, LZ_HEADER_LEN) != LZ_HEADER_LEN as Int {
        return 0;
    }

    for i in 0..LZ_MAGIC_LEN {
        head.magic[i] = buf[i];
    }
    head.compression_type = buf[LZ_MAGIC_LEN];
    head.last_char = buf[LZ_MAGIC_LEN + 1];

    head.real_length = (buf[LZ_MAGIC_LEN + 2] as Dword)
        | ((buf[LZ_MAGIC_LEN + 3] as Dword) << 8)
        | ((buf[LZ_MAGIC_LEN + 4] as Dword) << 16)
        | ((buf[LZ_MAGIC_LEN + 5] as Dword) << 24);

    if head.magic != LZ_MAGIC {
        return 0;
    }
    if head.compression_type != b'A' {
        return LZERROR_UNKNOWNALG;
    }
    1
}

/* === Stub file I/O wrappers — replaced by ÖZKAN-OS VFS === */

fn _lread(_fd: Hfile, _buf: &mut [Byte], _count: usize) -> Int {
    0
}

fn _lwrite(_fd: Hfile, _buf: &[Byte], _count: usize) -> Int {
    0
}

fn _llseek(_fd: Hfile, _offset: Long, _whence: Int) -> Long {
    0
}

fn _lclose(_fd: Hfile) {
}

fn open_file(_name: Lpcstr, _ofs: *mut u8, _mode: Word) -> Hfile {
    -1
}

fn close_handle(_handle: Hfile) {
}

fn get_file_time(_fd: Hfile, _ctime: *mut u64, _atime: *mut u64, _mtime: *mut u64) -> Bool {
    FALSE
}

fn set_file_time(_fd: Hfile, _ctime: *const u64, _atime: *const u64, _mtime: *const u64) -> Bool {
    FALSE
}

fn heap_alloc_lz(size: usize) -> *mut u8 {
    /* In kernel context, use a static bump allocator for LZ state to avoid recursion */
    static mut LZ_ALLOC_BUF: [u8; 32768] = [0; 32768];
    static mut LZ_ALLOC_OFF: usize = 0;

    unsafe {
        let off = LZ_ALLOC_OFF;
        if off + size > LZ_ALLOC_BUF.len() {
            return ptr::null_mut();
        }
        LZ_ALLOC_OFF = off + size;
        LZ_ALLOC_BUF.as_mut_ptr().add(off)
    }
}

fn heap_free_lz(_ptr: *mut u8) {
    /* Bump allocator — no individual frees */
}

/* === Public API === */

pub fn lz_start() -> Int {
    1
}

pub fn lz_init(hf_src: Hfile) -> Hfile {
    let mut head = LzFileHeader {
        magic: [0; 8],
        compression_type: 0,
        last_char: 0,
        real_length: 0,
    };

    let ret = read_header(hf_src, &mut head);
    if ret <= 0 {
        _llseek(hf_src, 0, 0);
        return if ret == 0 { hf_src } else { ret as Hfile };
    }

    let mut slot: Option<usize> = None;
    unsafe {
        for i in 0..MAX_LZSTATES {
            if LZ_STATES[i].is_null() {
                slot = Some(i);
                break;
            }
        }
    }

    match slot {
        None => LZERROR_GLOBALLOC as Hfile,
        Some(i) => {
            let mem = heap_alloc_lz(core::mem::size_of::<LzState>());
            if mem.is_null() {
                return LZERROR_GLOBALLOC as Hfile;
            }
            unsafe {
                let lzs = mem as *mut LzState;
                ptr::write(lzs, LzState {
                    real_fd: hf_src,
                    last_char: head.last_char,
                    real_length: head.real_length,
                    real_current: 0,
                    real_wanted: 0,
                    table: [b' '; LZ_TABLE_SIZE],
                    cur_tab_ent: 0xff0,
                    string_len: 0,
                    string_pos: 0,
                    byte_type: 0,
                    get_buf: [0; GETLEN],
                    get_cur: 0,
                    get_len: 0,
                });
                LZ_STATES[i] = lzs;
            }
            (LZ_MIN_HANDLE + i as i32) as Hfile
        }
    }
}

pub fn lz_done() {
}

pub fn get_expanded_name_a(input: Lpstr, output: Lpstr) -> Int {
    if input.is_null() || output.is_null() {
        return LZERROR_BADINHANDLE;
    }

    let mut head = LzFileHeader {
        magic: [0; 8],
        compression_type: 0,
        last_char: 0,
        real_length: 0,
    };

    let fd = open_file(input, core::ptr::null_mut(), 0);
    if fd < 0 {
        return LZERROR_BADINHANDLE;
    }

    unsafe {
        let mut i: usize = 0;
        loop {
            let c = *input.add(i);
            *output.add(i) = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
    }

    let ret = read_header(fd, &mut head);
    if ret <= 0 {
        _lclose(fd);
        return 1;
    }

    unsafe {
        /* Find basename (skip directory prefix) */
        let mut s = output;
        loop {
            let mut found = false;
            let mut t = s;
            while *t != 0 {
                if *t == b'/' || *t == b'\\' || *t == b':' {
                    s = t.add(1);
                    found = true;
                    break;
                }
                t = t.add(1);
            }
            if !found {
                break;
            }
        }

        if *s == 0 {
            _lclose(fd);
            return 1;
        }

        /* Determine case of first alpha character in filename */
        let mut is_lowercase: Bool = 1;
        let mut len: usize = 0;
        while *s.add(len) != 0 { len += 1; }

        if len > 0 {
            let mut found_alpha = false;
            for i in (0..len).rev() {
                let c = *s.add(i);
                if c.is_ascii_alphabetic() {
                    is_lowercase = if c.is_ascii_lowercase() { 1 } else { 0 };
                    found_alpha = true;
                    break;
                }
            }
        }

        if head.last_char.is_ascii_alphabetic() {
            head.last_char = if is_lowercase != 0 {
                head.last_char.to_ascii_lowercase()
            } else {
                head.last_char.to_ascii_uppercase()
            };
        }

        /* Find last dot */
        let mut dot_pos: Option<usize> = None;
        for i in 0..len {
            if *s.add(i) == b'.' {
                dot_pos = Some(i);
            }
        }

        if let Some(dp) = dot_pos {
            if *s.add(dp + 1) == 0 {
                *s.add(dp) = 0;
            } else {
                let ext_len = len - dp - 1;
                if ext_len > 0 && *s.add(len - 1) == b'_' {
                    *s.add(len - 1) = head.last_char;
                }
            }
        }
    }

    _lclose(fd);
    1
}

pub fn get_expanded_name_w(input: Lpwstr, output: Lpwstr) -> Int {
    if input.is_null() || output.is_null() {
        return LZERROR_BADINHANDLE;
    }

    unsafe {
        let in_len: usize = {
            let mut i: usize = 0;
            while *input.add(i) != 0 { i += 1; }
            i
        };

        let mut xin: Vec<u8> = Vec::with_capacity(in_len + 1);
        for i in 0..in_len {
            let c = *input.add(i);
            xin.push(if c > 255 { b'?' } else { c as u8 });
        }
        xin.push(0);

        let mut xout: Vec<u8> = Vec::with_capacity(in_len + 4);
        xout.resize(in_len + 4, 0);

        let ret = get_expanded_name_a(xin.as_mut_ptr(), xout.as_mut_ptr());

        if ret > 0 {
            for i in 0..(in_len + 3) {
                if xout[i] == 0 {
                    *output.add(i) = 0;
                    break;
                }
                *output.add(i) = xout[i] as u16;
            }
        }

        ret
    }
}

fn decompress_one_byte(lzs: &mut LzState, b: &mut Byte) -> Int {
    if lzs.string_len != 0 {
        *b = lzs.table[lzs.string_pos as usize];
        lzs.string_pos = (lzs.string_pos + 1) & 0xFFF;
        lzs.string_len -= 1;
    } else {
        if (lzs.byte_type & 0x100) == 0 {
            if _lzget(lzs, b) != 1 {
                return 0;
            }
            lzs.byte_type = (*b as Word) | 0xFF00;
        }
        if (lzs.byte_type & 1) != 0 {
            if _lzget(lzs, b) != 1 {
                return 0;
            }
        } else {
            let mut b1: Byte = 0;
            let mut b2: Byte = 0;
            if _lzget(lzs, &mut b1) != 1 { return 0; }
            if _lzget(lzs, &mut b2) != 1 { return 0; }
            lzs.string_pos = b1 as Dword | ((b2 as Dword & 0xf0) << 4);
            lzs.string_len = (b2 & 0x0f) + 2;
            *b = lzs.table[lzs.string_pos as usize];
            lzs.string_pos = (lzs.string_pos + 1) & 0xFFF;
        }
        lzs.byte_type >>= 1;
    }

    lzs.table[lzs.cur_tab_ent as usize] = *b;
    lzs.cur_tab_ent = (lzs.cur_tab_ent + 1) & 0xFFF;
    lzs.real_current += 1;
    1
}

pub fn lz_read(fd: Hfile, vbuf: Lpstr, toread: Int) -> Int {
    if let Some(lzs) = get_lz_state(fd) {
        let mut howmuch = toread;
        let buf = vbuf as *mut Byte;

        /* Seek to wanted position if needed */
        if lzs.real_current != lzs.real_wanted {
            if lzs.real_current > lzs.real_wanted {
                _llseek(lzs.real_fd, LZ_HEADER_LEN as Long, 0);
                _lzget_flush(lzs);
                lzs.real_current = 0;
                lzs.byte_type = 0;
                lzs.string_len = 0;
                for i in 0..LZ_TABLE_SIZE {
                    lzs.table[i] = b' ';
                }
                lzs.cur_tab_ent = 0xFF0;
            }
            while lzs.real_current < lzs.real_wanted {
                let mut _b: Byte = 0;
                decompress_one_byte(lzs, &mut _b);
            }
        }

        let mut out_pos: usize = 0;
        while howmuch > 0 {
            let mut b: Byte = 0;
            decompress_one_byte(lzs, &mut b);
            lzs.real_wanted += 1;
            unsafe { *buf.add(out_pos) = b; }
            out_pos += 1;
            howmuch -= 1;
        }

        toread
    } else {
        _lread(fd, unsafe {
            core::slice::from_raw_parts_mut(vbuf as *mut Byte, toread as usize)
        }, toread as usize) as Int
    }
}

pub fn lz_seek(fd: Hfile, off: Long, type_: Int) -> Long {
    if let Some(lzs) = get_lz_state(fd) {
        let mut new_wanted = lzs.real_wanted as Long;
        match type_ {
            1 => new_wanted = new_wanted.wrapping_add(off),
            2 => new_wanted = (lzs.real_length as Long).wrapping_sub(off),
            _ => new_wanted = off,
        }
        if new_wanted > lzs.real_length as Long {
            return LZERROR_BADVALUE as Long;
        }
        if new_wanted < 0 {
            return LZERROR_BADVALUE as Long;
        }
        lzs.real_wanted = new_wanted as Dword;
        new_wanted
    } else {
        _llseek(fd, off, type_)
    }
}

pub fn lz_copy(src: Hfile, dest: Hfile) -> Long {
    let old_src = src;
    let mut used_lz_init: Bool = FALSE;
    let mut src_fd = src;

    if !is_lz_handle(src) {
        src_fd = lz_init(src);
        if src_fd <= 0 {
            return 0;
        }
        if src_fd != old_src {
            used_lz_init = TRUE;
        }
    }

    let buf_size = 1000usize;
    let mut len: Long = 0;
    let mut buf: [u8; 1000] = [0; 1000];

    loop {
        let ret = if is_lz_handle(src_fd) {
            lz_read(src_fd, buf.as_mut_ptr() as Lpstr, buf_size as Int)
        } else {
            _lread(src_fd, &mut buf, buf_size)
        };

        if ret <= 0 {
            if ret == 0 {
                break;
            }
            if ret == -1 {
                return LZERROR_READ as Long;
            }
            return ret as Long;
        }

        len += ret as Long;
        let wret = _lwrite(dest, &buf, ret as usize);
        if wret != ret {
            return LZERROR_WRITE as Long;
        }
    }

    let real_fd = if let Some(lzs) = get_lz_state(src_fd) {
        lzs.real_fd
    } else {
        src_fd
    };

    let mut filetime: u64 = 0;
    get_file_time(real_fd, core::ptr::null_mut(), core::ptr::null_mut(), &mut filetime);
    set_file_time(dest, core::ptr::null_mut(), core::ptr::null_mut(), &mut filetime);

    if used_lz_init != FALSE {
        lz_close(src_fd);
    }

    len
}

fn lzexpand_mangle_name(fn_: Lpcstr) -> Option<Vec<u8>> {
    unsafe {
        let mut len: usize = 0;
        while *fn_.add(len) != 0 { len += 1; }

        let mut mfn: Vec<u8> = Vec::with_capacity(len + 3);
        for i in 0..len {
            mfn.push(*fn_.add(i));
        }

        /* Find basename start */
        let mut base_start: usize = 0;
        for i in (0..len).rev() {
            if mfn[i] == b'\\' {
                base_start = i + 1;
                break;
            }
        }

        /* Find dot in basename */
        let mut dot_pos: Option<usize> = None;
        for i in base_start..len {
            if mfn[i] == b'.' {
                dot_pos = Some(i);
                break;
            }
        }

        match dot_pos {
            Some(dp) => {
                let ext_len = len - dp - 1;
                if ext_len < 3 {
                    mfn.push(b'_');
                } else {
                    mfn[len - 1] = b'_';
                }
            }
            None => {
                mfn.push(b'.');
                mfn.push(b'_');
            }
        }
        mfn.push(0);

        Some(mfn)
    }
}

pub fn lz_open_filea(fn_: Lpstr, ofs: *mut u8, mode: Word) -> Hfile {
    let ofs_c_bytes = if !ofs.is_null() {
        unsafe { *ofs }
    } else {
        0
    };

    let mut fd = open_file(fn_, ofs, mode);

    if fd < 0 {
        if let Some(mfn) = lzexpand_mangle_name(fn_) {
            fd = open_file(mfn.as_ptr() as Lpcstr, ofs, mode);
        }
    }

    if fd < 0 && !ofs.is_null() {
        unsafe { *ofs = ofs_c_bytes; }
    }

    if (mode & !0x70) != 0 {
        return fd;
    }

    if fd < 0 {
        return -1;
    }

    let cfd = lz_init(fd);
    if cfd <= 0 {
        fd
    } else {
        cfd
    }
}

pub fn lz_open_filew(fn_: Lpwstr, ofs: *mut u8, mode: Word) -> Hfile {
    unsafe {
        let mut in_len: usize = 0;
        while *fn_.add(in_len) != 0 { in_len += 1; }

        let mut xfn: Vec<u8> = Vec::with_capacity(in_len + 1);
        for i in 0..in_len {
            let c = *fn_.add(i);
            xfn.push(if c > 255 { b'?' } else { c as u8 });
        }
        xfn.push(0);

        lz_open_filea(xfn.as_mut_ptr() as Lpstr, ofs, mode)
    }
}

pub fn lz_close(fd: Hfile) {
    if let Some(lzs) = get_lz_state(fd) {
        let real_fd = lzs.real_fd;
        close_handle(real_fd);
        let idx = (fd - LZ_MIN_HANDLE) as usize;
        heap_free_lz(lzs as *mut LzState as *mut u8);
        unsafe {
            LZ_STATES[idx] = ptr::null_mut();
        }
    } else {
        _lclose(fd);
    }
}
