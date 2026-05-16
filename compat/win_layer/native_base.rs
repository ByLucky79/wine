// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Windows NT Core Types
// File Path            : apps/system/compat/win_layer/winnt.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Fundamental Windows NT type definitions and constants for
//   the ÖZKAN-OS Win32 compatibility layer. Structures use
//   #[repr(C)] for ABI compatibility with PE images.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals)]

// ─── Scalar Type Aliases ─────────────────────────────────────

pub type VOID = ();
pub type PVOID = *mut VOID;
pub type PVOID64 = *mut VOID;

pub type CHAR = i8;
pub type SHORT = i16;
pub type LONG = i32;

pub type UCHAR = u8;
pub type USHORT = u16;
pub type ULONG = u32;

pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = u32;

pub type BOOL = i32;
pub type BOOLEAN = u8;
pub type INT = i32;
pub type UINT = u32;

pub type LONG64 = i64;
pub type ULONG64 = u64;
pub type DWORD64 = u64;
pub type LONGLONG = i64;
pub type ULONGLONG = u64;
pub type DWORDLONG = u64;

pub type SIZE_T = usize;
pub type SSIZE_T = isize;
pub type ULONG_PTR = usize;

pub type HANDLE = PVOID;
pub type PHANDLE = *mut HANDLE;

pub type LCID = DWORD;
pub type LANGID = WORD;
pub type EXECUTION_STATE = DWORD;

// ─── Large Integer ───────────────────────────────────────────

