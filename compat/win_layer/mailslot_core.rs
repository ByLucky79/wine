// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Mailslot Emulator
// File Path            : apps/system/compat/win_layer/mailslot_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Mailslot creation, read, write, and info operations for the
//   Win32 compatibility layer. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;

pub struct MailslotManager {
    pub slots: Vec<(String, u64, Vec<Vec<u8>>)>, pub next_handle: u64,
}

impl Default for MailslotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MailslotManager {
    pub fn new() -> Self { Self { slots: Vec::new(), next_handle: 0xE000 } }
    pub fn create_mailslot(&mut self, name: &str, _max_msg: u32, _timeout: u32) -> u64 {
        let h = self.next_handle; self.next_handle += 1;
        self.slots.push((String::from(name), h, Vec::new()));
        let msg = format!("{} CreateMailslot(\"{}\") -> 0x{:08X}", "[WIN32]", name, h);
        console_writeln(&msg); h
    }
    pub fn get_mailslot_info(&self, handle: u64, _max_msg: u64, _next_size: u64, _msg_count: u64, _read_timeout: u64) -> bool {
        let msg = format!("{} GetMailslotInfo(0x{:08X})", "[WIN32]", handle);
        console_writeln(&msg); true
    }
    pub fn write_mailslot(&mut self, name: &str, data: &[u8]) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|(n, _, _)| n == name) { slot.2.push(data.to_vec()); }
        let msg = format!("{} WriteMailslot(\"{}\", {} bytes)", "[WIN32]", name, data.len());
        console_writeln(&msg); true
    }
    pub fn read_mailslot(&mut self, handle: u64, buf: &mut [u8]) -> u32 {
        let count = if let Some(slot) = self.slots.iter_mut().find(|(_, h, _)| *h == handle) {
            if let Some(msg_data) = slot.2.first() {
                let n = buf.len().min(msg_data.len());
                buf[..n].copy_from_slice(&msg_data[..n]);
                slot.2.remove(0);
                n as u32
            } else { 0 }
        } else { 0 };
        let msg = format!("{} ReadMailslot(0x{:08X}, {} bytes)", "[WIN32]", handle, count);
        console_writeln(&msg); count
    }
}
