// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Linux ELF64 Compatibility Layer
// File Path            : apps/system/compat/src/linux_elf.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32, RISC-V 64, MIPS 32, MIPS 64, PowerPC 32, PowerPC 64, m68k, SPARC, LoongArch64
//
// Description:
//   Linux ELF64 binary format parser and loader.
//   Header, Program Header, Section Header, Dynamic, Symbol, Relocation
//   Parses Header, Program Header, Section Header, Dynamic, Symbol, Relocation
//   tables. Contains x86_64 Linux syscall table (syscall 0-353)
//
// Dependent Files:
//   1-) apps/system/compat/src/dos_emulator.rs  (console_writeln)
//   2-) kernel/graphics/ui/src/lang/lang_flags.rs (MsgId)
//   3-) kernel/graphics/ui/src/lang/lang_manager.rs (Lang::get)
//
//              File Modifications
// 2026-04-17      C → Rust translation (no_std), full ELF64 parser
// 2026-04-18      Lang system, error type, #[must_use], unit tests
// *******************************************************************

#![allow(dead_code)]

use crate::dos_emulator::console_writeln;
use kernel_ui::{Lang, MsgId};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

// ─── ELF64 Header Parser ───────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ElfHeader64 {
    pub e_ident: [u8; 16], pub e_type: u16, pub e_machine: u16,
    pub e_version: u32, pub e_entry: u64, pub e_phoff: u64,
    pub e_shoff: u64, pub e_flags: u32, pub e_ehsize: u16,
    pub e_phentsize: u16, pub e_phnum: u16, pub e_shentsize: u16,
    pub e_shnum: u16, pub e_shstrndx: u16,
}

impl ElfHeader64 {
    pub const ELFMAG: [u8; 4] = [0x7F, b'E', b'L', b'F'];
    pub const ELFCLASS64: u8 = 2;
    pub const ELFDATA2LSB: u8 = 1;
    pub const EV_CURRENT: u8 = 1;
    pub const ET_EXEC: u16 = 2;
    pub const ET_DYN: u16 = 3;
    pub const EM_X86_64: u16 = 62;
    pub const EM_AARCH64: u16 = 183;
    pub const EM_RISCV: u16 = 243;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 64 { return None; }
        if data[0..4] != Self::ELFMAG { return None; }
        if data[4] != Self::ELFCLASS64 { return None; }
        Some(Self {
            e_ident: { let mut id = [0u8; 16]; id.copy_from_slice(&data[0..16]); id },
            e_type: u16::from_le_bytes([data[16], data[17]]),
            e_machine: u16::from_le_bytes([data[18], data[19]]),
            e_version: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            e_entry: u64::from_le_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]),
            e_phoff: u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]),
            e_shoff: u64::from_le_bytes([data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47]]),
            e_flags: u32::from_le_bytes([data[48], data[49], data[50], data[51]]),
            e_ehsize: u16::from_le_bytes([data[52], data[53]]),
            e_phentsize: u16::from_le_bytes([data[54], data[55]]),
            e_phnum: u16::from_le_bytes([data[56], data[57]]),
            e_shentsize: u16::from_le_bytes([data[58], data[59]]),
            e_shnum: u16::from_le_bytes([data[60], data[61]]),
            e_shstrndx: u16::from_le_bytes([data[62], data[63]]),
        })
    }
    #[must_use]
    pub fn is_executable(&self) -> bool { self.e_type == Self::ET_EXEC || self.e_type == Self::ET_DYN }
    #[must_use]
    pub fn machine_name(&self) -> &'static str {
        match self.e_machine {
            62 => "x86_64", 183 => "AArch64", 243 => "RISC-V",
            40 => "ARM", 3 => "x86", 8 => "MIPS", _ => "Unknown",
        }
    }
}

// ─── ELF64 Program Header Parser ───────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ElfProgramHeader64 {
    pub p_type: u32, pub p_flags: u32, pub p_offset: u64,
    pub p_vaddr: u64, pub p_paddr: u64, pub p_filesz: u64,
    pub p_memsz: u64, pub p_align: u64,
}

impl ElfProgramHeader64 {
    pub const PT_LOAD: u32 = 1;
    pub const PT_DYNAMIC: u32 = 2;
    pub const PT_INTERP: u32 = 3;
    pub const PT_NOTE: u32 = 4;
    pub const PT_GNU_EH_FRAME: u32 = 0x6474E550;
    pub const PT_GNU_STACK: u32 = 0x6474E551;
    pub const PT_GNU_RELRO: u32 = 0x6474E552;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 56 { return None; }
        Some(Self {
            p_type: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            p_flags: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            p_offset: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]),
            p_vaddr: u64::from_le_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]),
            p_paddr: u64::from_le_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]),
            p_filesz: u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]),
            p_memsz: u64::from_le_bytes([data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47]]),
            p_align: u64::from_le_bytes([data[48], data[49], data[50], data[51], data[52], data[53], data[54], data[55]]),
        })
    }
    #[must_use]
    pub fn is_readable(&self) -> bool { (self.p_flags & 4) != 0 }
    #[must_use]
    pub fn is_writable(&self) -> bool { (self.p_flags & 2) != 0 }
    #[must_use]
    pub fn is_executable(&self) -> bool { (self.p_flags & 1) != 0 }
}

