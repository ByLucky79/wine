// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Islem Yoneticileri - PEB, TEB, FileSearch, SCM, ThreadScheduler
// Dosya Yolu         : apps/system/compat/win32/process_mgr_extra.rs
// Yazar              : Ozkan Yildirim
// Lisans             : GPLv3
//
// Destekledigi Islemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64,
//   Alpha, VAX, HPPA, SH-4, IA-64
//
// Aciklama:
//   Win32 Islem Yoneticileri - PEB, TEB, FileSearch, SCM, ThreadScheduler
//
// Bagimli Dosyalar:
//   1-) apps/system/compat/win32/win32_mod.rs
//
//              Dosyaya Mudahaleler
// 2026-05-13      syscall_emu.rs bolundu
// *******************************************************************

extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::VecDeque;
use crate::win_layer::shared_defs::ThreadState;
use crate::dos_emulator::console_writeln;
use crate::win32::registry::WinProcess;

pub struct PebEmulator {
    pub image_base: u64, pub ldr: u64, pub process_parameters: u64,
    pub being_debugged: bool, pub os_major: u32, pub os_minor: u32, pub os_build: u32,
    pub session_id: u32, pub number_of_processors: u32, pub processor_arch: u16,
    pub heap: u64, pub tls_bitmap: u64, pub tls_count: u32,
}

impl PebEmulator {
    pub fn new(image_base: u64) -> Self {
        Self { image_base, ldr: 0x7FFF_0000_0000, process_parameters: 0x7FFF_0000_1000, being_debugged: false, os_major: 10, os_minor: 0, os_build: 19045, session_id: 1, number_of_processors: 8, processor_arch: 9, heap: 0x1000_0000, tls_bitmap: 0, tls_count: 0 }
    }
    pub fn get_peb_base(&self) -> u64 { 0x7FFF_0000_0000 }
    pub fn read_field_u8(&self, offset: usize) -> u8 {
        match offset {
            2 => self.being_debugged as u8,
            _ => 0,
        }
    }
    pub fn read_field_u32(&self, offset: usize) -> u32 {
        match offset {
            4 => self.os_major, 8 => self.os_minor, 12 => self.os_build,
            44 => self.session_id, 64 => self.number_of_processors,
            _ => 0,
        }
    }
    pub fn read_field_u64(&self, offset: usize) -> u64 {
        match offset {
            16 => self.image_base, 24 => self.ldr, 32 => self.process_parameters, 48 => self.heap, _ => 0,
        }
    }
    pub fn dump(&self) {
        let msg = format!("[PEB] base=0x{:016X} image_base=0x{:016X} os={}.{}.{}", self.get_peb_base(), self.image_base, self.os_major, self.os_minor, self.os_build);
        console_writeln(&msg);
    }
}

// ─── Thread Environment Block (TEB) Emulator ───────────────────

#[derive(Debug, Clone)]
pub struct TebEmulator {
    pub teb_base: u64, pub stack_base: u64, pub stack_limit: u64,
    pub peb: u64, pub tid: u32, pub pid: u32, pub last_error: u32,
    pub tls_slots: [u64; 64], pub locale: u32, pub win32_thread_info: u64,
}

impl TebEmulator {
    pub fn new(tid: u32, pid: u32, peb: u64) -> Self {
        Self { teb_base: 0x7FFF_0001_0000 + tid as u64 * 0x1000, stack_base: 0x7FFF_0002_0000, stack_limit: 0x7FFF_0001_0000, peb, tid, pid, last_error: 0, tls_slots: [0; 64], locale: 0x0409, win32_thread_info: 0 }
    }
    pub fn get_tls_slot(&self, index: u32) -> u64 { if index < 64 { self.tls_slots[index as usize] } else { 0 } }
    pub fn set_tls_slot(&mut self, index: u32, value: u64) { if index < 64 { self.tls_slots[index as usize] = value; } }
    pub fn set_last_error(&mut self, err: u32) { self.last_error = err; }
    pub fn get_last_error(&self) -> u32 { self.last_error }
    pub fn dump(&self) {
        let msg = format!("{} base=0x{:016X} tid={} pid={} last_err=0x{:08X}", "[WIN32]", self.teb_base, self.tid, self.pid, self.last_error);
        console_writeln(&msg);
    }
}

// ─── File Search Emulator (FindFirstFile/FindNextFile) ─────────

#[derive(Debug, Clone)]
pub struct FindFileEntry {
    pub name: String, pub attrs: u32, pub size: u64, pub creation: u64, pub access: u64, pub write: u64,
}

pub struct FileSearchHandle {
    pub handle: u64, pub pattern: String, pub entries: Vec<FindFileEntry>, pub index: usize,
}

pub struct FileSearchManager {
    pub handles: Vec<FileSearchHandle>, pub next_handle: u64,
}

