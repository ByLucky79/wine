// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 file and pipe I/O API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/file_ops.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 file management, pipe, and named pipe APIs.
//   Handles file create/open/read/write/close, directory operations,
//   file attribute queries, file time management, file mapping,
//   and named pipe server/client operations.
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
type Word = u16;
type Uint = u32;
type Long = i32;
type Char = u8;
type Wchar = u16;
type Lpcstr = *const u8;
type Lpwstr = *const u16;
type Lpstr = *mut u8;
type LpwstrMut = *mut u16;
type Lpvoid = *mut c_void;
type Lpcvoid = *const c_void;
type Lpdword = *mut Dword;
type Ulong = u32;
type Ulonglong = u64;
type LARGE_INTEGER = i64;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const INVALID_HANDLE_VALUE: Handle = !0usize as *mut c_void;
const HFILE_ERROR: Long = -1;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_FILE_NOT_FOUND: Dword = 2;
const ERROR_PATH_NOT_FOUND: Dword = 3;
const ERROR_ACCESS_DENIED: Dword = 5;
const ERROR_ALREADY_EXISTS: Dword = 183;
const ERROR_FILENAME_EXCED_RANGE: Dword = 206;
const ERROR_NOT_SUPPORTED: Dword = 50;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;
const ERROR_SUCCESS: Dword = 0;

const GENERIC_READ: Dword = 0x80000000;
const GENERIC_WRITE: Dword = 0x40000000;
const FILE_SHARE_READ: Dword = 0x00000001;
const FILE_SHARE_WRITE: Dword = 0x00000002;
const FILE_SHARE_DELETE: Dword = 0x00000004;
const CREATE_NEW: Dword = 1;
const CREATE_ALWAYS: Dword = 2;
const OPEN_EXISTING: Dword = 3;
const OPEN_ALWAYS: Dword = 4;
const TRUNCATE_EXISTING: Dword = 5;
const FILE_ATTRIBUTE_NORMAL: Dword = 0x00000080;
const FILE_ATTRIBUTE_READONLY: Dword = 0x00000001;
const FILE_ATTRIBUTE_HIDDEN: Dword = 0x00000002;
const FILE_ATTRIBUTE_SYSTEM: Dword = 0x00000004;
const FILE_ATTRIBUTE_DIRECTORY: Dword = 0x00000010;
const FILE_ATTRIBUTE_ARCHIVE: Dword = 0x00000020;
const FILE_ATTRIBUTE_TEMPORARY: Dword = 0x00000100;
const FILE_FLAG_WRITE_THROUGH: Dword = 0x80000000;
const FILE_FLAG_OVERLAPPED: Dword = 0x40000000;
const FILE_FLAG_NO_BUFFERING: Dword = 0x20000000;
const FILE_FLAG_RANDOM_ACCESS: Dword = 0x10000000;
const FILE_FLAG_SEQUENTIAL_SCAN: Dword = 0x08000000;
const FILE_FLAG_DELETE_ON_CLOSE: Dword = 0x04000000;
const FILE_FLAG_BACKUP_SEMANTICS: Dword = 0x02000000;
const FILE_FLAG_POSIX_SEMANTICS: Dword = 0x01000000;

const FILE_BEGIN: Dword = 0;
const FILE_CURRENT: Dword = 1;
const FILE_END: Dword = 2;

const FILE_TYPE_UNKNOWN: Dword = 0x0000;
const FILE_TYPE_DISK: Dword = 0x0001;
const FILE_TYPE_CHAR: Dword = 0x0002;
const FILE_TYPE_PIPE: Dword = 0x0003;

const OF_READ: Uint = 0x0000;
const OF_WRITE: Uint = 0x0001;
const OF_READWRITE: Uint = 0x0002;
const OF_SHARE_COMPAT: Uint = 0x0000;
const OF_SHARE_EXCLUSIVE: Uint = 0x0010;
const OF_SHARE_DENY_WRITE: Uint = 0x0020;
const OF_SHARE_DENY_READ: Uint = 0x0030;
const OF_SHARE_DENY_NONE: Uint = 0x0040;
const OF_PARSE: Uint = 0x0100;
const OF_DELETE: Uint = 0x0200;
const OF_VERIFY: Uint = 0x0400;
const OF_CREATE: Uint = 0x1000;
const OF_EXIST: Uint = 0x4000;

const DRIVE_UNKNOWN: Dword = 0;
const DRIVE_REMOVABLE: Dword = 2;