// ─── ELF64 Section Header Parser ───────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ElfSectionHeader64 {
    pub sh_name: u32, pub sh_type: u32, pub sh_flags: u64,
    pub sh_addr: u64, pub sh_offset: u64, pub sh_size: u64,
    pub sh_link: u32, pub sh_info: u32, pub sh_addralign: u64, pub sh_entsize: u64,
}

impl ElfSectionHeader64 {
    pub const SHT_NULL: u32 = 0;
    pub const SHT_PROGBITS: u32 = 1;
    pub const SHT_SYMTAB: u32 = 2;
    pub const SHT_STRTAB: u32 = 3;
    pub const SHT_RELA: u32 = 4;
    pub const SHT_HASH: u32 = 5;
    pub const SHT_DYNAMIC: u32 = 6;
    pub const SHT_NOTE: u32 = 7;
    pub const SHT_NOBITS: u32 = 8;
    pub const SHT_REL: u32 = 9;
    pub const SHT_DYNSYM: u32 = 11;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 64 { return None; }
        Some(Self {
            sh_name: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            sh_type: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            sh_flags: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]),
            sh_addr: u64::from_le_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]),
            sh_offset: u64::from_le_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]),
            sh_size: u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]),
            sh_link: u32::from_le_bytes([data[40], data[41], data[42], data[43]]),
            sh_info: u32::from_le_bytes([data[44], data[45], data[46], data[47]]),
            sh_addralign: u64::from_le_bytes([data[48], data[49], data[50], data[51], data[52], data[53], data[54], data[55]]),
            sh_entsize: u64::from_le_bytes([data[56], data[57], data[58], data[59], data[60], data[61], data[62], data[63]]),
        })
    }
}

// ─── ELF64 Dynamic Entry Parser ────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ElfDynamicEntry64 {
    pub d_tag: i64, pub d_val: u64,
}

impl ElfDynamicEntry64 {
    pub const DT_NULL: i64 = 0;
    pub const DT_NEEDED: i64 = 1;
    pub const DT_PLTRELSZ: i64 = 2;
    pub const DT_PLTGOT: i64 = 3;
    pub const DT_HASH: i64 = 4;
    pub const DT_STRTAB: i64 = 5;
    pub const DT_SYMTAB: i64 = 6;
    pub const DT_RELA: i64 = 7;
    pub const DT_RELASZ: i64 = 8;
    pub const DT_RELAENT: i64 = 9;
    pub const DT_STRSZ: i64 = 10;
    pub const DT_SYMENT: i64 = 11;
    pub const DT_INIT: i64 = 12;
    pub const DT_FINI: i64 = 13;
    pub const DT_SONAME: i64 = 14;
    pub const DT_RPATH: i64 = 15;
    pub const DT_SYMBOLIC: i64 = 16;
    pub const DT_REL: i64 = 17;
    pub const DT_RELSZ: i64 = 18;
    pub const DT_RELENT: i64 = 19;
    pub const DT_PLTREL: i64 = 20;
    pub const DT_DEBUG: i64 = 21;
    pub const DT_TEXTREL: i64 = 22;
    pub const DT_JMPREL: i64 = 23;
    pub const DT_BIND_NOW: i64 = 24;
    pub const DT_INIT_ARRAY: i64 = 25;
    pub const DT_FINI_ARRAY: i64 = 26;
    pub const DT_INIT_ARRAYSZ: i64 = 27;
    pub const DT_FINI_ARRAYSZ: i64 = 28;
    pub const DT_RUNPATH: i64 = 29;
    pub const DT_FLAGS: i64 = 30;
    pub const DT_GNU_HASH: i64 = 0x6FFFFEF5;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { return None; }
        Some(Self {
            d_tag: i64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]),
            d_val: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]),
        })
    }
}

// ─── ELF64 Symbol Parser ───────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ElfSymbol64 {
    pub st_name: u32, pub st_info: u8, pub st_other: u8,
    pub st_shndx: u16, pub st_value: u64, pub st_size: u64,
}

impl ElfSymbol64 {
    pub const STB_GLOBAL: u8 = 1;
    pub const STB_WEAK: u8 = 2;
    pub const STT_FUNC: u8 = 2;
    pub const STT_OBJECT: u8 = 1;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 24 { return None; }
        Some(Self {
            st_name: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            st_info: data[4], st_other: data[5],
            st_shndx: u16::from_le_bytes([data[6], data[7]]),
            st_value: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]),
            st_size: u64::from_le_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]),
        })
    }
    #[must_use]
    pub fn bind(&self) -> u8 { self.st_info >> 4 }
    #[must_use]
    pub fn type_(&self) -> u8 { self.st_info & 0x0F }
    #[must_use]
    pub fn is_global(&self) -> bool { self.bind() == Self::STB_GLOBAL || self.bind() == Self::STB_WEAK }
}

// ─── ELF64 Relocation Parser ───────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ElfRela64 {
    pub r_offset: u64, pub r_info: u64, pub r_addend: i64,
}

