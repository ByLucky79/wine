// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Win32 Common Types and Enums
// File Path            : apps/system/compat/win_layer/types.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Shared enums and lightweight structs for the Win32 compatibility
//   layer. Extracted from the legacy monolithic module during
//   modularisation. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

// ─── NT Status ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NtStatus {
    Success = 0x0000_0000,
    Pending = 0x0000_0103,
    NotImplemented = 0xC000_0002,
    InvalidParameter = 0xC000_000D,
    AccessDenied = 0xC000_0022,
    ObjectNameNotFound = 0xC000_0034,
    ObjectNameCollision = 0xC000_0035,
    ObjectPathNotFound = 0xC000_003A,
    NoMemory = 0xC000_0017,
    InvalidHandle = 0xC000_0008,
    EndOfFile = 0xC000_0011,
}

// ─── Thread State ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Initialized, Ready, Running, Waiting, Terminated,
}

// ─── Handle Type ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleType {
    Process, Thread, File, RegistryKey, Event, Mutex, Section, Unknown,
}

// ─── Window Messages ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinMsg {
    Null, Create, Destroy, Move, Size, Activate, SetFocus, KillFocus,
    Enable, SetRedraw, SetText, GetText, Paint, Close, QueryEndSession,
    Quit, EraseBackground, SysColorChange, ShowWindow, KeyDown, KeyUp,
    Char, MouseMove, LButtonDown, LButtonUp, RButtonDown, RButtonUp,
    Command, Notify, Timer, User(u32),
}

impl WinMsg {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0x0000 => Self::Null, 0x0001 => Self::Create, 0x0002 => Self::Destroy,
            0x0003 => Self::Move, 0x0005 => Self::Size, 0x0006 => Self::Activate,
            0x0007 => Self::SetFocus, 0x0008 => Self::KillFocus, 0x000A => Self::Enable,
            0x000B => Self::SetRedraw, 0x000C => Self::SetText, 0x000D => Self::GetText,
            0x000F => Self::Paint, 0x0010 => Self::Close, 0x0011 => Self::QueryEndSession,
            0x0012 => Self::Quit, 0x0014 => Self::EraseBackground, 0x0015 => Self::SysColorChange,
            0x0018 => Self::ShowWindow, 0x0100 => Self::KeyDown, 0x0101 => Self::KeyUp,
            0x0102 => Self::Char, 0x0200 => Self::MouseMove, 0x0201 => Self::LButtonDown,
            0x0202 => Self::LButtonUp, 0x0204 => Self::RButtonDown, 0x0205 => Self::RButtonUp,
            0x0111 => Self::Command, 0x004E => Self::Notify, 0x0113 => Self::Timer,
            _ => Self::User(v),
        }
    }
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Null => 0x0000, Self::Create => 0x0001, Self::Destroy => 0x0002, Self::Move => 0x0003,
            Self::Size => 0x0005, Self::Activate => 0x0006, Self::SetFocus => 0x0007, Self::KillFocus => 0x0008,
            Self::Enable => 0x000A, Self::SetRedraw => 0x000B, Self::SetText => 0x000C, Self::GetText => 0x000D,
            Self::Paint => 0x000F, Self::Close => 0x0010, Self::QueryEndSession => 0x0011, Self::Quit => 0x0012,
            Self::EraseBackground => 0x0014, Self::SysColorChange => 0x0015, Self::ShowWindow => 0x0018,
            Self::KeyDown => 0x0100, Self::KeyUp => 0x0101, Self::Char => 0x0102, Self::MouseMove => 0x0200,
            Self::LButtonDown => 0x0201, Self::LButtonUp => 0x0202, Self::RButtonDown => 0x0204,
            Self::RButtonUp => 0x0205, Self::Command => 0x0111, Self::Notify => 0x004E, Self::Timer => 0x0113,
            Self::User(v) => *v,
        }
    }
}

// ─── Service Status ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Stopped, StartPending, StopPending, Running,
    ContinuePending, PausePending, Paused,
}

// ─── GDI Object Type ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GdiObjectType {
    Pen, Brush, Bitmap, Font, Region, Palette, Dc,
}

// ─── Exception Disposition ───────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionDisposition {
    ExceptionContinueExecution,
    ExceptionContinueSearch,
    ExceptionNestedException,
    ExceptionCollidedUnwind,
}