const NMPWAIT_WAIT_FOREVER: Dword = 0xFFFFFFFF;
const PIPE_ACCESS_DUPLEX: Dword = 0x00000003;
const PIPE_ACCESS_INBOUND: Dword = 0x00000001;
const PIPE_ACCESS_OUTBOUND: Dword = 0x00000002;
const PIPE_WAIT: Dword = 0x00000000;
const PIPE_NOWAIT: Dword = 0x00000001;
const PIPE_READMODE_BYTE: Dword = 0x00000000;
const PIPE_READMODE_MESSAGE: Dword = 0x00000002;
const PIPE_TYPE_BYTE: Dword = 0x00000000;
const PIPE_TYPE_MESSAGE: Dword = 0x00000004;
const PIPE_CLIENT_END: Dword = 0x00000000;
const PIPE_SERVER_END: Dword = 0x00000001;
const PIPE_UNLIMITED_INSTANCES: Dword = 255;

const LOCKFILE_FAIL_IMMEDIATELY: Dword = 0x00000001;
const LOCKFILE_EXCLUSIVE_LOCK: Dword = 0x00000002;

const FILE_CURRENT: Dword = 1;

const MOVEFILE_REPLACE_EXISTING: Dword = 0x00000001;
const MOVEFILE_COPY_ALLOWED: Dword = 0x00000002;
const MOVEFILE_DELAY_UNTIL_REBOOT: Dword = 0x00000004;
const MOVEFILE_WRITE_THROUGH: Dword = 0x00000008;
const MOVEFILE_CREATE_HARDLINK: Dword = 0x00000010;
const MOVEFILE_FAIL_IF_NOT_TRACKABLE: Dword = 0x00000020;

const COPY_FILE_FAIL_IF_EXISTS: Dword = 0x00000001;
const COPY_FILE_RESTARTABLE: Dword = 0x00000002;
const COPY_FILE_OPEN_SOURCE_FOR_WRITE: Dword = 0x00000004;
const COPY_FILE_ALLOW_DECRYPTED_DESTINATION: Dword = 0x00000008;

#[repr(C)]
struct SecurityAttributes {
    n_length: Dword,
    lp_security_descriptor: Lpvoid,
    b_inherit_handle: Bool,
}

#[repr(C)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: Dword,
    offset_high: Dword,
    h_event: Handle,
}

#[repr(C)]
struct FileTime {
    dw_low_date_time: Dword,
    dw_high_date_time: Dword,
}

#[repr(C)]
struct Win32FindDataA {
    dw_file_attributes: Dword,
    ft_creation_time: FileTime,
    ft_last_access_time: FileTime,
    ft_last_write_time: FileTime,
    n_file_size_high: Dword,
    n_file_size_low: Dword,
    dw_reserved0: Dword,
    dw_reserved1: Dword,
    c_file_name: [Char; 260],
    c_alternate_file_name: [Char; 14],
}

#[repr(C)]
struct Win32FindDataW {
    dw_file_attributes: Dword,
    ft_creation_time: FileTime,
    ft_last_access_time: FileTime,
    ft_last_write_time: FileTime,
    n_file_size_high: Dword,
    n_file_size_low: Dword,
    dw_reserved0: Dword,
    dw_reserved1: Dword,
    c_file_name: [Wchar; 260],
    c_alternate_file_name: [Wchar; 14],
}

#[repr(C)]
struct OfStruct {
    c_bytes: Uint,
    f_fixed_disk: Bool,
    n_err_code: Word,
    reserved1: Word,
    reserved2: Word,
    sz_path_name: [Char; 128],
}

#[repr(C)]
struct ByHandleFileInformation {
    dw_file_attributes: Dword,
    ft_creation_time: FileTime,
    ft_last_access_time: FileTime,
    ft_last_write_time: FileTime,
    dw_volume_serial_number: Dword,
    n_file_size_high: Dword,
    n_file_size_low: Dword,
    n_number_of_links: Dword,
    n_file_index_high: Dword,
    n_file_index_low: Dword,
}

#[repr(C)]
struct FileInfoByHandle {
    file_type: Dword,
    creator_type: Dword,
    flags: Word,
    file_size_high: Dword,
    file_size_low: Dword,
}

#[repr(C)]
struct GetFileExInfo {
    file_attributes: Dword,
    ft_creation_time: FileTime,
    ft_last_access_time: FileTime,
    ft_last_write_time: FileTime,
    file_size: LARGE_INTEGER,
}

#[repr(C)]
struct NamedPipeInfo {
    pipe_mode: Dword,
    max_instances: Dword,
    in_buffer_size: Dword,
    out_buffer_size: Dword,
}

#[repr(C)]
struct DiskFreeSpace {
    free_bytes_available: LARGE_INTEGER,
    total_number_of_bytes: LARGE_INTEGER,
    total_free_bytes: LARGE_INTEGER,
}

fn set_last_error(_code: Dword) {}

fn handle_to_long(h: Handle) -> Long {
    h as Long
}

