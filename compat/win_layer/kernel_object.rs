// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Kernel Object Manager
// File Path            : apps/system/compat/win_layer/kernel_object.rs
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

#[derive(Debug, Clone)]
pub struct KernelEvent { pub handle: u64, pub name: String, pub manual_reset: bool, pub signaled: bool }

#[derive(Debug, Clone)]
pub struct KernelMutex { pub handle: u64, pub name: String, pub owner: u32, pub locked: bool }

#[derive(Debug, Clone)]
pub struct KernelSemaphore { pub handle: u64, pub name: String, pub count: i32, pub max_count: i32 }

pub struct KernelObjectManager {
    pub events: Vec<KernelEvent>, pub mutexes: Vec<KernelMutex>, pub semaphores: Vec<KernelSemaphore>, pub next_handle: u64,
}

impl Default for KernelObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelObjectManager {
    pub fn new() -> Self { Self { events: Vec::new(), mutexes: Vec::new(), semaphores: Vec::new(), next_handle: 0x80000000 } }
    pub fn create_event(name: &str, manual_reset: bool, initial_state: bool, kom: &mut KernelObjectManager) -> u64 {
        let handle = kom.next_handle; kom.next_handle += 1;
        kom.events.push(KernelEvent { handle, name: String::from(name), manual_reset, signaled: initial_state });
        let msg = format!("[KERNEL] CreateEvent(\"{}\", manual={}, signaled={}) -> 0x{:08X}", name, manual_reset, initial_state, handle);
        console_writeln(&msg); handle
    }
    pub fn set_event(handle: u64, kom: &mut KernelObjectManager) -> bool {
        if let Some(e) = kom.events.iter_mut().find(|e| e.handle == handle) { e.signaled = true; true } else { false }
    }
    pub fn reset_event(handle: u64, kom: &mut KernelObjectManager) -> bool {
        if let Some(e) = kom.events.iter_mut().find(|e| e.handle == handle) { e.signaled = false; true } else { false }
    }
    pub fn create_mutex(name: &str, initial_owner: bool, kom: &mut KernelObjectManager) -> u64 {
        let handle = kom.next_handle; kom.next_handle += 1;
        kom.mutexes.push(KernelMutex { handle, name: String::from(name), owner: 0, locked: initial_owner });
        let msg = format!("[KERNEL] CreateMutex(\"{}\", owner={}) -> 0x{:08X}", name, initial_owner, handle);
        console_writeln(&msg); handle
    }
    pub fn release_mutex(handle: u64, kom: &mut KernelObjectManager) -> bool {
        if let Some(m) = kom.mutexes.iter_mut().find(|m| m.handle == handle) { m.locked = false; m.owner = 0; true } else { false }
    }
    pub fn create_semaphore(name: &str, initial_count: i32, max_count: i32, kom: &mut KernelObjectManager) -> u64 {
        let handle = kom.next_handle; kom.next_handle += 1;
        kom.semaphores.push(KernelSemaphore { handle, name: String::from(name), count: initial_count, max_count });
        let msg = format!("[KERNEL] CreateSemaphore(\"{}\", init={}, max={}) -> 0x{:08X}", name, initial_count, max_count, handle);
        console_writeln(&msg); handle
    }
    pub fn release_semaphore(handle: u64, count: i32, kom: &mut KernelObjectManager) -> bool {
        if let Some(s) = kom.semaphores.iter_mut().find(|s| s.handle == handle) { s.count = core::cmp::min(s.count + count, s.max_count); true } else { false }
    }
}

#[derive(Debug, Clone)]
pub struct CriticalSection {
    pub debug_info: u64, pub lock_count: i32, pub recursion_count: i32,
    pub owning_thread: u32, pub lock_semaphore: u64, pub spin_count: u32,
}

impl Default for CriticalSection {
    fn default() -> Self {
        Self::new()
    }
}

impl CriticalSection {
    pub fn new() -> Self { Self { debug_info: 0, lock_count: 0, recursion_count: 0, owning_thread: 0, lock_semaphore: 0, spin_count: 0 } }
    pub fn initialize(&mut self) { self.lock_count = -1; self.recursion_count = 0; self.owning_thread = 0; }
    pub fn enter(&mut self, thread_id: u32) { self.lock_count += 1; if self.owning_thread == 0 { self.owning_thread = thread_id; } self.recursion_count += 1; }
    pub fn leave(&mut self) { self.recursion_count -= 1; if self.recursion_count == 0 { self.owning_thread = 0; self.lock_count = -1; } }
    pub fn try_enter(&mut self, thread_id: u32) -> bool { if self.owning_thread == 0 || self.owning_thread == thread_id { self.enter(thread_id); true } else { false } }
}