impl Default for FileSearchManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSearchManager {
    pub fn new() -> Self { Self { handles: Vec::new(), next_handle: 0xB000 } }
    pub fn find_first_file(&mut self, pattern: &str) -> u64 {
        let h = self.next_handle; self.next_handle += 1;
        let mut entries = Vec::new();
        entries.push(FindFileEntry { name: String::from("."), attrs: 0x10, size: 0, creation: 0, access: 0, write: 0 });
        entries.push(FindFileEntry { name: String::from(".."), attrs: 0x10, size: 0, creation: 0, access: 0, write: 0 });
        if pattern.contains("*") || pattern.contains("?") {
            entries.push(FindFileEntry { name: String::from("file1.txt"), attrs: 0x20, size: 1024, creation: 0, access: 0, write: 0 });
            entries.push(FindFileEntry { name: String::from("file2.dll"), attrs: 0x20, size: 2048, creation: 0, access: 0, write: 0 });
            entries.push(FindFileEntry { name: String::from("file3.exe"), attrs: 0x20, size: 4096, creation: 0, access: 0, write: 0 });
        }
        self.handles.push(FileSearchHandle { handle: h, pattern: String::from(pattern), entries, index: 0 });
        let msg = format!("{} FindFirstFile(\"{}\") -> 0x{:08X}", "[WIN32]", pattern, h);
        console_writeln(&msg); h
    }
    pub fn find_next_file(&mut self, handle: u64, entry: &mut FindFileEntry) -> bool {
        if let Some(h) = self.handles.iter_mut().find(|h| h.handle == handle) {
            if h.index < h.entries.len() {
                *entry = h.entries[h.index].clone(); h.index += 1;
                let msg = format!("{} FindNextFile(0x{:08X}) -> \"{}\"", "[WIN32]", handle, entry.name);
                console_writeln(&msg); return true;
            }
        }
        let msg = format!("{} FindNextFile() -> false", "[WIN32]");
        console_writeln(&msg); false
    }
    pub fn find_close(&mut self, handle: u64) -> bool {
        let before = self.handles.len();
        self.handles.retain(|h| h.handle != handle);
        let after = self.handles.len();
        let msg = format!("{} FindClose(0x{:08X}) -> {}", "[WIN32]", handle, before != after);
        console_writeln(&msg); before != after
    }
}

// ─── Service Control Manager (SCM) ─────────────────────────────

// ─── Window Subclassing Emulator ───────────────────────────────

// ─── Thread Scheduler ────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ThreadScheduler {
    pub ready_queue: VecDeque<u32>,
    pub current_tid: Option<u32>,
    pub time_slice: u32,
}

impl Default for ThreadScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ThreadScheduler {
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new(), current_tid: None, time_slice: 10 }
    }
    pub fn add_thread(&mut self, tid: u32) {
        self.ready_queue.push_back(tid);
    }
    pub fn remove_thread(&mut self, tid: u32) {
        self.ready_queue.retain(|&t| t != tid);
        if self.current_tid == Some(tid) { self.current_tid = None; }
    }
    pub fn tick(&mut self, processes: &mut [WinProcess]) -> Option<u32> {
        if let Some(current) = self.current_tid {
            if let Some(proc) = processes.iter_mut().find(|p| p.threads.iter().any(|t| t.tid == current)) {
                if let Some(thread) = proc.threads.iter_mut().find(|t| t.tid == current) {
                    if thread.state != ThreadState::Terminated {
                        thread.state = ThreadState::Ready;
                        self.ready_queue.push_back(current);
                    }
                }
            }
        }
        self.schedule(processes)
    }
    pub fn schedule(&mut self, processes: &mut [WinProcess]) -> Option<u32> {
        while let Some(tid) = self.ready_queue.pop_front() {
            if let Some(proc) = processes.iter_mut().find(|p| p.threads.iter().any(|t| t.tid == tid)) {
                if let Some(thread) = proc.threads.iter_mut().find(|t| t.tid == tid) {
                    if matches!(thread.state, ThreadState::Ready | ThreadState::Initialized) {
                        thread.state = ThreadState::Running;
                        self.current_tid = Some(tid);
                        return Some(tid);
                    }
                }
            }
        }
        self.current_tid = None;
        None
    }
    pub fn yield_current(&mut self, processes: &mut [WinProcess]) -> Option<u32> {
        if let Some(current) = self.current_tid {
            if let Some(proc) = processes.iter_mut().find(|p| p.threads.iter().any(|t| t.tid == current)) {
                if let Some(thread) = proc.threads.iter_mut().find(|t| t.tid == current) {
                    if thread.state != ThreadState::Terminated {
                        thread.state = ThreadState::Ready;
                        self.ready_queue.push_back(current);
                    }
                }
            }
        }
        self.schedule(processes)
    }
}

// ─── Unit Tests ─────────────────────────────────────────────