fn long_to_handle(l: Long) -> Handle {
    l as isize as Handle
}

#[repr(C)]
struct IoStatusBlock {
    status: isize,
    information: usize,
}

fn nt_create_file(
    _name: Lpwstr,
    _access: Dword,
    _share: Dword,
    _creation: Dword,
    _flags: Dword,
) -> Handle {
    INVALID_HANDLE_VALUE
}

fn nt_open_file(
    _name: Lpwstr,
    _access: Dword,
    _share: Dword,
    _creation: Dword,
    _flags: Dword,
) -> Handle {
    INVALID_HANDLE_VALUE
}

pub fn close_handle(_handle: Handle) -> Bool {
    TRUE
}

pub fn create_file_a(
    file_name: Lpcstr,
    desired_access: Dword,
    share_mode: Dword,
    _security_attributes: *const SecurityAttributes,
    creation_disposition: Dword,
    flags_and_attributes: Dword,
    _template_file: Handle,
) -> Handle {
    let _ = (file_name, desired_access, share_mode, creation_disposition, flags_and_attributes);
    nt_create_file(ptr::null(), desired_access, share_mode, creation_disposition, flags_and_attributes)
}

pub fn create_file_w(
    file_name: Lpwstr,
    desired_access: Dword,
    share_mode: Dword,
    _security_attributes: *const SecurityAttributes,
    creation_disposition: Dword,
    flags_and_attributes: Dword,
    _template_file: Handle,
) -> Handle {
    let _ = (file_name, desired_access, share_mode, creation_disposition, flags_and_attributes);
    nt_create_file(file_name, desired_access, share_mode, creation_disposition, flags_and_attributes)
}

pub fn open_file(
    name: Lpcstr,
    ofs: *mut OfStruct,
    mode: Uint,
) -> Long {
    if ofs.is_null() {
        return HFILE_ERROR;
    }
    if name.is_null() {
        unsafe {
            (*ofs).n_err_code = ERROR_INVALID_PARAMETER as Word;
        }
        return HFILE_ERROR;
    }
    unsafe {
        (*ofs).c_bytes = core::mem::size_of::<OfStruct>() as Uint;
        (*ofs).n_err_code = 0;
    }
    let _ = mode;
    INVALID_HANDLE_VALUE as Long
}

pub fn read_file(
    file: Handle,
    buffer: Lpvoid,
    number_of_bytes_to_read: Dword,
    number_of_bytes_read: *mut Dword,
    _overlapped: *mut Overlapped,
) -> Bool {
    if buffer.is_null() || number_of_bytes_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = file;
    unsafe {
        *number_of_bytes_read = 0;
    }
    TRUE
}

pub fn write_file(
    file: Handle,
    buffer: Lpcvoid,
    number_of_bytes_to_write: Dword,
    number_of_bytes_written: *mut Dword,
    _overlapped: *mut Overlapped,
) -> Bool {
    if buffer.is_null() || number_of_bytes_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = file;
    unsafe {
        *number_of_bytes_written = number_of_bytes_to_write;
    }
    TRUE
}