impl ElfRela64 {
    pub const R_X86_64_64: u32 = 1;
    pub const R_X86_64_PC32: u32 = 2;
    pub const R_X86_64_GOT32: u32 = 3;
    pub const R_X86_64_PLT32: u32 = 4;
    pub const R_X86_64_GLOB_DAT: u32 = 6;
    pub const R_X86_64_JUMP_SLOT: u32 = 7;
    pub const R_X86_64_RELATIVE: u32 = 8;

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 24 { return None; }
        Some(Self {
            r_offset: u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]),
            r_info: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]),
            r_addend: i64::from_le_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]),
        })
    }
    #[must_use]
    pub fn sym(&self) -> u32 { (self.r_info >> 32) as u32 }
    #[must_use]
    pub fn type_(&self) -> u32 { (self.r_info & 0xFFFFFFFF) as u32 }
}

#[derive(Debug, Clone, Copy)]
pub struct ElfRel64 {
    pub r_offset: u64, pub r_info: u64,
}

impl ElfRel64 {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { return None; }
        Some(Self {
            r_offset: u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]),
            r_info: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]),
        })
    }
    #[must_use]
    pub fn sym(&self) -> u32 { (self.r_info >> 32) as u32 }
    #[must_use]
    pub fn type_(&self) -> u32 { (self.r_info & 0xFFFFFFFF) as u32 }
}

// ─── ELF String Table ──────────────────────────────────────────

pub struct ElfStringTable;

impl ElfStringTable {
    pub fn get(data: &[u8], offset: u32) -> String {
        let off = offset as usize;
        if off >= data.len() { return String::new(); }
        let end = data[off..].iter().position(|&b| b == 0).unwrap_or(data.len() - off);
        core::str::from_utf8(&data[off..off + end]).unwrap_or("").to_string()
    }
}

// ─── ELF Loader ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ElfLoadedImage {
    pub header: ElfHeader64, pub phdrs: Vec<ElfProgramHeader64>,
    pub shdrs: Vec<ElfSectionHeader64>, pub dynamic: Vec<ElfDynamicEntry64>,
    pub symbols: Vec<ElfSymbol64>, pub relas: Vec<ElfRela64>,
    pub needed: Vec<String>, pub base: u64,
    pub memory: Vec<u8>, pub mem_base: u64, pub mem_size: u64,
}

impl ElfLoadedImage {
    pub fn read_u64(&self, addr: u64) -> u64 {
        let off = (addr - self.mem_base) as usize;
        if off + 8 <= self.memory.len() { u64::from_le_bytes([self.memory[off], self.memory[off+1], self.memory[off+2], self.memory[off+3], self.memory[off+4], self.memory[off+5], self.memory[off+6], self.memory[off+7]]) } else { 0 }
    }
    pub fn write_u64(&mut self, addr: u64, val: u64) {
        let off = (addr - self.mem_base) as usize;
        if off + 8 <= self.memory.len() { let b = val.to_le_bytes(); self.memory[off..off+8].copy_from_slice(&b); }
    }
    pub fn write_u32(&mut self, addr: u64, val: u32) {
        let off = (addr - self.mem_base) as usize;
        if off + 4 <= self.memory.len() { let b = val.to_le_bytes(); self.memory[off..off+4].copy_from_slice(&b); }
    }
    #[must_use]
    pub fn get_entry_point(&self) -> u64 { self.mem_base + self.header.e_entry }
}

pub struct ElfLoader;

impl ElfLoader {
    pub fn parse(data: &[u8]) -> Option<ElfLoadedImage> {
        let hdr = ElfHeader64::parse(data)?;
        let mut phdrs = Vec::new();
        for i in 0..hdr.e_phnum {
            let off = hdr.e_phoff as usize + i as usize * hdr.e_phentsize as usize;
            if data.len() < off + 56 { break; }
            if let Some(ph) = ElfProgramHeader64::parse(&data[off..]) { phdrs.push(ph); }
        }
        let mut shdrs = Vec::new();
        for i in 0..hdr.e_shnum {
            let off = hdr.e_shoff as usize + i as usize * hdr.e_shentsize as usize;
            if data.len() < off + 64 { break; }
            if let Some(sh) = ElfSectionHeader64::parse(&data[off..]) { shdrs.push(sh); }
        }
        let mut dynamic = Vec::new();
        let mut needed = Vec::new();
        let mut symbols = Vec::new();
        let mut relas = Vec::new();
        for ph in &phdrs {
            if ph.p_type == ElfProgramHeader64::PT_DYNAMIC {
                let mut off = ph.p_offset as usize;
                while off + 16 <= data.len() {
                    if let Some(entry) = ElfDynamicEntry64::parse(&data[off..]) {
                        if entry.d_tag == ElfDynamicEntry64::DT_NULL { break; }
                        dynamic.push(entry);
                        off += 16;
                    } else { break; }
                }
            }
        }
        let mut strtab_off: u64 = 0;
        let mut strtab_size: u64 = 0;
        let mut symtab_off: u64 = 0;
        let mut symtab_entsize: u64 = 24;
        let mut rela_off: u64 = 0;
        let mut rela_entsize: u64 = 24;
        let mut relasz: u64 = 0;
        for e in &dynamic {
            match e.d_tag {
                ElfDynamicEntry64::DT_STRTAB => strtab_off = e.d_val,
                ElfDynamicEntry64::DT_STRSZ => strtab_size = e.d_val,
                ElfDynamicEntry64::DT_SYMTAB => symtab_off = e.d_val,
                ElfDynamicEntry64::DT_SYMENT => symtab_entsize = e.d_val,
                ElfDynamicEntry64::DT_RELA => rela_off = e.d_val,
                ElfDynamicEntry64::DT_RELAENT => rela_entsize = e.d_val,
                ElfDynamicEntry64::DT_RELASZ => relasz = e.d_val,
                _ => {}
            }
        }
        if strtab_off > 0 && strtab_size > 0 {
            let end = ((strtab_off + strtab_size) as usize).min(data.len());
            let str_data = &data[strtab_off as usize..end];
            for e in &dynamic {
                if e.d_tag == ElfDynamicEntry64::DT_NEEDED {
                    needed.push(ElfStringTable::get(str_data, e.d_val as u32));
                }
            }
            if symtab_off > 0 && symtab_entsize > 0 {
                let count = ((data.len() as u64 - symtab_off) / symtab_entsize).min(4096);
                for i in 0..count {
                    let off = symtab_off as usize + i as usize * symtab_entsize as usize;
                    if data.len() < off + 24 { break; }
                    if let Some(sym) = ElfSymbol64::parse(&data[off..]) { symbols.push(sym); }
                }
            }
            if rela_off > 0 && rela_entsize > 0 && relasz > 0 {
                let count = relasz / rela_entsize;
                for i in 0..count {
                    let off = rela_off as usize + i as usize * rela_entsize as usize;
                    if data.len() < off + 24 { break; }
                    if let Some(r) = ElfRela64::parse(&data[off..]) { relas.push(r); }
                }
            }
        }
        let msg = format!("{}: {} phdrs, {} shdrs, {} dynamic, {} symbols, {} relas, needed={:?}",
            &Lang::get(MsgId::CompatElfMapped),
            phdrs.len(), shdrs.len(), dynamic.len(), symbols.len(), relas.len(), needed);
        console_writeln(&msg);
        Some(ElfLoadedImage { header: hdr, phdrs, shdrs, dynamic, symbols, relas, needed, base: 0, mem_base: 0, mem_size: 0, memory: Vec::new() })
    }
}

