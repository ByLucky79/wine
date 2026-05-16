// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Clipboard Manager
// File Path            : apps/system/compat/win_layer/clipboard_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;
use alloc::format;

pub struct ClipboardManager {
    pub data: BTreeMap<u32, Vec<u8>>, pub open: bool, pub owner: u64,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    pub fn new() -> Self { Self { data: BTreeMap::new(), open: false, owner: 0 } }
    pub fn open_clipboard(&mut self, owner: u64) -> bool {
        if self.open { return false; }
        self.open = true; self.owner = owner;
        let msg = format!("[CLIPBOARD] OpenClipboard(hwnd=0x{:08X})", owner);
        console_writeln(&msg); true
    }
    pub fn close_clipboard(&mut self) -> bool {
        if !self.open { return false; }
        self.open = false; self.owner = 0;
        console_writeln("[CLIPBOARD] CloseClipboard()"); true
    }
    pub fn empty_clipboard(&mut self) -> bool {
        self.data.clear();
        console_writeln("[CLIPBOARD] EmptyClipboard()"); true
    }
    pub fn set_clipboard_data(&mut self, format: u32, data: &[u8]) -> u64 {
        self.data.insert(format, data.to_vec());
        let handle = 0x9000 + format as u64;
        let msg = format!("[CLIPBOARD] SetClipboardData(fmt={}) -> 0x{:08X}", format, handle);
        console_writeln(&msg); handle
    }
    pub fn get_clipboard_data(&self, format: u32) -> Option<&Vec<u8>> {
        let msg = format!("[CLIPBOARD] GetClipboardData(fmt={})", format);
        console_writeln(&msg); self.data.get(&format)
    }
    pub fn enum_formats(&self) -> Vec<u32> { self.data.keys().copied().collect() }
}