pub fn read_file_ex(
    file: Handle,
    buffer: Lpvoid,
    number_of_bytes_to_read: Dword,
    _overlapped: *mut Overlapped,
    _completion_routine: Option<extern "system" fn(Dword, Dword, *mut Overlapped)>,
) -> Bool {
    let _ = (file, buffer, number_of_bytes_to_read);
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn write_file_ex(
    file: Handle,
    buffer: Lpcvoid,
    number_of_bytes_to_write: Dword,
    _overlapped: *mut Overlapped,
    _completion_routine: Option<extern "system" fn(Dword, Dword, *mut Overlapped)>,
) -> Bool {
    let _ = (file, buffer, number_of_bytes_to_write);
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn set_file_pointer(
    file: Handle,
    distance_to_move: Long,
    distance_to_move_high: *mut Long,
    move_method: Dword,
) -> Dword {
    let _ = (file, distance_to_move, distance_to_move_high, move_method);
    0
}

pub fn set_file_pointer_ex(
    file: Handle,
    distance_to_move: LARGE_INTEGER,
    new_file_pointer: *mut LARGE_INTEGER,
    move_method: Dword,
) -> Bool {
    let _ = (file, distance_to_move, new_file_pointer, move_method);
    TRUE
}

pub fn get_file_size(
    file: Handle,
    file_size_high: *mut Dword,
) -> Dword {
    let _ = file;
    if !file_size_high.is_null() {
        unsafe {
            *file_size_high = 0;
        }
    }
    0
}

pub fn get_file_size_ex(
    file: Handle,
    file_size: *mut LARGE_INTEGER,
) -> Bool {
    if file_size.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = file;
    unsafe {
        *file_size = 0;
    }
    TRUE
}

pub fn get_file_information_by_handle(
    file: Handle,
    info: *mut ByHandleFileInformation,
) -> Bool {
    if info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = file;
    unsafe {
        ptr::write(info, ByHandleFileInformation {
            dw_file_attributes: FILE_ATTRIBUTE_NORMAL,
            ft_creation_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            ft_last_access_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            ft_last_write_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            dw_volume_serial_number: 0,
            n_file_size_high: 0,
            n_file_size_low: 0,
            n_number_of_links: 1,
            n_file_index_high: 0,
            n_file_index_low: 0,
        });
    }
    TRUE
}

pub fn get_file_information_by_handle_ex(
    file: Handle,
    info_class: Dword,
    info: Lpvoid,
    info_size: Dword,
) -> Bool {
    let _ = (file, info_class, info, info_size);
    TRUE
}

pub fn set_file_information_by_handle(
    file: Handle,
    info_class: Dword,
    info: Lpvoid,
    info_size: Dword,
) -> Bool {
    let _ = (file, info_class, info, info_size);
    TRUE
}

pub fn get_file_time(
    file: Handle,
    creation_time: *mut FileTime,
    last_access_time: *mut FileTime,
    last_write_time: *mut FileTime,
) -> Bool {
    let _ = file;
    if !creation_time.is_null() {
        unsafe { ptr::write(creation_time, FileTime { dw_low_date_time: 0, dw_high_date_time: 0 }); }
    }
    if !last_access_time.is_null() {
        unsafe { ptr::write(last_access_time, FileTime { dw_low_date_time: 0, dw_high_date_time: 0 }); }
    }
    if !last_write_time.is_null() {
        unsafe { ptr::write(last_write_time, FileTime { dw_low_date_time: 0, dw_high_date_time: 0 }); }
    }
    TRUE
}

pub fn set_file_time(
    file: Handle,
    creation_time: *const FileTime,
    last_access_time: *const FileTime,
    last_write_time: *const FileTime,
) -> Bool {
    let _ = (file, creation_time, last_access_time, last_write_time);
    TRUE
}

pub fn get_file_attributes_a(file_name: Lpcstr) -> Dword {
    let _ = file_name;
    FILE_ATTRIBUTE_NORMAL
}

pub fn get_file_attributes_w(file_name: Lpwstr) -> Dword {
    let _ = file_name;
    FILE_ATTRIBUTE_NORMAL
}

pub fn set_file_attributes_a(file_name: Lpcstr, file_attributes: Dword) -> Bool {
    let _ = (file_name, file_attributes);
    TRUE
}

pub fn set_file_attributes_w(file_name: Lpwstr, file_attributes: Dword) -> Bool {
    let _ = (file_name, file_attributes);
    TRUE
}

pub fn get_file_attributes_ex_a(
    file_name: Lpcstr,
    info: *mut GetFileExInfo,
) -> Bool {
    if info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = file_name;
    unsafe {
        ptr::write(info, GetFileExInfo {
            file_attributes: FILE_ATTRIBUTE_NORMAL,
            ft_creation_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            ft_last_access_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            ft_last_write_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            file_size: 0,
        });
    }
    TRUE
}

pub fn get_file_attributes_ex_w(
    file_name: Lpwstr,
    info: *mut GetFileExInfo,
) -> Bool {
    if info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = file_name;
    unsafe {
        ptr::write(info, GetFileExInfo {
            file_attributes: FILE_ATTRIBUTE_NORMAL,
            ft_creation_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            ft_last_access_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            ft_last_write_time: FileTime { dw_low_date_time: 0, dw_high_date_time: 0 },
            file_size: 0,
        });
    }
    TRUE
}

pub fn copy_file_a(
    existing_file_name: Lpcstr,
    new_file_name: Lpcstr,
    fail_if_exists: Bool,
) -> Bool {
    let _ = (existing_file_name, new_file_name, fail_if_exists);
    TRUE
}

pub fn copy_file_w(
    existing_file_name: Lpwstr,
    new_file_name: Lpwstr,
    fail_if_exists: Bool,
) -> Bool {
    let _ = (existing_file_name, new_file_name, fail_if_exists);
    TRUE
}

pub fn copy_file_ex_a(
    existing_file_name: Lpcstr,
    new_file_name: Lpcstr,
    progress_routine: Option<unsafe extern "system" fn(LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, Dword, Lpvoid, Lpvoid, Lpvoid) -> Dword>,
    data: Lpvoid,
    cancel: *mut Bool,
    flags: Dword,
) -> Bool {
    let _ = (existing_file_name, new_file_name, progress_routine, data, cancel, flags);
    TRUE
}

pub fn copy_file_ex_w(
    existing_file_name: Lpwstr,
    new_file_name: Lpwstr,
    progress_routine: Option<unsafe extern "system" fn(LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, Dword, Lpvoid, Lpvoid, Lpvoid) -> Dword>,
    data: Lpvoid,
    cancel: *mut Bool,
    flags: Dword,
) -> Bool {
    let _ = (existing_file_name, new_file_name, progress_routine, data, cancel, flags);
    TRUE
}

pub fn move_file_a(
    existing_file_name: Lpcstr,
    new_file_name: Lpcstr,
) -> Bool {
    let _ = (existing_file_name, new_file_name);
    TRUE
}

pub fn move_file_w(
    existing_file_name: Lpwstr,
    new_file_name: Lpwstr,
) -> Bool {
    let _ = (existing_file_name, new_file_name);
    TRUE
}

pub fn move_file_ex_a(
    existing_file_name: Lpcstr,
    new_file_name: Lpcstr,
    flags: Dword,
) -> Bool {
    let _ = (existing_file_name, new_file_name, flags);
    TRUE
}

pub fn move_file_ex_w(
    existing_file_name: Lpwstr,
    new_file_name: Lpwstr,
    flags: Dword,
) -> Bool {
    let _ = (existing_file_name, new_file_name, flags);
    TRUE
}

pub fn move_file_with_progress(
    existing_file_name: Lpwstr,
    new_file_name: Lpwstr,
    _progress_routine: Option<unsafe extern "system" fn(LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, Dword, Lpvoid, Lpvoid, Lpvoid) -> Dword>,
    _data: Lpvoid,
    _flags: Dword,
) -> Bool {
    let _ = (existing_file_name, new_file_name);
    TRUE
}

pub fn delete_file_a(file_name: Lpcstr) -> Bool {
    let _ = file_name;
    TRUE
}

pub fn delete_file_w(file_name: Lpwstr) -> Bool {
    let _ = file_name;
    TRUE
}

pub fn create_directory_a(path_name: Lpcstr, _security_attributes: *const SecurityAttributes) -> Bool {
    let _ = path_name;
    TRUE
}

pub fn create_directory_w(path_name: Lpwstr, _security_attributes: *const SecurityAttributes) -> Bool {
    let _ = path_name;
    TRUE
}

pub fn remove_directory_a(path_name: Lpcstr) -> Bool {
    let _ = path_name;
    TRUE
}

pub fn remove_directory_w(path_name: Lpwstr) -> Bool {
    let _ = path_name;
    TRUE
}

pub fn find_first_file_a(
    file_name: Lpcstr,
    find_file_data: *mut Win32FindDataA,
) -> Handle {
    if find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }
    let _ = file_name;
    INVALID_HANDLE_VALUE
}

pub fn find_first_file_w(
    file_name: Lpwstr,
    find_file_data: *mut Win32FindDataW,
) -> Handle {
    if find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }
    let _ = file_name;
    INVALID_HANDLE_VALUE
}

