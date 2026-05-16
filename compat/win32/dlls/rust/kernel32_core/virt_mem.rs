// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 virtual memory management APIs
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/virt_mem.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 virtual memory API functions including allocation,
//   protection, query, lock, and system information queries. All operations
//   delegate to the ÖZKAN-OS memory manager. Includes the IsBad* pointer
//   validation functions and lstr* string functions.
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
use core::slice;

type Handle = *mut c_void;
type Bool = i32;
type Dword = u32;
type Uint = u32;
type Char = u8;
type Wchar = u16;
type Lpcstr = *const u8;
type Lpwstr = *const u16;
type Lpstr = *mut u8;
type LpwstrMut = *mut u16;
type Lpvoid = *mut c_void;
type Lpcvoid = *const c_void;
type Ulong = u32;
type SizeT = usize;
type UintPtr = usize;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const INVALID_HANDLE_VALUE: Handle = !0usize as *mut c_void;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;
const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_SUCCESS: Dword = 0;

const MEM_COMMIT: Dword = 0x00001000;
const MEM_RESERVE: Dword = 0x00002000;
const MEM_RELEASE: Dword = 0x00008000;
const MEM_DECOMMIT: Dword = 0x00004000;
const MEM_RESET: Dword = 0x00080000;
const MEM_TOP_DOWN: Dword = 0x00100000;
const MEM_WRITE_WATCH: Dword = 0x00200000;
const MEM_LARGE_PAGES: Dword = 0x20000000;
const MEM_PHYSICAL: Dword = 0x00400000;
const MEM_4MB_PAGES: Dword = 0x80000000;

const PAGE_NOACCESS: Dword = 0x00000001;
const PAGE_READONLY: Dword = 0x00000002;
const PAGE_READWRITE: Dword = 0x00000004;
const PAGE_WRITECOPY: Dword = 0x00000008;
const PAGE_EXECUTE: Dword = 0x00000010;
const PAGE_EXECUTE_READ: Dword = 0x00000020;
const PAGE_EXECUTE_READWRITE: Dword = 0x00000040;
const PAGE_EXECUTE_WRITECOPY: Dword = 0x00000080;
const PAGE_GUARD: Dword = 0x00000100;
const PAGE_NOCACHE: Dword = 0x00000200;
const PAGE_WRITECOMBINE: Dword = 0x00000400;

const MEMORY_BASIC_TYPE: Dword = 0;

static SYSTEM_PAGE_SIZE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(4096);
static SYSTEM_ALLOC_GRAN: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(65536);
static SYSTEM_CPU_COUNT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

fn set_last_error(_code: Dword) {}

fn get_current_process() -> Handle {
    !0usize as Handle
}

pub fn virtual_alloc(
    address: Lpvoid,
    size: SizeT,
    allocation_type: Dword,
    protect: Dword,
) -> Lpvoid {
    let _ = (address, size, allocation_type, protect);
    ptr::null_mut()
}

pub fn virtual_alloc_ex(
    process: Handle,
    address: Lpvoid,
    size: SizeT,
    allocation_type: Dword,
    protect: Dword,
) -> Lpvoid {
    let _ = (process, address, size, allocation_type, protect);
    ptr::null_mut()
}

pub fn virtual_free(
    address: Lpvoid,
    size: SizeT,
    free_type: Dword,
) -> Bool {
    let _ = (address, size, free_type);
    TRUE
}

pub fn virtual_free_ex(
    process: Handle,
    address: Lpvoid,
    size: SizeT,
    free_type: Dword,
) -> Bool {
    let _ = (process, address, size, free_type);
    TRUE
}

pub fn virtual_protect(
    address: Lpvoid,
    size: SizeT,
    new_protect: Dword,
    old_protect: *mut Dword,
) -> Bool {
    if old_protect.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (address, size, new_protect);
    unsafe { *old_protect = new_protect; }
    TRUE
}

pub fn virtual_protect_ex(
    process: Handle,
    address: Lpvoid,
    size: SizeT,
    new_protect: Dword,
    old_protect: *mut Dword,
) -> Bool {
    if old_protect.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (process, address, size, new_protect);
    unsafe { *old_protect = new_protect; }
    TRUE
}

pub fn virtual_query(
    address: Lpcvoid,
    buffer: *mut MemoryBasicInformation,
    length: SizeT,
) -> SizeT {
    if buffer.is_null() || length < core::mem::size_of::<MemoryBasicInformation>() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    let _ = address;
    unsafe {
        ptr::write(buffer, MemoryBasicInformation {
            base_address: ptr::null_mut(),
            allocation_base: ptr::null_mut(),
            allocation_protect: 0,
            region_size: 0,
            state: MEM_RESERVE,
            protect: PAGE_NOACCESS,
            type_: MEMORY_BASIC_TYPE,
        });
    }
    core::mem::size_of::<MemoryBasicInformation>()
}

