// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Registry Hive Ayrıştırıcı
// Dosya Yolu         : apps/system/compat/win32/registry.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64,
//   Alpha, VAX, HPPA, SH-4, IA-64
//
// Açıklama:
//   Windows registry hive dosyasını ayrıştırıcı ve üst seviye registry API.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use crate::win_layer::shared_defs::{RegistryValue, ThreadState, HandleType};

// ─── Registry Hive Parser ──────────────────────────────────────

// ─── Process / Thread Manager ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct WinThread {
    pub tid: u32, pub pid: u32, pub state: ThreadState, pub priority: i32,
    pub entry_point: u64, pub stack_base: u64, pub stack_limit: u64,
    pub context_eax: u32, pub context_ebx: u32, pub context_ecx: u32,
    pub context_edx: u32, pub context_esi: u32, pub context_edi: u32,
    pub context_ebp: u32, pub context_esp: u32, pub context_eip: u32,
    pub context_eflags: u32, pub exit_code: i32,
}

impl WinThread {
    pub fn new(tid: u32, pid: u32, entry: u64) -> Self {
        Self { tid, pid, state: ThreadState::Initialized, priority: 8, entry_point: entry, stack_base: 0, stack_limit: 0, context_eax: 0, context_ebx: 0, context_ecx: 0, context_edx: 0, context_esi: 0, context_edi: 0, context_ebp: 0, context_esp: 0, context_eip: entry as u32, context_eflags: 0x202, exit_code: 0 }
    }
    pub fn set_context(&mut self, eax: u32, ebx: u32, ecx: u32, edx: u32) { self.context_eax = eax; self.context_ebx = ebx; self.context_ecx = ecx; self.context_edx = edx; }
    pub fn suspend(&mut self) { if self.state == ThreadState::Running { self.state = ThreadState::Waiting; } }
    pub fn resume(&mut self) { if self.state == ThreadState::Waiting { self.state = ThreadState::Ready; } }
}

#[derive(Debug, Clone)]
pub struct WinHandle { pub handle: u64, pub handle_type: HandleType, pub value: u64, pub access: u32 }

#[derive(Debug, Clone)]
pub struct WinProcess {
    pub pid: u32, pub parent_pid: u32, pub name: String, pub image_path: String,
    pub cmd_line: String, pub threads: Vec<WinThread>, pub handles: Vec<WinHandle>,
    pub base_address: u64, pub image_size: u64, pub peb_address: u64, pub exit_code: u32,
}

impl WinProcess {
    pub fn new(pid: u32, name: &str) -> Self { Self { pid, parent_pid: 0, name: String::from(name), image_path: String::from(""), cmd_line: String::from(""), threads: Vec::new(), handles: Vec::new(), base_address: 0x400000, image_size: 0, peb_address: 0, exit_code: 0x103 } }
    pub fn create_thread(&mut self, entry: u64) -> u32 { let tid = self.threads.len() as u32 + 1; self.threads.push(WinThread::new(tid, self.pid, entry)); tid }
    pub fn terminate_thread(&mut self, tid: u32) -> bool { if let Some(t) = self.threads.iter_mut().find(|t| t.tid == tid) { t.state = ThreadState::Terminated; true } else { false } }
    pub fn allocate_handle(&mut self, handle_type: HandleType, value: u64) -> u64 { let handle = 0x1000 + self.handles.len() as u64; self.handles.push(WinHandle { handle, handle_type, value, access: 0x1F0FFF }); handle }
    pub fn close_handle(&mut self, handle: u64) -> bool { if let Some(idx) = self.handles.iter().position(|h| h.handle == handle) { self.handles.remove(idx); true } else { false } }
}


// ─── High-Level Registry ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RegistryKey {
    pub name: String, pub values: BTreeMap<String, RegistryValue>, pub subkeys: Vec<String>,
}

impl RegistryKey {
    pub fn new(name: &str) -> Self { Self { name: String::from(name), values: BTreeMap::new(), subkeys: Vec::new() } }
    pub fn set_value(&mut self, value_name: &str, value: RegistryValue) { self.values.insert(String::from(value_name), value); }
    pub fn get_value(&self, value_name: &str) -> Option<&RegistryValue> { self.values.get(value_name) }
    pub fn add_subkey(&mut self, name: &str) { self.subkeys.push(String::from(name)); }
}