// ─── Linux x86_64 Syscall Table ────────────────────────────────

pub struct LinuxSyscallTable;

impl LinuxSyscallTable {
    pub fn lookup(num: u64) -> &'static str {
        match num {
            0 => "read", 1 => "write", 2 => "open", 3 => "close", 4 => "stat", 5 => "fstat",
            6 => "lstat", 7 => "poll", 8 => "lseek", 9 => "mmap", 10 => "mprotect", 11 => "munmap",
            12 => "brk", 13 => "rt_sigaction", 14 => "rt_sigprocmask", 15 => "rt_sigreturn",
            16 => "ioctl", 17 => "pread64", 18 => "pwrite64", 19 => "readv", 20 => "writev",
            21 => "access", 22 => "pipe", 23 => "select", 24 => "sched_yield", 25 => "mremap",
            26 => "msync", 27 => "mincore", 28 => "madvise", 29 => "shmget", 30 => "shmat",
            31 => "shmctl", 32 => "dup", 33 => "dup2", 34 => "pause", 35 => "nanosleep",
            36 => "getitimer", 37 => "alarm", 38 => "setitimer", 39 => "getpid", 40 => "sendfile",
            41 => "socket", 42 => "connect", 43 => "accept", 44 => "sendto", 45 => "recvfrom",
            46 => "sendmsg", 47 => "recvmsg", 48 => "shutdown", 49 => "bind", 50 => "listen",
            51 => "getsockname", 52 => "getpeername", 53 => "socketpair", 54 => "setsockopt",
            55 => "getsockopt", 56 => "clone", 57 => "fork", 58 => "vfork", 59 => "execve",
            60 => "exit", 61 => "wait4", 62 => "kill", 63 => "uname", 64 => "semget",
            65 => "semop", 66 => "semctl", 67 => "shmdt", 68 => "msgget", 69 => "msgsnd",
            70 => "msgrcv", 71 => "msgctl", 72 => "fcntl", 73 => "flock", 74 => "fsync",
            75 => "fdatasync", 76 => "truncate", 77 => "ftruncate", 78 => "getcwd", 79 => "chdir",
            80 => "fchdir", 81 => "rename", 82 => "mkdir", 83 => "rmdir", 84 => "creat",
            85 => "link", 86 => "unlink", 87 => "symlink", 88 => "readlink", 89 => "chmod",
            90 => "fchmod", 91 => "chown", 92 => "fchown", 93 => "lchown", 94 => "umask",
            95 => "gettimeofday", 96 => "getrlimit", 97 => "getrusage", 98 => "sysinfo",
            99 => "times", 100 => "ptrace", 101 => "getuid", 102 => "getgid", 103 => "geteuid",
            104 => "getegid", 105 => "setpgid", 106 => "getppid", 107 => "getpgrp", 108 => "setsid",
            109 => "setreuid", 110 => "setregid", 111 => "getgroups", 112 => "setgroups",
            113 => "setresuid", 114 => "getresuid", 115 => "setresgid", 116 => "getresgid",
            117 => "getpgid", 118 => "setfsuid", 119 => "setfsgid", 120 => "getsid",
            121 => "capget", 122 => "capset", 123 => "rt_sigpending", 124 => "rt_sigtimedwait",
            125 => "rt_sigqueueinfo", 126 => "rt_sigsuspend", 127 => "sigaltstack", 128 => "utime",
            129 => "mknod", 130 => "personality", 131 => "ustat", 132 => "statfs", 133 => "fstatfs",
            134 => "sysfs", 135 => "getpriority", 136 => "setpriority", 137 => "sched_setparam",
            138 => "sched_getparam", 139 => "sched_setscheduler", 140 => "sched_getscheduler",
            141 => "sched_get_priority_max", 142 => "sched_get_priority_min", 143 => "sched_rr_get_interval",
            144 => "mlock", 145 => "munlock", 146 => "mlockall", 147 => "munlockall", 148 => "vhangup",
            149 => "modify_ldt", 150 => "pivot_root", 151 => "prctl", 152 => "arch_prctl",
            153 => "adjtimex", 154 => "setrlimit", 155 => "chroot", 156 => "sync", 157 => "acct",
            158 => "settimeofday", 159 => "mount", 160 => "umount2", 161 => "swapon", 162 => "swapoff",
            163 => "reboot", 164 => "sethostname", 165 => "setdomainname", 166 => "iopl",
            167 => "ioperm", 168 => "create_module", 169 => "init_module", 170 => "delete_module",
            171 => "get_kernel_syms", 172 => "query_module", 173 => "quotactl", 174 => "nfsservctl",
            175 => "getpmsg", 176 => "putpmsg", 177 => "afs_syscall", 178 => "tuxcall",
            179 => "security", 180 => "gettid", 181 => "readahead", 182 => "setxattr",
            183 => "lsetxattr", 184 => "fsetxattr", 185 => "getxattr", 186 => "lgetxattr",
            187 => "fgetxattr", 188 => "listxattr", 189 => "llistxattr", 190 => "flistxattr",
            191 => "removexattr", 192 => "lremovexattr", 193 => "fremovexattr", 194 => "tkill",
            195 => "time", 196 => "futex", 197 => "sched_setaffinity", 198 => "sched_getaffinity",
            199 => "set_thread_area", 200 => "io_setup", 201 => "io_destroy", 202 => "io_getevents",
            203 => "io_submit", 204 => "io_cancel", 205 => "get_thread_area", 206 => "lookup_dcookie",
            207 => "epoll_create", 208 => "epoll_ctl_old", 209 => "epoll_wait_old", 210 => "remap_file_pages",
            211 => "getdents64", 212 => "set_tid_address", 213 => "restart_syscall", 214 => "semtimedop",
            215 => "fadvise64", 216 => "timer_create", 217 => "timer_settime", 218 => "timer_gettime",
            219 => "timer_getoverrun", 220 => "timer_delete", 221 => "clock_settime", 222 => "clock_gettime",
            223 => "clock_getres", 224 => "clock_nanosleep", 225 => "exit_group", 226 => "epoll_wait",
            227 => "epoll_ctl", 228 => "tgkill", 229 => "utimes", 230 => "vserver", 231 => "mbind",
            232 => "set_mempolicy", 233 => "get_mempolicy", 234 => "mq_open", 235 => "mq_unlink",
            236 => "mq_timedsend", 237 => "mq_timedreceive", 238 => "mq_notify", 239 => "mq_getsetattr",
            240 => "kexec_load", 241 => "waitid", 242 => "add_key", 243 => "request_key", 244 => "keyctl",
            245 => "ioprio_set", 246 => "ioprio_get", 247 => "inotify_init", 248 => "inotify_add_watch",
            249 => "inotify_rm_watch", 250 => "migrate_pages", 251 => "openat", 252 => "mkdirat",
            253 => "mknodat", 254 => "fchownat", 255 => "futimesat", 256 => "newfstatat", 257 => "unlinkat",
            258 => "renameat", 259 => "linkat", 260 => "symlinkat", 261 => "readlinkat", 262 => "fchmodat",
            263 => "faccessat", 264 => "pselect6", 265 => "ppoll", 266 => "unshare", 267 => "set_robust_list",
            268 => "get_robust_list", 269 => "splice", 270 => "tee", 271 => "sync_file_range",
            272 => "vmsplice", 273 => "move_pages", 274 => "utimensat", 275 => "epoll_pwait",
            276 => "signalfd", 277 => "timerfd_create", 278 => "eventfd", 279 => "fallocate",
            280 => "timerfd_settime", 281 => "timerfd_gettime", 282 => "accept4", 283 => "signalfd4",
            284 => "eventfd2", 285 => "epoll_create1", 286 => "dup3", 287 => "pipe2", 288 => "inotify_init1",
            289 => "preadv", 290 => "pwritev", 291 => "rt_tgsigqueueinfo", 292 => "perf_event_open",
            293 => "recvmmsg", 294 => "fanotify_init", 295 => "fanotify_mark", 296 => "prlimit64",
            297 => "name_to_handle_at", 298 => "open_by_handle_at", 299 => "clock_adjtime",
            300 => "syncfs", 301 => "setns", 302 => "getcpu", 303 => "process_vm_readv",
            304 => "process_vm_writev", 305 => "kcmp", 306 => "finit_module", 307 => "sched_setattr",
            308 => "sched_getattr", 309 => "renameat2", 310 => "seccomp", 311 => "getrandom",
            312 => "memfd_create", 313 => "kexec_file_load", 314 => "bpf", 315 => "stub_execveat",
            316 => "userfaultfd", 317 => "membarrier", 318 => "mlock2", 319 => "copy_file_range",
            320 => "preadv2", 321 => "pwritev2", 322 => "pkey_mprotect", 323 => "pkey_alloc",
            324 => "pkey_free", 325 => "statx", 326 => "io_pgetevents", 327 => "rseq",
            328 => "pidfd_send_signal", 329 => "io_uring_setup", 330 => "io_uring_enter",
            331 => "io_uring_register", 332 => "open_tree", 333 => "move_mount", 334 => "fsopen",
            335 => "fsconfig", 336 => "fsmount", 337 => "fspick", 338 => "pidfd_open",
            339 => "clone3", 340 => "openat2", 341 => "pidfd_getfd", 342 => "faccessat2",
            343 => "process_madvise", 344 => "epoll_pwait2", 345 => "mount_setattr",
            346 => "quotactl_fd", 347 => "landlock_create_ruleset", 348 => "landlock_add_rule",
            349 => "landlock_restrict_self", 350 => "memfd_secret", 351 => "process_mrelease",
            352 => "futex_waitv", 353 => "set_mempolicy_home_node",
            _ => "unknown",
        }
    }
    pub fn dump_table() {
        console_writeln(&Lang::get(MsgId::CompatLinuxSyscall));
        for i in 0..=353u64 {
            let name = Self::lookup(i);
            if name != "unknown" {
                let msg = format!("  {} => {}", i, name);
                console_writeln(&msg);
            }
        }
    }
}

