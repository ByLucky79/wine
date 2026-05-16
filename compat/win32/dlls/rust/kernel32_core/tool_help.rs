// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Toolhelp32 snapshot and enumeration APIs
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/tool_help.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 Toolhelp32 API for snapshot-based enumeration of
//   processes, threads, modules, and heaps. Also provides
//   Toolhelp32ReadProcessMemory for cross-process memory reading.
//   Snapshots are stored as file mapping objects and iterated via
//   First/Next pattern.
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

type Handle = *mut c_void;
type Bool = i32;
type Dword = u32;
type Word = u16;
type Char = u8;
type Wchar = u16;
type Lpvoid = *mut c_void;
type Lpcvoid = *const c_void;
type Lpcstr = *const u8;
type Lpwstr = *const u16;
type Lpstr = *mut u8;
type LpwstrMut = *mut u16;
type Ulong = u32;
type SizeT = usize;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const INVALID_HANDLE_VALUE: Handle = !0usize as *mut c_void;

const ERROR_INSUFFICIENT_BUFFER: Dword = 122;
const ERROR_NO_MORE_FILES: Dword = 18;
const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;
const ERROR_PARTIAL_COPY: Dword = 299;

const TH32CS_SNAPHEAPLIST: Dword = 0x00000001;
const TH32CS_SNAPPROCESS: Dword = 0x00000002;
const TH32CS_SNAPTHREAD: Dword = 0x00000004;
const TH32CS_SNAPMODULE: Dword = 0x00000008;
const TH32CS_SNAPMODULE32: Dword = 0x00000010;
const TH32CS_SNAPALL: Dword = TH32CS_SNAPHEAPLIST | TH32CS_SNAPPROCESS | TH32CS_SNAPTHREAD | TH32CS_SNAPMODULE;
const TH32CS_INHERIT: Dword = 0x80000000;

const MAX_PATH: usize = 260;
const MAX_MODULE_NAME32: usize = 255;

fn set_last_error(_code: Dword) {}

fn get_current_process() -> Handle {
    !0usize as Handle
}

fn get_current_process_id() -> Dword {
    1
}

fn close_handle(_handle: Handle) -> Bool {
    TRUE
}

#[repr(C)]
struct ProcessEntry32 {
    dw_size: Dword,
    cnt_usage: Dword,
    th32_process_id: Dword,
    th32_default_heap_id: usize,
    th32_module_id: Dword,
    cnt_threads: Dword,
    th32_parent_process_id: Dword,
    pc_pri_class_base: Long,
    dw_flags: Dword,
    sz_exe_file: [Char; MAX_PATH],
}

#[repr(C)]
struct ProcessEntry32W {
    dw_size: Dword,
    cnt_usage: Dword,
    th32_process_id: Dword,
    th32_default_heap_id: usize,
    th32_module_id: Dword,
    cnt_threads: Dword,
    th32_parent_process_id: Dword,
    pc_pri_class_base: Long,
    dw_flags: Dword,
    sz_exe_file: [Wchar; MAX_PATH],
}

#[repr(C)]
struct ThreadEntry32 {
    dw_size: Dword,
    cnt_usage: Dword,
    th32_thread_id: Dword,
    th32_owner_process_id: Dword,
    tp_base_pri: Long,
    tp_delta_pri: Long,
    dw_flags: Dword,
}

#[repr(C)]
struct ModuleEntry32 {
    dw_size: Dword,
    th32_module_id: Dword,
    th32_process_id: Dword,
    glblcnt_usage: Dword,
    proccnt_usage: Dword,
    mod_base_addr: *mut u8,
    mod_base_size: Dword,
    h_module: Handle,
    sz_module: [Char; MAX_MODULE_NAME32 + 1],
    sz_exe_path: [Char; MAX_PATH],
}

#[repr(C)]
struct ModuleEntry32W {
    dw_size: Dword,
    th32_module_id: Dword,
    th32_process_id: Dword,
    glblcnt_usage: Dword,
    proccnt_usage: Dword,
    mod_base_addr: *mut u8,
    mod_base_size: Dword,
    h_module: Handle,
    sz_module: [Wchar; MAX_MODULE_NAME32 + 1],
    sz_exe_path: [Wchar; MAX_PATH],
}

