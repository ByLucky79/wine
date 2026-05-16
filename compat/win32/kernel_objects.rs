// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Kernel nesneleri ve kritik bölüm altyapısı.
// Dosya Yolu         : compat/win32/kernel_objects.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Event, Mutex, Semaphore kernel nesneleri ile kritik bölüm yapısını
//   ortak handle tablosu ile uyumlu biçimde sağlar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/win32_handle_table.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// 2026-05-16      Kernel nesneleri ortak handle tablosu ile hizalandı
// *******************************************************************

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::win32::win32_base_types::{WinHandle, WinHandleKind};
use crate::win32::win32_handle_table::WinHandleTable;

#[derive(Debug, Clone)]
pub struct KernelEvent {
    pub handle: WinHandle,
    pub name: String,
    pub manual_reset: bool,
    pub signaled: bool,
}

#[derive(Debug, Clone)]
pub struct KernelMutex {
    pub handle: WinHandle,
    pub name: String,
    pub owner_thread_id: u32,
    pub locked: bool,
}

#[derive(Debug, Clone)]
pub struct KernelSemaphore {
    pub handle: WinHandle,
    pub name: String,
    pub current_count: i32,
    pub maximum_count: i32,
}

#[derive(Debug, Default)]
pub struct KernelObjectManager {
    pub events: Vec<KernelEvent>,
    pub mutexes: Vec<KernelMutex>,
    pub semaphores: Vec<KernelSemaphore>,
    pub handles: WinHandleTable,
}

impl KernelObjectManager {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            mutexes: Vec::new(),
            semaphores: Vec::new(),
            handles: WinHandleTable::new(),
        }
    }

    pub fn create_event(&mut self, name: &str, manual_reset: bool, initial_state: bool) -> WinHandle {
        let handle = self
            .handles
            .allocate(WinHandleKind::Event, 0, self.events.len() as u64, name, false);
        self.events.push(KernelEvent {
            handle,
            name: String::from(name),
            manual_reset,
            signaled: initial_state,
        });
        handle
    }

    pub fn set_event(&mut self, handle: WinHandle) -> bool {
        if let Some(event) = self.events.iter_mut().find(|event| event.handle == handle) {
            event.signaled = true;
            true
        } else {
            false
        }
    }

    pub fn reset_event(&mut self, handle: WinHandle) -> bool {
        if let Some(event) = self.events.iter_mut().find(|event| event.handle == handle) {
            event.signaled = false;
            true
        } else {
            false
        }
    }

    pub fn create_mutex(&mut self, name: &str, initial_owner: bool) -> WinHandle {
        let handle = self
            .handles
            .allocate(WinHandleKind::Mutex, 0, self.mutexes.len() as u64, name, false);
        self.mutexes.push(KernelMutex {
            handle,
            name: String::from(name),
            owner_thread_id: 0,
            locked: initial_owner,
        });
        handle
    }

    pub fn release_mutex(&mut self, handle: WinHandle) -> bool {
        if let Some(mutex) = self.mutexes.iter_mut().find(|mutex| mutex.handle == handle) {
            mutex.locked = false;
            mutex.owner_thread_id = 0;
            true
        } else {
            false
        }
    }

    pub fn create_semaphore(&mut self, name: &str, initial_count: i32, maximum_count: i32) -> WinHandle {
        let handle = self.handles.allocate(
            WinHandleKind::Semaphore,
            0,
            self.semaphores.len() as u64,
            name,
            false,
        );
        self.semaphores.push(KernelSemaphore {
            handle,
            name: String::from(name),
            current_count: initial_count,
            maximum_count,
        });
        handle
    }

    pub fn release_semaphore(&mut self, handle: WinHandle, release_count: i32) -> bool {
        if let Some(semaphore) = self
            .semaphores
            .iter_mut()
            .find(|semaphore| semaphore.handle == handle)
        {
            semaphore.current_count = core::cmp::min(
                semaphore.current_count + release_count,
                semaphore.maximum_count,
            );
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct CriticalSection {
    pub debug_info: u64,
    pub lock_count: i32,
    pub recursion_count: i32,
    pub owning_thread: u32,
    pub lock_semaphore: WinHandle,
    pub spin_count: u32,
}

impl CriticalSection {
    pub fn new() -> Self {
        Self {
            debug_info: 0,
            lock_count: -1,
            recursion_count: 0,
            owning_thread: 0,
            lock_semaphore: 0,
            spin_count: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.lock_count = -1;
        self.recursion_count = 0;
        self.owning_thread = 0;
        self.lock_semaphore = 0;
    }

    pub fn enter(&mut self, thread_id: u32) {
        self.lock_count += 1;
        if self.owning_thread == 0 {
            self.owning_thread = thread_id;
        }
        self.recursion_count += 1;
    }

    pub fn leave(&mut self) {
        if self.recursion_count > 0 {
            self.recursion_count -= 1;
        }
        if self.recursion_count == 0 {
            self.owning_thread = 0;
            self.lock_count = -1;
        }
    }

    pub fn try_enter(&mut self, thread_id: u32) -> bool {
        if self.owning_thread == 0 || self.owning_thread == thread_id {
            self.enter(thread_id);
            true
        } else {
            false
        }
    }
}
