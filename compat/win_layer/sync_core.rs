// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Synchronization Primitives Emulator
// File Path            : apps/system/compat/win_layer/sync_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

use alloc::string::String;
use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;
use alloc::format;

#[derive(Debug, Clone, Copy)]
pub enum SyncObjectType { CriticalSection, Mutex, Semaphore, Event }

#[derive(Debug, Clone)]
pub struct SyncObject {
    pub handle: u64, pub obj_type: SyncObjectType, pub name: String,
    pub locked: bool, pub owner_tid: u32, pub count: u32, pub signaled: bool, pub manual_reset: bool,
}

pub struct SyncManager {
    pub objects: Vec<SyncObject>, pub next_handle: u64,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncManager {
    pub fn new() -> Self { Self { objects: Vec::new(), next_handle: 0x8000 } }
    fn alloc_handle(&mut self) -> u64 { let h = self.next_handle; self.next_handle += 1; h }
    pub fn create_mutex(&mut self, name: &str, initial_owner: bool) -> u64 {
        let h = self.alloc_handle();
        self.objects.push(SyncObject { handle: h, obj_type: SyncObjectType::Mutex, name: String::from(name), locked: initial_owner, owner_tid: 0, count: 1, signaled: false, manual_reset: false });
        let msg = format!("[SYNC] CreateMutex(\"{}\", owner={}) -> 0x{:08X}", name, initial_owner, h);
        console_writeln(&msg); h
    }
    pub fn create_semaphore(&mut self, name: &str, initial: u32, maximum: u32) -> u64 {
        let h = self.alloc_handle();
        self.objects.push(SyncObject { handle: h, obj_type: SyncObjectType::Semaphore, name: String::from(name), locked: false, owner_tid: 0, count: initial, signaled: initial > 0, manual_reset: false });
        let msg = format!("[SYNC] CreateSemaphore(\"{}\", init={}, max={}) -> 0x{:08X}", name, initial, maximum, h);
        console_writeln(&msg); h
    }
    pub fn create_event(&mut self, name: &str, manual_reset: bool, initial_state: bool) -> u64 {
        let h = self.alloc_handle();
        self.objects.push(SyncObject { handle: h, obj_type: SyncObjectType::Event, name: String::from(name), locked: false, owner_tid: 0, count: 0, signaled: initial_state, manual_reset });
        let msg = format!("[SYNC] CreateEvent(\"{}\", manual={}, signaled={}) -> 0x{:08X}", name, manual_reset, initial_state, h);
        console_writeln(&msg); h
    }
    pub fn wait_for_single_object(&mut self, handle: u64, _timeout_ms: u32) -> u32 {
        let msg = format!("[SYNC] WaitForSingleObject(0x{:08X})", handle);
        console_writeln(&msg); 0 // WAIT_OBJECT_0
    }
    pub fn set_event(&mut self, handle: u64) -> bool {
        if let Some(obj) = self.objects.iter_mut().find(|o| o.handle == handle) {
            obj.signaled = true;
            let msg = format!("[SYNC] SetEvent(0x{:08X})", handle);
            console_writeln(&msg); return true;
        }
        false
    }
    pub fn reset_event(&mut self, handle: u64) -> bool {
        if let Some(obj) = self.objects.iter_mut().find(|o| o.handle == handle) {
            obj.signaled = false;
            let msg = format!("[SYNC] ResetEvent(0x{:08X})", handle);
            console_writeln(&msg); return true;
        }
        false
    }
    pub fn release_mutex(&mut self, handle: u64) -> bool {
        let msg = format!("[SYNC] ReleaseMutex(0x{:08X})", handle);
        console_writeln(&msg); true
    }
    pub fn release_semaphore(&mut self, handle: u64, count: u32) -> bool {
        let msg = format!("[SYNC] ReleaseSemaphore(0x{:08X}, count={})", handle, count);
        console_writeln(&msg); true
    }
    pub fn close_handle(&mut self, handle: u64) -> bool {
        let before = self.objects.len();
        self.objects.retain(|o| o.handle != handle);
        let after = self.objects.len();
        let msg = format!("[SYNC] CloseHandle(0x{:08X}) -> {}", handle, before != after);
        console_writeln(&msg); before != after
    }
}

pub struct CriticalSectionEmulator;

impl CriticalSectionEmulator {
    pub fn initialize(_cs: u64) { let msg = format!("{} InitializeCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn delete(_cs: u64) { let msg = format!("{} DeleteCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn enter(_cs: u64) { let msg = format!("{} EnterCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn leave(_cs: u64) { let msg = format!("{} LeaveCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn try_enter(_cs: u64) -> bool { let msg = format!("{} TryEnterCriticalSection() -> true", "[WIN32]"); console_writeln(&msg); true }
}