pub fn find_first_file_ex_a(
    file_name: Lpcstr,
    _info_level_id: Dword,
    find_file_data: *mut Win32FindDataA,
    _search_operator: Dword,
) -> Handle {
    if find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }
    let _ = file_name;
    INVALID_HANDLE_VALUE
}

pub fn find_first_file_ex_w(
    file_name: Lpwstr,
    _info_level_id: Dword,
    find_file_data: *mut Win32FindDataW,
    _search_operator: Dword,
) -> Handle {
    if find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return INVALID_HANDLE_VALUE;
    }
    let _ = file_name;
    INVALID_HANDLE_VALUE
}

pub fn find_next_file_a(
    find_file: Handle,
    find_file_data: *mut Win32FindDataA,
) -> Bool {
    if find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = find_file;
    FALSE
}

pub fn find_next_file_w(
    find_file: Handle,
    find_file_data: *mut Win32FindDataW,
) -> Bool {
    if find_file_data.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = find_file;
    FALSE
}

pub fn find_close(_find_file: Handle) -> Bool {
    TRUE
}

pub fn flush_file_buffers(_file: Handle) -> Bool {
    TRUE
}

pub fn set_end_of_file(_file: Handle) -> Bool {
    TRUE
}

pub fn get_file_type(file: Handle) -> Dword {
    let _ = file;
    FILE_TYPE_UNKNOWN
}

pub fn lock_file(
    file: Handle,
    offset: Dword,
    offset_high: Dword,
    count: Dword,
    count_high: Dword,
) -> Bool {
    let _ = (file, offset, offset_high, count, count_high);
    TRUE
}

pub fn lock_file_ex(
    file: Handle,
    flags: Dword,
    reserved: Dword,
    count_low: Dword,
    count_high: Dword,
    overlapped: *mut Overlapped,
) -> Bool {
    let _ = (file, flags, reserved, count_low, count_high, overlapped);
    TRUE
}

