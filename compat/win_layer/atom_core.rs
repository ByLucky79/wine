// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Atom Table Manager
// File Path            : apps/system/compat/win_layer/atom_core.rs
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

pub struct AtomTable {
    pub atoms: Vec<(u16, String)>, pub next_id: u16,
}

impl Default for AtomTable {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomTable {
    pub fn new() -> Self { Self { atoms: Vec::new(), next_id: 0xC000 } }
    pub fn add(&mut self, name: &str) -> u16 {
        if let Some((id, _)) = self.atoms.iter().find(|(_, n)| n == name) { return *id; }
        let id = self.next_id; self.next_id += 1;
        self.atoms.push((id, String::from(name)));
        let msg = format!("[ATOM] GlobalAddAtom(\"{}\") -> 0x{:04X}", name, id);
        console_writeln(&msg); id
    }
    pub fn delete(&mut self, atom: u16) -> bool {
        let before = self.atoms.len();
        self.atoms.retain(|(id, _)| *id != atom);
        let after = self.atoms.len();
        let msg = format!("[ATOM] GlobalDeleteAtom(0x{:04X}) -> {}", atom, before != after);
        console_writeln(&msg); before != after
    }
    pub fn find(&self, atom: u16) -> Option<String> {
        self.atoms.iter().find(|(id, _)| *id == atom).map(|(_, n)| n.clone())
    }
    pub fn find_by_name(&self, name: &str) -> Option<u16> {
        self.atoms.iter().find(|(_, n)| n == name).map(|(id, _)| *id)
    }
}