// ─── Linux Syscall Emulator ────────────────────────────────────

pub struct LinuxSyscallEmulator;

impl LinuxSyscallEmulator {
    pub fn syscall(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64, arg6: u64) -> i64 {
        let name = LinuxSyscallTable::lookup(num);
        let msg = format!("{}: syscall({}, {}, {}, {}, {}, {}, {}) -> {}",
            &Lang::get(MsgId::CompatLinuxSyscall), num, arg1, arg2, arg3, arg4, arg5, arg6, name);
        console_writeln(&msg);
        match num {
            0 => arg3 as i64, // read -> return count
            1 => arg3 as i64, // write -> return count
            2 => 3, // open -> return fd
            3 => 0, // close
            9 => arg1 as i64, // mmap -> return addr
            10 => 0, // mprotect
            11 => 0, // munmap
            12 => arg1 as i64, // brk -> return new brk
            39 => 0xABCD, // getpid
            41 => 4, // socket -> return fd
            42 => 0, // connect
            43 => 5, // accept -> return fd
            56 => 0x1234, // clone -> return pid
            57 => 0x1235, // fork -> return pid
            59 => 0, // execve
            60 => 0, // exit
            61 => 0x1235, // wait4 -> return pid
            63 => 0, // uname
            78 => arg1 as i64, // getcwd
            102 => 1000, // getuid
            103 => 1000, // getgid
            104 => 1000, // geteuid
            105 => 1000, // getegid
            186 => 0, // gettid
            222 => 0, // clock_gettime
            _ => -1,
        }
    }
}

