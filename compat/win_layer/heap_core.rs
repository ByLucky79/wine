// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Heap Manager
// File Path            : apps/system/compat/win_layer/heap_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;
use alloc::format;

pub struct HeapBlock {
    pub base: u64, pub size: u64, pub flags: u32,
}

pub struct HeapManager {
    pub blocks: Vec<HeapBlock>, pub base: u64, pub next: u64,
}

impl HeapManager {
    pub fn new(base: u64, _initial_size: u64) -> Self { Self { blocks: Vec::new(), base, next: base } }
    pub fn alloc(&mut self, size: u64, flags: u32) -> u64 {
        let aligned = (size + 0xF) & !0xF;
        let addr = self.next;
        self.blocks.push(HeapBlock { base: addr, size: aligned, flags });
        self.next += aligned;
        let msg = format!("{} HeapAlloc(addr=0x{:016X}, size=0x{:X}, flags=0x{:X})", "[WIN32]", addr, size, flags);
        console_writeln(&msg); addr
    }
    pub fn free(&mut self, addr: u64) -> bool {
        let before = self.blocks.len();
        self.blocks.retain(|b| b.base != addr);
        let after = self.blocks.len();
        let msg = format!("{} HeapFree(addr=0x{:016X}) -> {}", "[WIN32]", addr, before != after);
        console_writeln(&msg); before != after
    }
    pub fn realloc(&mut self, addr: u64, size: u64) -> u64 {
        let new_addr = self.alloc(size, 0);
        let msg = format!("{} HeapReAlloc(old=0x{:016X}, new=0x{:016X}, size=0x{:X})", "[WIN32]", addr, new_addr, size);
        console_writeln(&msg);
        self.free(addr); new_addr
    }
    pub fn size(&self, addr: u64) -> u64 { self.blocks.iter().find(|b| b.base == addr).map_or(0, |b| b.size) }
    pub fn dump(&self) {
        let msg = format!("{} Active blocks:", "[WIN32]");
        console_writeln(&msg);
        for b in &self.blocks { let msg = format!("  addr=0x{:016X} size=0x{:X} flags=0x{:X}", b.base, b.size, b.flags); console_writeln(&msg); }
    }
}
