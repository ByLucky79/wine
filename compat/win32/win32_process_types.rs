// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 süreç ve thread temel tipleri.
// Dosya Yolu         : compat/win32/win32_process_types.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Bu dosya Win32 uyumluluk katmanında kullanılan süreç, thread,
//   başlangıç bilgisi ve temel zamanlama durumlarını ortak yapılarda toplar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/win32_handle_table.rs
//   3-) compat/win32/win32_virtual_memory.rs
//
//              Dosyaya Müdahaleler
// 2026-05-16      Ortak Win32 süreç tipleri dosyası oluşturuldu
// *******************************************************************

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::win32::win32_base_types::{WinDword, WinHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinThreadState {
    Initialized,
    Ready,
    Running,
    Waiting,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone)]
pub struct WinStartupInfo {
    pub desktop_name: String,
    pub title: String,
    pub x: WinDword,
    pub y: WinDword,
    pub width: WinDword,
    pub height: WinDword,
    pub flags: WinDword,
    pub show_window: WinDword,
}

impl WinStartupInfo {
    pub fn new() -> Self {
        Self {
            desktop_name: String::new(),
            title: String::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            flags: 0,
            show_window: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WinThreadRecord {
    pub tid: WinDword,
    pub owner_pid: WinDword,
    pub state: WinThreadState,
    pub entry_point: u64,
    pub stack_base: u64,
    pub stack_limit: u64,
    pub teb_address: u64,
    pub suspend_count: WinDword,
    pub exit_code: WinDword,
    pub handle: WinHandle,
}

impl WinThreadRecord {
    pub fn new(tid: WinDword, owner_pid: WinDword, entry_point: u64, handle: WinHandle) -> Self {
        Self {
            tid,
            owner_pid,
            state: WinThreadState::Initialized,
            entry_point,
            stack_base: 0,
            stack_limit: 0,
            teb_address: 0,
            suspend_count: 0,
            exit_code: 0,
            handle,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WinProcessRecord {
    pub pid: WinDword,
    pub parent_pid: WinDword,
    pub image_path: String,
    pub command_line: String,
    pub current_directory: String,
    pub image_base: u64,
    pub image_size: u64,
    pub peb_address: u64,
    pub process_handle: WinHandle,
    pub primary_thread_handle: WinHandle,
    pub exit_code: WinDword,
    pub threads: Vec<WinThreadRecord>,
    pub startup_info: WinStartupInfo,
}

impl WinProcessRecord {
    pub fn new(pid: WinDword, parent_pid: WinDword, image_path: &str, process_handle: WinHandle) -> Self {
        Self {
            pid,
            parent_pid,
            image_path: String::from(image_path),
            command_line: String::from(image_path),
            current_directory: String::new(),
            image_base: 0,
            image_size: 0,
            peb_address: 0,
            process_handle,
            primary_thread_handle: 0,
            exit_code: 0,
            threads: Vec::new(),
            startup_info: WinStartupInfo::new(),
        }
    }

    pub fn add_thread(&mut self, thread: WinThreadRecord) {
        if self.primary_thread_handle == 0 {
            self.primary_thread_handle = thread.handle;
        }
        self.threads.push(thread);
    }

    pub fn thread_count(&self) -> usize {
        self.threads.len()
    }
}