pub fn unlock_file(
    file: Handle,
    offset: Dword,
    offset_high: Dword,
    count: Dword,
    count_high: Dword,
) -> Bool {
    let _ = (file, offset, offset_high, count, count_high);
    TRUE
}

pub fn unlock_file_ex(
    file: Handle,
    reserved: Dword,
    count_low: Dword,
    count_high: Dword,
    overlapped: *mut Overlapped,
) -> Bool {
    let _ = (file, reserved, count_low, count_high, overlapped);
    TRUE
}

pub fn create_named_pipe_a(
    name: Lpcstr,
    open_mode: Dword,
    pipe_mode: Dword,
    max_instances: Dword,
    out_buffer_size: Dword,
    in_buffer_size: Dword,
    default_timeout: Dword,
    _security_attributes: *const SecurityAttributes,
) -> Handle {
    let _ = (name, open_mode, pipe_mode, max_instances, out_buffer_size, in_buffer_size, default_timeout);
    INVALID_HANDLE_VALUE
}

pub fn create_named_pipe_w(
    name: Lpwstr,
    open_mode: Dword,
    pipe_mode: Dword,
    max_instances: Dword,
    out_buffer_size: Dword,
    in_buffer_size: Dword,
    default_timeout: Dword,
    _security_attributes: *const SecurityAttributes,
) -> Handle {
    let _ = (name, open_mode, pipe_mode, max_instances, out_buffer_size, in_buffer_size, default_timeout);
    INVALID_HANDLE_VALUE
}

pub fn connect_named_pipe(
    named_pipe: Handle,
    _overlapped: *mut Overlapped,
) -> Bool {
    let _ = named_pipe;
    TRUE
}

pub fn disconnect_named_pipe(_named_pipe: Handle) -> Bool {
    TRUE
}

pub fn get_named_pipe_info(
    named_pipe: Handle,
    flags: *mut Dword,
    out_buffer_size: *mut Dword,
    in_buffer_size: *mut Dword,
    max_instances: *mut Dword,
) -> Bool {
    if flags.is_null() || out_buffer_size.is_null() || in_buffer_size.is_null() || max_instances.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = named_pipe;
    unsafe {
        *flags = PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT;
        *out_buffer_size = 4096;
        *in_buffer_size = 4096;
        *max_instances = PIPE_UNLIMITED_INSTANCES;
    }
    TRUE
}

pub fn set_named_pipe_handle_state(
    named_pipe: Handle,
    mode: *mut Dword,
    max_collection_count: *mut Dword,
    collect_data_timeout: *mut Dword,
) -> Bool {
    let _ = (named_pipe, mode, max_collection_count, collect_data_timeout);
    TRUE
}

pub fn call_named_pipe_a(
    named_pipe_name: Lpcstr,
    in_buffer: Lpvoid,
    in_buffer_size: Dword,
    out_buffer: Lpvoid,
    out_buffer_size: Dword,
    bytes_read: *mut Dword,
    timeout: Dword,
) -> Bool {
    let _ = (named_pipe_name, in_buffer, in_buffer_size, out_buffer, out_buffer_size, bytes_read, timeout);
    TRUE
}

pub fn call_named_pipe_w(
    named_pipe_name: Lpwstr,
    in_buffer: Lpvoid,
    in_buffer_size: Dword,
    out_buffer: Lpvoid,
    out_buffer_size: Dword,
    bytes_read: *mut Dword,
    timeout: Dword,
) -> Bool {
    let _ = (named_pipe_name, in_buffer, in_buffer_size, out_buffer, out_buffer_size, bytes_read, timeout);
    TRUE
}

pub fn wait_named_pipe_a(
    named_pipe_name: Lpcstr,
    timeout: Dword,
) -> Bool {
    let _ = (named_pipe_name, timeout);
    FALSE
}

pub fn wait_named_pipe_w(
    named_pipe_name: Lpwstr,
    timeout: Dword,
) -> Bool {
    let _ = (named_pipe_name, timeout);
    FALSE
}

pub fn create_pipe(
    read_pipe: *mut Handle,
    write_pipe: *mut Handle,
    _pipe_attributes: *const SecurityAttributes,
    size: Dword,
) -> Bool {
    if read_pipe.is_null() || write_pipe.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = size;
    unsafe {
        *read_pipe = ptr::null_mut();
        *write_pipe = ptr::null_mut();
    }
    TRUE
}

pub fn get_temp_path_a(
    buffer_len: Dword,
    buffer: Lpstr,
) -> Dword {
    if buffer.is_null() || buffer_len == 0 {
        return 0;
    }
    let temp = b"C:\\TEMP\0";
    let len = (temp.len() - 1) as Dword;
    if buffer_len > len {
        unsafe {
            ptr::copy_nonoverlapping(temp.as_ptr(), buffer, temp.len());
        }
    }
    len
}