// ─── Linux Process Emulator ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LinuxProcess {
    pub pid: u32, pub ppid: u32, pub uid: u32, pub gid: u32,
    pub cwd: String, pub argv: Vec<String>, pub envp: BTreeMap<String, String>,
    pub brk: u64, pub image: Option<ElfLoadedImage>,
}

impl LinuxProcess {
    pub fn new(pid: u32) -> Self {
        let mut envp = BTreeMap::new();
        envp.insert(String::from("PATH"), String::from("/bin:/usr/bin:/sbin:/usr/sbin"));
        envp.insert(String::from("HOME"), String::from("/root"));
        envp.insert(String::from("USER"), String::from("root"));
        envp.insert(String::from("TERM"), String::from("linux"));
        envp.insert(String::from("LANG"), String::from("en_US.UTF-8"));
        Self { pid, ppid: 1, uid: 0, gid: 0, cwd: String::from("/"), argv: Vec::new(), envp, brk: 0x20000000, image: None }
    }
    pub fn load_elf(&mut self, data: &[u8]) -> bool {
        match ElfLoader::parse(data) {
            Some(img) => {
                let msg = format!("{}: pid={} entry=0x{:016X}",
                    &Lang::get(MsgId::CompatElfMapped), self.pid, img.header.e_entry);
                console_writeln(&msg);
                self.image = Some(img); true
            }
            None => { console_writeln(&Lang::get(MsgId::Error)); false }
        }
    }
    pub fn get_env(&self, key: &str) -> Option<String> { self.envp.get(key).cloned() }
    pub fn set_env(&mut self, key: &str, value: &str) { self.envp.insert(String::from(key), String::from(value)); }
    pub fn dump(&self) {
        let msg = format!("{} pid={} ppid={} uid={} gid={} cwd={} brk=0x{:X}", &Lang::get(MsgId::CompatLinuxProcInfo), self.pid, self.ppid, self.uid, self.gid, self.cwd, self.brk);
        console_writeln(&msg);
        for (k, v) in &self.envp { let msg = format!("  {}={}", k, v); console_writeln(&msg); }
    }
}

// ─── ELF Loader Memory Mapping & Relocation ────────────────────