// ─── Pipe Type ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeType {
    Anonymous, Named,
}

// ─── Registry Value ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum RegistryValue {
    String(String), Dword(u32), Qword(u64), Binary(Vec<u8>), MultiString(Vec<String>),
}

// ─── System Info ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub processor_architecture: u16,
    pub page_size: u32,
    pub minimum_application_address: u64,
    pub maximum_application_address: u64,
    pub active_processor_mask: u64,
    pub number_of_processors: u32,
    pub processor_type: u32,
    pub allocation_granularity: u32,
    pub processor_level: u16,
    pub processor_revision: u16,
}

// ─── OS Version Info ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OsVersionInfo {
    pub major_version: u32,
    pub minor_version: u32,
    pub build_number: u32,
    pub platform_id: u32,
    pub csd_version: String,
}

// ─── Win32 Section ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Win32Section {
    pub handle: u64,
    pub size: u64,
    pub base: u64,
}

// ─── PEB / TEB ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ProcessEnvironmentBlock {
    pub image_base_address: u64,
    pub ldr: u64,
    pub process_parameters: u64,
    pub subsystem: u32,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub being_debugged: bool,
    pub process_heap: u64,
    pub tls_bitmap: u64,
}

impl ProcessEnvironmentBlock {
    pub fn new(image_base: u64) -> Self {
        Self {
            image_base_address: image_base,
            ldr: 0,
            process_parameters: 0,
            subsystem: 1,
            major_subsystem_version: 1,
            minor_subsystem_version: 0,
            being_debugged: false,
            process_heap: 0,
            tls_bitmap: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThreadEnvironmentBlock {
    pub self_ptr: u64,
    pub peb: u64,
    pub stack_base: u64,
    pub stack_limit: u64,
    pub sub_system_tib: u64,
    pub fiber_data: u64,
    pub arbitrary_user_pointer: u64,
    pub thread_id: u32,
    pub process_id: u32,
}

impl ThreadEnvironmentBlock {
    pub fn new(peb: u64, tid: u32, pid: u32) -> Self {
        Self {
            self_ptr: 0,
            peb,
            stack_base: 0,
            stack_limit: 0,
            sub_system_tib: 0,
            fiber_data: 0,
            arbitrary_user_pointer: 0,
            thread_id: tid,
            process_id: pid,
        }
    }
}


// ─── Color Theme Constants ───────────────────────────────────

pub const WIN_W: i32 = 580;
pub const WIN_H: i32 = 420;
pub const CLR_WIN_BG: u32 = 0xFF2D2D30;
pub const CLR_SIDEBAR: u32 = 0xFF3F3F46;
pub const CLR_ITEM_BG: u32 = 0xFF1E1E1E;
pub const CLR_ACCENT: u32 = 0xFF0078D4;
pub const CLR_TEXT: u32 = 0xFFF1F1F1;
pub const CLR_SUCCESS: u32 = 0xFF107C10;
pub const CLR_WARNING: u32 = 0xFFD83B01;

// ─── Helper Functions ────────────────────────────────────────

pub fn security_level_label(level: i32) -> String {
    match level {
        0 => String::from("[WIN32]"),
        1 => String::from("[WIN32]"),
        _ => String::from("[WIN32]"),
    }
}

pub fn split_find_pattern(pattern: &str) -> (String, String) {
    let p = pattern.to_ascii_uppercase();
    if let Some(idx) = p.rfind('\\') {
        (String::from(&p[..idx]), String::from(&p[idx + 1..]))
    } else {
        (String::new(), p)
    }
}

pub fn wildcard_match(name: &str, pattern: &str) -> bool {
    let mut name_chars = name.chars().peekable();
    let mut pat_chars = pattern.chars().peekable();
    while let Some(pc) = pat_chars.next() {
        match pc {
            '*' => {
                let next = pat_chars.peek().copied();
                if next.is_none() { return true; }
                for nc in name_chars.by_ref() {
                    if Some(nc) == next {
                        break;
                    }
                }
            }
            '?' => { name_chars.next(); }
            c => {
                if name_chars.next() != Some(c) { return false; }
            }
        }
    }
    name_chars.next().is_none()
}