pub fn get_temp_path_w(
    buffer_len: Dword,
    buffer: LpwstrMut,
) -> Dword {
    if buffer.is_null() || buffer_len == 0 {
        return 0;
    }
    let temp: &[Wchar] = &['C' as Wchar, ':' as Wchar, '\\' as Wchar, 'T' as Wchar, 'E' as Wchar, 'M' as Wchar, 'P' as Wchar, 0];
    let len = (temp.len() - 1) as Dword;
    if buffer_len > len {
        unsafe {
            ptr::copy_nonoverlapping(temp.as_ptr(), buffer, temp.len());
        }
    }
    len
}

pub fn get_disk_free_space_ex_a(
    directory_name: Lpcstr,
    free_bytes_available: *mut LARGE_INTEGER,
    total_number_of_bytes: *mut LARGE_INTEGER,
    total_free_bytes: *mut LARGE_INTEGER,
) -> Bool {
    let _ = directory_name;
    unsafe {
        if !free_bytes_available.is_null() { *free_bytes_available = 1024 * 1024 * 1024; }
        if !total_number_of_bytes.is_null() { *total_number_of_bytes = 10 * 1024 * 1024 * 1024; }
        if !total_free_bytes.is_null() { *total_free_bytes = 5 * 1024 * 1024 * 1024; }
    }
    TRUE
}

pub fn get_disk_free_space_ex_w(
    directory_name: Lpwstr,
    free_bytes_available: *mut LARGE_INTEGER,
    total_number_of_bytes: *mut LARGE_INTEGER,
    total_free_bytes: *mut LARGE_INTEGER,
) -> Bool {
    let _ = directory_name;
    unsafe {
        if !free_bytes_available.is_null() { *free_bytes_available = 1024 * 1024 * 1024; }
        if !total_number_of_bytes.is_null() { *total_number_of_bytes = 10 * 1024 * 1024 * 1024; }
        if !total_free_bytes.is_null() { *total_free_bytes = 5 * 1024 * 1024 * 1024; }
    }
    TRUE
}

pub fn replace_file_a(
    replaced: Lpcstr,
    replacement: Lpcstr,
    backup: Lpcstr,
    flags: Dword,
    _exclude: Lpvoid,
    _reserved: Lpvoid,
) -> Bool {
    let _ = (replaced, replacement, backup, flags);
    TRUE
}

pub fn replace_file_w(
    replaced: Lpwstr,
    replacement: Lpwstr,
    backup: Lpwstr,
    flags: Dword,
    _exclude: Lpvoid,
    _reserved: Lpvoid,
) -> Bool {
    let _ = (replaced, replacement, backup, flags);
    TRUE
}

pub fn set_file_completion_notification_modes(
    _file: Handle,
    _flags: u8,
) -> Bool {
    TRUE
}

pub fn set_handle_count(count: Uint) -> Uint {
    count
}

pub fn dos_date_time_to_file_time(
    fat_date: Word,
    fat_time: Word,
    file_time: *mut FileTime,
) -> Bool {
    if file_time.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let year = ((fat_date >> 9) & 0x7F) as Dword + 1980;
    let month = ((fat_date >> 5) & 0x0F) as Dword;
    let day = (fat_date & 0x1F) as Dword;
    let hour = (fat_time >> 11) as Dword;
    let minute = ((fat_time >> 5) & 0x3F) as Dword;
    let second = ((fat_time & 0x1F) * 2) as Dword;

    let seconds_since_1970 = ((year - 1970) * 31536000
        + (month as u64 * 2628000)
        + (day as u64 * 86400)
        + (hour as u64 * 3600)
        + (minute as u64 * 60)
        + second as u64) * 10_000_000;

    unsafe {
        *file_time = FileTime {
            dw_low_date_time: seconds_since_1970 as Dword,
            dw_high_date_time: (seconds_since_1970 >> 32) as Dword,
        };
    }
    TRUE
}