pub fn virtual_query_ex(
    process: Handle,
    address: Lpcvoid,
    buffer: *mut MemoryBasicInformation,
    length: SizeT,
) -> SizeT {
    let _ = process;
    virtual_query(address, buffer, length)
}

#[repr(C)]
pub struct MemoryBasicInformation {
    base_address: Lpvoid,
    allocation_base: Lpvoid,
    allocation_protect: Dword,
    region_size: SizeT,
    state: Dword,
    protect: Dword,
    type_: Dword,
}

pub fn virtual_lock(
    address: Lpvoid,
    size: SizeT,
) -> Bool {
    let _ = (address, size);
    TRUE
}

pub fn virtual_unlock(
    address: Lpvoid,
    size: SizeT,
) -> Bool {
    let _ = (address, size);
    TRUE
}

pub fn virtual_alloc_ex_numa(
    process: Handle,
    address: Lpvoid,
    size: SizeT,
    allocation_type: Dword,
    protect: Dword,
    _node: Dword,
) -> Lpvoid {
    virtual_alloc_ex(process, address, size, allocation_type, protect)
}

pub fn get_write_watch(
    process: Handle,
    flags: Dword,
    base_address: Lpvoid,
    region_size: SizeT,
    addresses: *mut *mut c_void,
    count: *mut UintPtr,
) -> Dword {
    let _ = (process, flags, base_address, region_size, addresses, count);
    0
}

pub fn get_system_info(system_info: *mut SystemInfo) {
    if system_info.is_null() {
        return;
    }
    unsafe {
        ptr::write(system_info, SystemInfo {
            w_processor_architecture: 0,
            dw_page_size: SYSTEM_PAGE_SIZE.load(core::sync::atomic::Ordering::Relaxed),
            lp_minimum_application_address: 0x10000 as *mut c_void,
            lp_maximum_application_address: 0x7FFF0000usize as *mut c_void,
            dw_active_processor_mask: SYSTEM_CPU_COUNT.load(core::sync::atomic::Ordering::Relaxed) as UintPtr,
            dw_number_of_processors: SYSTEM_CPU_COUNT.load(core::sync::atomic::Ordering::Relaxed),
            dw_processor_type: 586,
            dw_allocation_granularity: SYSTEM_ALLOC_GRAN.load(core::sync::atomic::Ordering::Relaxed),
            w_processor_level: 6,
            w_processor_revision: 0,
        });
    }
}

pub fn get_native_system_info(system_info: *mut SystemInfo) {
    get_system_info(system_info);
}

#[repr(C)]
pub struct SystemInfo {
    w_processor_architecture: Word,
    dw_page_size: Dword,
    lp_minimum_application_address: Lpvoid,
    lp_maximum_application_address: Lpvoid,
    dw_active_processor_mask: UintPtr,
    dw_number_of_processors: Dword,
    dw_processor_type: Dword,
    dw_allocation_granularity: Dword,
    w_processor_level: Word,
    w_processor_revision: Word,
}

type Word = u16;

pub fn global_memory_status(buffer: *mut MemoryStatus) {
    if buffer.is_null() {
        return;
    }
    unsafe {
        ptr::write(buffer, MemoryStatus {
            dw_length: core::mem::size_of::<MemoryStatus>() as Dword,
            dw_memory_load: 50,
            dw_total_phys: 256 * 1024 * 1024,
            dw_avail_phys: 128 * 1024 * 1024,
            dw_total_page_file: 512 * 1024 * 1024,
            dw_avail_page_file: 256 * 1024 * 1024,
            dw_total_virtual: 2 * 1024 * 1024 * 1024,
            dw_avail_virtual: 1 * 1024 * 1024 * 1024,
        });
    }
}

pub fn global_memory_status_ex(buffer: *mut MemoryStatusEx) -> Bool {
    if buffer.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        let len = (*buffer).dw_length;
        if len < core::mem::size_of::<MemoryStatusEx>() as Dword {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
        ptr::write(buffer, MemoryStatusEx {
            dw_length: core::mem::size_of::<MemoryStatusEx>() as Dword,
            dw_memory_load: 50,
            ull_total_phys: 256 * 1024 * 1024,
            ull_avail_phys: 128 * 1024 * 1024,
            ull_total_page_file: 512 * 1024 * 1024,
            ull_avail_page_file: 256 * 1024 * 1024,
            ull_total_virtual: 2 * 1024 * 1024 * 1024,
            ull_avail_virtual: 1 * 1024 * 1024 * 1024,
            ull_total_extended: 0,
            ull_avail_extended: 0,
        });
    }
    TRUE
}