impl ElfLoader {
    pub fn load_and_map(data: &[u8], base: u64) -> Option<ElfLoadedImage> {
        let mut img = Self::parse(data)?;
        let mut mem_size = 0u64;
        for ph in &img.phdrs {
            if ph.p_type == ElfProgramHeader64::PT_LOAD {
                let end = ph.p_vaddr + ph.p_memsz;
                if end > mem_size { mem_size = end; }
            }
        }
        if mem_size == 0 { mem_size = data.len() as u64; }
        let mut memory = vec![0u8; mem_size as usize];
        for ph in &img.phdrs {
            if ph.p_type == ElfProgramHeader64::PT_LOAD {
                let file_off = ph.p_offset as usize;
                let mem_off = ph.p_vaddr as usize;
                let file_size = ph.p_filesz as usize;
                if file_off + file_size <= data.len() && mem_off + file_size <= memory.len() {
                    memory[mem_off..mem_off + file_size].copy_from_slice(&data[file_off..file_off + file_size]);
                }
            }
        }
        img.memory = memory;
        img.mem_base = base;
        img.mem_size = mem_size;
        img.base = base;
        let msg = format!("{}: {} bytes @ base=0x{:016X}",
            &Lang::get(MsgId::CompatElfMapped), mem_size, base);
        console_writeln(&msg);
        Self::apply_relocations(&mut img);
        Some(img)
    }
    pub fn apply_relocations(img: &mut ElfLoadedImage) {
        let mut count = 0u32;
        let relas = img.relas.clone();
        for rela in &relas {
            let addr = img.mem_base + rela.r_offset;
            match rela.type_() {
                ElfRela64::R_X86_64_64 => {
                    let sym_val = if rela.sym() < img.symbols.len() as u32 { img.symbols[rela.sym() as usize].st_value } else { 0 };
                    let val = sym_val.wrapping_add(rela.r_addend as u64);
                    img.write_u64(addr, val); count += 1;
                }
                ElfRela64::R_X86_64_RELATIVE => {
                    let val = img.mem_base.wrapping_add(rela.r_addend as u64);
                    img.write_u64(addr, val); count += 1;
                }
                ElfRela64::R_X86_64_GLOB_DAT | ElfRela64::R_X86_64_JUMP_SLOT => {
                    let sym_val = if rela.sym() < img.symbols.len() as u32 { img.symbols[rela.sym() as usize].st_value } else { 0 };
                    img.write_u64(addr, sym_val); count += 1;
                }
                ElfRela64::R_X86_64_PC32 => {
                    let sym_val = if rela.sym() < img.symbols.len() as u32 { img.symbols[rela.sym() as usize].st_value } else { 0 };
                    let val = (sym_val as i64 + rela.r_addend - addr as i64) as u32;
                    img.write_u32(addr, val); count += 1;
                }
                _ => {}
            }
        }
        let msg = format!("{}: {} relocation",
            &Lang::get(MsgId::CompatElfRelocs), count);
        console_writeln(&msg);
    }
    #[must_use]
    pub fn resolve_symbol(img: &ElfLoadedImage, name: &str) -> Option<u64> {
        for sym in &img.symbols {
            if sym.is_global() && sym.st_name > 0 {
                let sym_name = ElfStringTable::get(&img.memory, sym.st_name);
                if sym_name == name { return Some(img.mem_base + sym.st_value); }
            }
        }
        None
    }
}

