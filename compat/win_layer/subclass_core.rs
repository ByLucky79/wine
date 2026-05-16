// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Window Subclassing Emulator
// File Path            : apps/system/compat/win_layer/subclass_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Window subclass entry tracking and default subclass procedure
//   for the Win32 compatibility layer. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;

#[derive(Debug, Clone)]
pub struct SubclassEntry {
    pub hwnd: u64, pub proc: u64, pub id: u64, pub ref_data: u64,
}

pub struct SubclassManager {
    pub entries: Vec<SubclassEntry>,
}

impl Default for SubclassManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SubclassManager {
    pub fn new() -> Self { Self { entries: Vec::new() } }
    pub fn set_window_subclass(&mut self, hwnd: u64, proc: u64, id: u64, ref_data: u64) -> bool {
        self.entries.retain(|e| !(e.hwnd == hwnd && e.id == id));
        self.entries.push(SubclassEntry { hwnd, proc, id, ref_data });
        let msg = format!("[SUBCLASS] SetWindowSubclass(hwnd=0x{:08X}, proc=0x{:016X}, id={})", hwnd, proc, id);
        console_writeln(&msg); true
    }
    pub fn remove_window_subclass(&mut self, hwnd: u64, id: u64) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| !(e.hwnd == hwnd && e.id == id));
        let after = self.entries.len();
        let msg = format!("[SUBCLASS] RemoveWindowSubclass(hwnd=0x{:08X}, id={}) -> {}", hwnd, id, before != after);
        console_writeln(&msg); before != after
    }
    pub fn def_subclass_proc(hwnd: u64, msg: u32, wparam: u64, lparam: u64) -> i64 {
        let m = format!("[SUBCLASS] DefSubclassProc(hwnd=0x{:08X}, msg=0x{:04X}, wp=0x{:016X}, lp=0x{:016X})", hwnd, msg, wparam, lparam);
        console_writeln(&m); 0
    }
}