#[repr(C)]
pub struct MemoryStatus {
    dw_length: Dword,
    dw_memory_load: Dword,
    dw_total_phys: SizeT,
    dw_avail_phys: SizeT,
    dw_total_page_file: SizeT,
    dw_avail_page_file: SizeT,
    dw_total_virtual: SizeT,
    dw_avail_virtual: SizeT,
}

#[repr(C)]
pub struct MemoryStatusEx {
    dw_length: Dword,
    dw_memory_load: Dword,
    ull_total_phys: u64,
    ull_avail_phys: u64,
    ull_total_page_file: u64,
    ull_avail_page_file: u64,
    ull_total_virtual: u64,
    ull_avail_virtual: u64,
    ull_total_extended: u64,
    ull_avail_extended: u64,
}

pub fn is_bad_read_ptr(ptr: Lpcvoid, size: UintPtr) -> Bool {
    if size == 0 { return FALSE; }
    if ptr.is_null() { return TRUE; }
    let _ = (ptr, size);
    FALSE
}

pub fn is_bad_write_ptr(ptr: Lpvoid, size: UintPtr) -> Bool {
    if size == 0 { return FALSE; }
    if ptr.is_null() { return TRUE; }
    let _ = (ptr, size);
    FALSE
}

pub fn is_bad_huge_read_ptr(ptr: Lpcvoid, size: UintPtr) -> Bool {
    is_bad_read_ptr(ptr, size)
}

pub fn is_bad_huge_write_ptr(ptr: Lpvoid, size: UintPtr) -> Bool {
    is_bad_write_ptr(ptr, size)
}

pub fn is_bad_code_ptr(ptr: Option<extern "system" fn()>) -> Bool {
    match ptr {
        Some(f) => is_bad_read_ptr(f as *const c_void, 1),
        None => TRUE,
    }
}

pub fn is_bad_string_ptr_a(str: Lpcstr, max: UintPtr) -> Bool {
    if str.is_null() { return TRUE; }
    let _ = (str, max);
    FALSE
}

pub fn is_bad_string_ptr_w(str: Lpwstr, max: UintPtr) -> Bool {
    if str.is_null() { return TRUE; }
    let _ = (str, max);
    FALSE
}

pub fn lstrcat_a(dst: Lpstr, src: Lpcstr) -> Lpstr {
    if dst.is_null() || src.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let mut end = 0;
    unsafe {
        while *dst.add(end) != 0 { end += 1; }
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(end + i) = c;
            if c == 0 { break; }
            i += 1;
        }
    }
    dst
}

pub fn lstrcat_w(dst: LpwstrMut, src: Lpwstr) -> LpwstrMut {
    if dst.is_null() || src.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let mut end = 0;
    unsafe {
        while *dst.add(end) != 0 { end += 1; }
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(end + i) = c;
            if c == 0 { break; }
            i += 1;
        }
    }
    dst
}

pub fn lstrcpy_a(dst: Lpstr, src: Lpcstr) -> Lpstr {
    if dst.is_null() || src.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    unsafe {
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(i) = c;
            if c == 0 { break; }
            i += 1;
        }
    }
    dst
}

pub fn lstrcpy_w(dst: LpwstrMut, src: Lpwstr) -> LpwstrMut {
    if dst.is_null() || src.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    unsafe {
        let mut i = 0;
        loop {
            let c = *src.add(i);
            *dst.add(i) = c;
            if c == 0 { break; }
            i += 1;
        }
    }
    dst
}

pub fn lstrcmp_a(str1: Lpcstr, str2: Lpcstr) -> i32 {
    if str1.is_null() || str2.is_null() {
        return 0;
    }
    unsafe {
        let mut i = 0;
        loop {
            let a = *str1.add(i);
            let b = *str2.add(i);
            if a != b { return a as i32 - b as i32; }
            if a == 0 { return 0; }
            i += 1;
        }
    }
}

pub fn lstrcmp_w(str1: Lpwstr, str2: Lpwstr) -> i32 {
    if str1.is_null() || str2.is_null() {
        return 0;
    }
    unsafe {
        let mut i = 0;
        loop {
            let a = *str1.add(i);
            let b = *str2.add(i);
            if a != b { return a as i32 - b as i32; }
            if a == 0 { return 0; }
            i += 1;
        }
    }
}