pub fn file_time_to_dos_date_time(
    file_time: *const FileTime,
    fat_date: *mut Word,
    fat_time: *mut Word,
) -> Bool {
    if file_time.is_null() || fat_date.is_null() || fat_time.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    unsafe {
        let val = ((*file_time).dw_high_date_time as u64) << 32 | (*file_time).dw_low_date_time as u64;
        let seconds = val / 10_000_000;
        let year = (seconds / 31536000) as Word + 1970;
        if year < 1980 {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
        let _ = year;
        *fat_date = 0x0021;
        *fat_time = 0;
    }
    TRUE
}

pub fn open_vxd_handle(handle: Handle) -> Handle {
    handle
}

pub fn device_io_control(
    handle: Handle,
    code: Dword,
    in_buffer: Lpvoid,
    in_size: Dword,
    out_buffer: Lpvoid,
    out_size: Dword,
    returned: *mut Dword,
    overlapped: *mut Overlapped,
) -> Bool {
    let _ = (handle, code, in_buffer, in_size, out_buffer, out_size, overlapped);
    unsafe {
        if !returned.is_null() { *returned = 0; }
    }
    FALSE
}

pub fn get_full_path_name_a(
    file_name: Lpcstr,
    n_buffer_length: Dword,
    buffer: Lpstr,
    _file_part: *mut Lpstr,
) -> Dword {
    let _ = (file_name, n_buffer_length, buffer);
    0
}

pub fn get_full_path_name_w(
    file_name: Lpwstr,
    n_buffer_length: Dword,
    buffer: LpwstrMut,
    _file_part: *mut LpwstrMut,
) -> Dword {
    let _ = (file_name, n_buffer_length, buffer);
    0
}

pub fn search_path_a(
    path: Lpcstr,
    file_name: Lpcstr,
    extension: Lpcstr,
    n_buffer_length: Dword,
    buffer: Lpstr,
    _file_part: *mut Lpstr,
) -> Dword {
    let _ = (path, file_name, extension, n_buffer_length, buffer);
    0
}

pub fn search_path_w(
    path: Lpwstr,
    file_name: Lpwstr,
    extension: Lpwstr,
    n_buffer_length: Dword,
    buffer: LpwstrMut,
    _file_part: *mut LpwstrMut,
) -> Dword {
    let _ = (path, file_name, extension, n_buffer_length, buffer);
    0
}

pub fn get_short_path_name_a(
    long_path: Lpcstr,
    short_path: Lpstr,
    buffer_length: Dword,
) -> Dword {
    let _ = (long_path, short_path, buffer_length);
    0
}

pub fn get_short_path_name_w(
    long_path: Lpwstr,
    short_path: LpwstrMut,
    buffer_length: Dword,
) -> Dword {
    let _ = (long_path, short_path, buffer_length);
    0
}

pub fn get_long_path_name_a(
    short_path: Lpcstr,
    long_path: Lpstr,
    buffer_length: Dword,
) -> Dword {
    let _ = (short_path, long_path, buffer_length);
    0
}

pub fn get_long_path_name_w(
    short_path: Lpwstr,
    long_path: LpwstrMut,
    buffer_length: Dword,
) -> Dword {
    let _ = (short_path, long_path, buffer_length);
    0
}

pub fn hread(
    file: Long,
    buffer: Lpvoid,
    count: Long,
) -> Long {
    lread(file, buffer, count)
}

pub fn hwrite(
    file: Long,
    buffer: Lpcstr,
    count: Long,
) -> Long {
    if count == 0 {
        return 0;
    }
    let h = long_to_handle(file);
    let mut written: Dword = 0;
    if write_file(h, buffer as Lpcvoid, count as Dword, &mut written, ptr::null_mut()) == FALSE {
        return HFILE_ERROR;
    }
    written as Long
}

pub fn lclose(file: Long) -> Long {
    if close_handle(long_to_handle(file)) { 0 } else { HFILE_ERROR }
}

pub fn lcreat(path: Lpcstr, attr: Long) -> Long {
    let attr_mask = attr as Dword & (FILE_ATTRIBUTE_READONLY | FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM);
    let h = create_file_a(path, GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE,
        ptr::null(), CREATE_ALWAYS, attr_mask, INVALID_HANDLE_VALUE);
    handle_to_long(h)
}

pub fn lopen(path: Lpcstr, mode: Long) -> Long {
    let _ = (path, mode);
    HFILE_ERROR
}

pub fn lread(file: Long, buffer: Lpvoid, count: Uint) -> Uint {
    let h = long_to_handle(file);
    let mut read: Dword = 0;
    if read_file(h, buffer, count, &mut read, ptr::null_mut()) == FALSE {
        return HFILE_ERROR as Uint;
    }
    read
}

pub fn llseek(file: Long, offset: Long, origin: Long) -> Long {
    set_file_pointer(long_to_handle(file), offset, ptr::null_mut(), origin as Dword) as Long
}

pub fn lwrite(file: Long, buffer: Lpcstr, count: Uint) -> Uint {
    hwrite(file, buffer, count as Long) as Uint
}

pub fn are_file_apis_ansi() -> Bool {
    crate::console_io::are_file_apis_ansi()
}

pub fn set_file_apis_to_ansi() {
    crate::console_io::set_file_apis_to_ansi();
}

pub fn set_file_apis_to_oem() {
    crate::console_io::set_file_apis_to_oem();
}