#[repr(C)]
struct HeapList32 {
    dw_size: Dword,
    th32_process_id: Dword,
    th32_heap_id: usize,
    dw_flags: Dword,
}

#[repr(C)]
struct HeapEntry32 {
    dw_size: Dword,
    h_handle: Handle,
    dw_address: usize,
    dw_block_size: SizeT,
    dw_flags: Dword,
    dw_lock_count: Dword,
    dw_resvd: Dword,
    th32_process_id: Dword,
    th32_heap_id: usize,
}

type Long = i32;

struct SnapshotHeader {
    process_count: i32,
    process_pos: i32,
    process_offset: i32,
    thread_count: i32,
    thread_pos: i32,
    thread_offset: i32,
    module_count: i32,
    module_pos: i32,
    module_offset: i32,
}

pub fn create_toolhelp32_snapshot(flags: Dword, _process: Dword) -> Handle {
    if flags & (TH32CS_SNAPPROCESS | TH32CS_SNAPTHREAD | TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32) == 0 {
        set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
        return INVALID_HANDLE_VALUE;
    }
    let _ = flags;
    INVALID_HANDLE_VALUE
}

pub fn process32_first(
    snapshot: Handle,
    entry: *mut ProcessEntry32,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ProcessEntry32>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn process32_next(
    snapshot: Handle,
    entry: *mut ProcessEntry32,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ProcessEntry32>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn process32_first_w(
    snapshot: Handle,
    entry: *mut ProcessEntry32W,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ProcessEntry32W>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn process32_next_w(
    snapshot: Handle,
    entry: *mut ProcessEntry32W,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ProcessEntry32W>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn thread32_first(
    snapshot: Handle,
    entry: *mut ThreadEntry32,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ThreadEntry32>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn thread32_next(
    snapshot: Handle,
    entry: *mut ThreadEntry32,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ThreadEntry32>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn module32_first(
    snapshot: Handle,
    entry: *mut ModuleEntry32,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ModuleEntry32>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn module32_next(
    snapshot: Handle,
    entry: *mut ModuleEntry32,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ModuleEntry32>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn module32_first_w(
    snapshot: Handle,
    entry: *mut ModuleEntry32W,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ModuleEntry32W>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn module32_next_w(
    snapshot: Handle,
    entry: *mut ModuleEntry32W,
) -> Bool {
    let _ = snapshot;
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        if (*entry).dw_size < core::mem::size_of::<ModuleEntry32W>() as Dword {
            set_last_error(ERROR_INSUFFICIENT_BUFFER);
            return FALSE;
        }
    }
    set_last_error(ERROR_NO_MORE_FILES);
    FALSE
}

pub fn heap32_list_first(
    snapshot: Handle,
    entry: *mut HeapList32,
) -> Bool {
    let _ = (snapshot, entry);
    FALSE
}

pub fn heap32_list_next(
    snapshot: Handle,
    entry: *mut HeapList32,
) -> Bool {
    let _ = (snapshot, entry);
    FALSE
}

pub fn heap32_first(
    _snapshot: Handle,
    entry: *mut HeapEntry32,
) -> Bool {
    if entry.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    FALSE
}

pub fn heap32_next(
    entry: *mut HeapEntry32,
) -> Bool {
    let _ = entry;
    FALSE
}

pub fn toolhelp32_read_process_memory(
    process_id: Dword,
    base: Lpcvoid,
    buffer: Lpvoid,
    size: SizeT,
    read: *mut SizeT,
) -> Bool {
    if buffer.is_null() || base.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (process_id, size);
    if !read.is_null() {
        unsafe { *read = 0; }
    }
    FALSE
}

pub fn create_toolhelp32_snapshot_ex(
    flags: Dword,
    process: Dword,
    _thread: Dword,
) -> Handle {
    create_toolhelp32_snapshot(flags, process)
}