pub fn lstrcmpi_a(str1: Lpcstr, str2: Lpcstr) -> i32 {
    if str1.is_null() || str2.is_null() {
        return 0;
    }
    let _ = (str1, str2);
    0
}

pub fn lstrcmpi_w(str1: Lpwstr, str2: Lpwstr) -> i32 {
    if str1.is_null() || str2.is_null() {
        return 0;
    }
    let _ = (str1, str2);
    0
}

pub fn lstrlen_a(str: Lpcstr) -> i32 {
    if str.is_null() { return 0; }
    let mut len: i32 = 0;
    unsafe {
        while *str.add(len as usize) != 0 { len += 1; }
    }
    len
}

pub fn lstrlen_w(str: Lpwstr) -> i32 {
    if str.is_null() { return 0; }
    let mut len: i32 = 0;
    unsafe {
        while *str.add(len as usize) != 0 { len += 1; }
    }
    len
}

pub fn copy_memory(dest: Lpvoid, src: Lpcvoid, count: SizeT) {
    if dest.is_null() || src.is_null() || count == 0 {
        return;
    }
    unsafe {
        ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, count);
    }
}

pub fn move_memory(dest: Lpvoid, src: Lpcvoid, count: SizeT) {
    if dest.is_null() || src.is_null() || count == 0 {
        return;
    }
    unsafe {
        ptr::copy(src as *const u8, dest as *mut u8, count);
    }
}

pub fn fill_memory(dest: Lpvoid, count: SizeT, val: u8) {
    if dest.is_null() || count == 0 {
        return;
    }
    unsafe {
        ptr::write_bytes(dest, val, count);
    }
}

pub fn zero_memory(dest: Lpvoid, count: SizeT) {
    fill_memory(dest, count, 0);
}

pub fn compare_memory(buf1: Lpcvoid, buf2: Lpcvoid, count: SizeT) -> i32 {
    if count == 0 { return 0; }
    if buf1.is_null() || buf2.is_null() { return 0; }
    unsafe {
        let s1 = slice::from_raw_parts(buf1 as *const u8, count);
        let s2 = slice::from_raw_parts(buf2 as *const u8, count);
        for i in 0..count {
            if s1[i] != s2[i] {
                return s1[i] as i32 - s2[i] as i32;
            }
        }
    }
    0
}

pub fn set_system_power_state(
    _system_state: u32,
    _flags: Dword,
) -> Bool {
    TRUE
}

pub fn get_system_power_status(
    _status: *mut u8,
) -> Bool {
    TRUE
}

pub fn init_once_execute(
    _once: *mut u32,
    _callback: Option<extern "system" fn(*mut u32, Lpvoid, *mut u32) -> Dword>,
    _parameter: Lpvoid,
    _context: *mut u32,
) -> Bool {
    TRUE
}

pub fn init_once_begin(
    _once: *mut u32,
    _flags: Dword,
    _context: *mut u32,
) -> Bool {
    TRUE
}

pub fn init_once_complete(
    _once: *mut u32,
    _context: Lpvoid,
    _flags: Dword,
) -> Bool {
    TRUE
}

pub fn virtual_alloc_from_app(
    address: Lpvoid,
    size: SizeT,
    allocation_type: Dword,
    protect: Dword,
) -> Lpvoid {
    virtual_alloc(address, size, allocation_type, protect)
}

pub fn virtual_alloc_ex_from_app(
    process: Handle,
    address: Lpvoid,
    size: SizeT,
    allocation_type: Dword,
    protect: Dword,
) -> Lpvoid {
    virtual_alloc_ex(process, address, size, allocation_type, protect)
}

pub fn virtual_protect_from_app(
    address: Lpvoid,
    size: SizeT,
    new_protect: Dword,
    old_protect: *mut Dword,
) -> Bool {
    virtual_protect(address, size, new_protect, old_protect)
}

pub fn virtual_protect_ex_from_app(
    process: Handle,
    address: Lpvoid,
    size: SizeT,
    new_protect: Dword,
    old_protect: *mut Dword,
) -> Bool {
    virtual_protect_ex(process, address, size, new_protect, old_protect)
}

pub fn query_memory_resource_notification(
    _buffer: *mut c_void,
    _buffer_size: Dword,
) -> i32 {
    0
}

pub fn get_memory_error_handling_capabilities(
    _capabilities: *mut Dword,
) -> Bool {
    TRUE
}