#[derive(Clone, Copy)]
#[repr(C)]
pub union LARGE_INTEGER {
    pub s: LARGE_INTEGER_s,
    pub u: ULARGE_INTEGER_u,
    pub QuadPart: LONGLONG,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LARGE_INTEGER_s {
    pub LowPart: DWORD,
    pub HighPart: LONG,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union ULARGE_INTEGER {
    pub u: ULARGE_INTEGER_u,
    pub QuadPart: ULONGLONG,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ULARGE_INTEGER_u {
    pub LowPart: DWORD,
    pub HighPart: DWORD,
}

// ─── Strings ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: *mut u16,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ANSI_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: *mut u8,
}

// ─── SID ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SID_IDENTIFIER_AUTHORITY {
    pub Value: [BYTE; 6],
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SID {
    pub Revision: BYTE,
    pub SubAuthorityCount: BYTE,
    pub IdentifierAuthority: SID_IDENTIFIER_AUTHORITY,
    pub SubAuthority: [ULONG; 1],
}

// ─── ACL / ACE ───────────────────────────────────────────────

pub const ACL_REVISION: DWORD = 2;
pub const ACL_REVISION_DS: DWORD = 4;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ACL {
    pub AclRevision: BYTE,
    pub Sbz1: BYTE,
    pub AclSize: WORD,
    pub AceCount: WORD,
    pub Sbz2: WORD,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ACE_HEADER {
    pub AceType: BYTE,
    pub AceFlags: BYTE,
    pub AceSize: WORD,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ACCESS_ALLOWED_ACE {
    pub Header: ACE_HEADER,
    pub Mask: DWORD,
    pub SidStart: DWORD,
}

// ─── Security Descriptor ─────────────────────────────────────

pub type SECURITY_DESCRIPTOR_CONTROL = WORD;
pub const SE_SELF_RELATIVE: SECURITY_DESCRIPTOR_CONTROL = 0x8000;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SECURITY_DESCRIPTOR {
    pub Revision: BYTE,
    pub Sbz1: BYTE,
    pub Control: SECURITY_DESCRIPTOR_CONTROL,
    pub Owner: *mut SID,
    pub Group: *mut SID,
    pub Sacl: *mut ACL,
    pub Dacl: *mut ACL,
}

// ─── Object Attributes ───────────────────────────────────────

pub const OBJ_INHERIT: ULONG = 0x00000002;
pub const OBJ_PERMANENT: ULONG = 0x00000010;
pub const OBJ_EXCLUSIVE: ULONG = 0x00000020;
pub const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
pub const OBJ_OPENIF: ULONG = 0x00000080;
pub const OBJ_OPENLINK: ULONG = 0x00000100;
pub const OBJ_KERNEL_HANDLE: ULONG = 0x00000200;
pub const OBJ_FORCE_ACCESS_CHECK: ULONG = 0x00000400;
pub const OBJ_IGNORE_IMPERSONATED_DEVICEMAP: ULONG = 0x00000800;
pub const OBJ_DONT_REPARSE: ULONG = 0x00001000;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct OBJECT_ATTRIBUTES {
    pub Length: ULONG,
    pub RootDirectory: HANDLE,
    pub ObjectName: *mut UNICODE_STRING,
    pub Attributes: ULONG,
    pub SecurityDescriptor: *mut SECURITY_DESCRIPTOR,
    pub SecurityQualityOfService: PVOID,
}

impl OBJECT_ATTRIBUTES {
    pub fn new(name: *mut UNICODE_STRING, attributes: ULONG) -> Self {
        Self {
            Length: core::mem::size_of::<Self>() as ULONG,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: name,
            Attributes: attributes,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        }
    }
}

// ─── IO Status Block ─────────────────────────────────────────

#[derive(Clone, Copy)]
#[repr(C)]
pub union IO_STATUS_BLOCK_u {
    pub Status: LONG,
    pub Pointer: PVOID,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IO_STATUS_BLOCK {
    pub u: IO_STATUS_BLOCK_u,
    pub Information: ULONG_PTR,
}

// ─── NT Status Codes ─────────────────────────────────────────

pub const STATUS_SUCCESS: LONG = 0x00000000;
pub const STATUS_PENDING: LONG = 0x00000103;
pub const STATUS_NOT_IMPLEMENTED: LONG = 0xC0000002u32 as i32;
pub const STATUS_INVALID_PARAMETER: LONG = 0xC000000Du32 as i32;
pub const STATUS_ACCESS_DENIED: LONG = 0xC0000022u32 as i32;
pub const STATUS_OBJECT_NAME_NOT_FOUND: LONG = 0xC0000034u32 as i32;
pub const STATUS_OBJECT_NAME_COLLISION: LONG = 0xC0000035u32 as i32;
pub const STATUS_NO_MEMORY: LONG = 0xC0000017u32 as i32;
pub const STATUS_INVALID_HANDLE: LONG = 0xC0000008u32 as i32;
pub const STATUS_END_OF_FILE: LONG = 0xC0000011u32 as i32;

// ─── File Access Rights ──────────────────────────────────────

pub const FILE_READ_DATA: DWORD = 0x00000001;
pub const FILE_WRITE_DATA: DWORD = 0x00000002;
pub const FILE_APPEND_DATA: DWORD = 0x00000004;
pub const FILE_READ_EA: DWORD = 0x00000008;
pub const FILE_WRITE_EA: DWORD = 0x00000010;
pub const FILE_EXECUTE: DWORD = 0x00000020;
pub const FILE_DELETE_CHILD: DWORD = 0x00000040;
pub const FILE_READ_ATTRIBUTES: DWORD = 0x00000080;
pub const FILE_WRITE_ATTRIBUTES: DWORD = 0x00000100;

pub const DELETE: DWORD = 0x00010000;
pub const READ_CONTROL: DWORD = 0x00020000;
pub const WRITE_DAC: DWORD = 0x00040000;
pub const WRITE_OWNER: DWORD = 0x00080000;
pub const SYNCHRONIZE: DWORD = 0x00100000;

pub const FILE_GENERIC_READ: DWORD = 0x80000000 | READ_CONTROL | SYNCHRONIZE | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA;
pub const FILE_GENERIC_WRITE: DWORD = 0x40000000 | READ_CONTROL | SYNCHRONIZE | FILE_WRITE_DATA | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA;
pub const FILE_GENERIC_EXECUTE: DWORD = 0x20000000 | READ_CONTROL | SYNCHRONIZE | FILE_READ_ATTRIBUTES | FILE_EXECUTE;

// ─── Memory Protection ───────────────────────────────────────

pub const PAGE_NOACCESS: DWORD = 0x01;
pub const PAGE_READONLY: DWORD = 0x02;
pub const PAGE_READWRITE: DWORD = 0x04;
pub const PAGE_WRITECOPY: DWORD = 0x08;
pub const PAGE_EXECUTE: DWORD = 0x10;
pub const PAGE_EXECUTE_READ: DWORD = 0x20;
pub const PAGE_EXECUTE_READWRITE: DWORD = 0x40;
pub const PAGE_EXECUTE_WRITECOPY: DWORD = 0x80;
pub const PAGE_GUARD: DWORD = 0x100;
pub const PAGE_NOCACHE: DWORD = 0x200;
pub const PAGE_WRITECOMBINE: DWORD = 0x400;