// ─── Unit Tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_elf64_exec() -> Vec<u8> {
        // Minimal valid ELF64 exec header (64 bytes)
        let mut data = vec![0u8; 128];
        data[0..4].copy_from_slice(&[0x7F, b'E', b'L', b'F']);
        data[4] = 2;    // ELFCLASS64
        data[5] = 1;    // ELFDATA2LSB
        data[6] = 1;    // EV_CURRENT
        data[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
        data[18..20].copy_from_slice(&62u16.to_le_bytes()); // EM_X86_64
        data[20..24].copy_from_slice(&1u32.to_le_bytes()); // e_version
        data[24..32].copy_from_slice(&0x401000u64.to_le_bytes()); // e_entry
        data
    }

    #[test]
    fn test_elf_header_parse_valid() {
        let data = minimal_elf64_exec();
        let hdr = ElfHeader64::parse(&data);
        assert!(hdr.is_some());
        let hdr = hdr.unwrap();
        assert_eq!(hdr.e_type, ElfHeader64::ET_EXEC);
        assert_eq!(hdr.e_machine, ElfHeader64::EM_X86_64);
        assert_eq!(hdr.e_entry, 0x401000);
    }

    #[test]
    fn test_elf_header_parse_invalid_magic() {
        let mut data = minimal_elf64_exec();
        data[0] = 0x00; // corrupt magic
        assert!(ElfHeader64::parse(&data).is_none());
    }

    #[test]
    fn test_elf_header_parse_too_short() {
        let data = vec![0x7F, b'E', b'L', b'F'];
        assert!(ElfHeader64::parse(&data).is_none());
    }

    #[test]
    fn test_elf_header_parse_32bit_class() {
        let mut data = minimal_elf64_exec();
        data[4] = 1; // ELFCLASS32 — should be rejected
        assert!(ElfHeader64::parse(&data).is_none());
    }

    #[test]
    fn test_elf_header_is_executable() {
        let data = minimal_elf64_exec();
        let hdr = ElfHeader64::parse(&data).unwrap();
        assert!(hdr.is_executable());
    }

    #[test]
    fn test_elf_header_machine_name() {
        let data = minimal_elf64_exec();
        let hdr = ElfHeader64::parse(&data).unwrap();
        assert_eq!(hdr.machine_name(), "x86_64");
    }

    #[test]
    fn test_elf_program_header_parse() {
        let mut data = vec![0u8; 56];
        data[0..4].copy_from_slice(&1u32.to_le_bytes()); // PT_LOAD
        data[4..8].copy_from_slice(&5u32.to_le_bytes()); // PF_R | PF_X
        let ph = ElfProgramHeader64::parse(&data);
        assert!(ph.is_some());
        let ph = ph.unwrap();
        assert_eq!(ph.p_type, ElfProgramHeader64::PT_LOAD);
        assert!(ph.is_readable());
        assert!(ph.is_executable());
        assert!(!ph.is_writable());
    }

    #[test]
    fn test_elf_section_header_parse() {
        let data = vec![0u8; 64];
        let sh = ElfSectionHeader64::parse(&data);
        assert!(sh.is_some());
        assert_eq!(sh.unwrap().sh_type, ElfSectionHeader64::SHT_NULL);
    }

    #[test]
    fn test_elf_dynamic_entry_parse() {
        let mut data = vec![0u8; 16];
        data[0..8].copy_from_slice(&1i64.to_le_bytes()); // DT_NEEDED
        data[8..16].copy_from_slice(&42u64.to_le_bytes());
        let entry = ElfDynamicEntry64::parse(&data).unwrap();
        assert_eq!(entry.d_tag, ElfDynamicEntry64::DT_NEEDED);
        assert_eq!(entry.d_val, 42);
    }

    #[test]
    fn test_elf_symbol_parse() {
        let mut data = vec![0u8; 24];
        data[4] = (1 << 4) | 2; // STB_GLOBAL | STT_FUNC
        let sym = ElfSymbol64::parse(&data).unwrap();
        assert_eq!(sym.bind(), ElfSymbol64::STB_GLOBAL);
        assert_eq!(sym.type_(), ElfSymbol64::STT_FUNC);
        assert!(sym.is_global());
    }

    #[test]
    fn test_elf_rela_parse() {
        let mut data = vec![0u8; 24];
        let r_info: u64 = (3u64 << 32) | ElfRela64::R_X86_64_64 as u64;
        data[8..16].copy_from_slice(&r_info.to_le_bytes());
        let rela = ElfRela64::parse(&data).unwrap();
        assert_eq!(rela.sym(), 3);
        assert_eq!(rela.type_(), ElfRela64::R_X86_64_64);
    }

    #[test]
    fn test_elf_string_table_get() {
        let data = b"\x00hello\x00world\x00";
        assert_eq!(ElfStringTable::get(data, 1), "hello");
        assert_eq!(ElfStringTable::get(data, 7), "world");
        assert_eq!(ElfStringTable::get(data, 0), "");
    }

    #[test]
    fn test_linux_syscall_table_lookup() {
        assert_eq!(LinuxSyscallTable::lookup(0), "read");
        assert_eq!(LinuxSyscallTable::lookup(1), "write");
        assert_eq!(LinuxSyscallTable::lookup(59), "execve");
        assert_eq!(LinuxSyscallTable::lookup(60), "exit");
        assert_eq!(LinuxSyscallTable::lookup(353), "set_mempolicy_home_node");
        assert_eq!(LinuxSyscallTable::lookup(9999), "unknown");
    }

    #[test]
    fn test_linux_syscall_emulator_read() {
        // syscall 0 (read): arg3 = count
        let ret = LinuxSyscallEmulator::syscall(0, 3, 0x1000, 64, 0, 0, 0);
        assert_eq!(ret, 64);
    }

    #[test]
    fn test_linux_syscall_emulator_write() {
        let ret = LinuxSyscallEmulator::syscall(1, 1, 0x2000, 128, 0, 0, 0);
        assert_eq!(ret, 128);
    }

    #[test]
    fn test_linux_syscall_emulator_getpid() {
        let ret = LinuxSyscallEmulator::syscall(39, 0, 0, 0, 0, 0, 0);
        assert_eq!(ret, 0xABCD);
    }

    #[test]
    fn test_linux_process_new() {
        let proc = LinuxProcess::new(42);
        assert_eq!(proc.pid, 42);
        assert_eq!(proc.ppid, 1);
        assert_eq!(proc.uid, 0);
        assert!(proc.image.is_none());
        assert_eq!(proc.get_env("USER").as_deref(), Some("root"));
    }

    #[test]
    fn test_linux_process_set_env() {
        let mut proc = LinuxProcess::new(1);
        proc.set_env("MYVAR", "hello");
        assert_eq!(proc.get_env("MYVAR").as_deref(), Some("hello"));
    }

    #[test]
    fn test_linux_process_load_elf_invalid() {
        let mut proc = LinuxProcess::new(1);
        let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert!(!proc.load_elf(&garbage));
        assert!(proc.image.is_none());
    }

    #[test]
    fn test_elf_loader_load_and_map_invalid() {
        let data = vec![0u8; 32];
        assert!(ElfLoader::load_and_map(&data, 0x400000).is_none());
    }
}
