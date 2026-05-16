// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Proses/Thread yöneticisi ve süreç bellek entegrasyonu.
// Dosya Yolu         : compat/win32/process_mgr.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Bu dosya PE imaj eşleme, import çözümleme ve süreç bazlı sanal bellek
//   kullanımını bir araya getiren Win32 süreç yöneticisini sağlar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/win32_process_types.rs
//   3-) compat/win32/win32_virtual_memory.rs
//   4-) compat/win32/pe_loader.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// 2026-05-16      Süreç bellek yöneticisi ortak modüllere göre sadeleştirildi
// *******************************************************************

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::dos_emulator::console_writeln;
use crate::win32::pe_loader::{ImageSectionHeader, PeImport, PeParser, PeParserResult};
use crate::win32::win32_base_types::{WinProtection, WIN_PAGE_EXECUTE_READ, WIN_PAGE_READONLY, WIN_PAGE_READWRITE};
use crate::win32::win32_virtual_memory::WinVirtualMemoryManager;

#[derive(Debug, Clone)]
pub struct MappedImage {
    pub base_address: u64,
    pub image_size: u64,
    pub sections: Vec<(String, u64, u64, u32)>,
    pub imports_resolved: Vec<(String, String, u64)>,
    pub relocs_applied: u32,
    pub memory: Vec<u8>,
}

impl MappedImage {
    pub fn new(base: u64, size: u64) -> Self {
        Self {
            base_address: base,
            image_size: size,
            sections: Vec::new(),
            imports_resolved: Vec::new(),
            relocs_applied: 0,
            memory: vec![0u8; size as usize],
        }
    }

    pub fn read_u64(&self, addr: u64) -> u64 {
        let off = (addr.saturating_sub(self.base_address)) as usize;
        if off + 8 <= self.memory.len() {
            u64::from_le_bytes([
                self.memory[off],
                self.memory[off + 1],
                self.memory[off + 2],
                self.memory[off + 3],
                self.memory[off + 4],
                self.memory[off + 5],
                self.memory[off + 6],
                self.memory[off + 7],
            ])
        } else {
            0
        }
    }

    pub fn write_u64(&mut self, addr: u64, value: u64) {
        let off = (addr.saturating_sub(self.base_address)) as usize;
        if off + 8 <= self.memory.len() {
            self.memory[off..off + 8].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn write_u32(&mut self, addr: u64, value: u32) {
        let off = (addr.saturating_sub(self.base_address)) as usize;
        if off + 4 <= self.memory.len() {
            self.memory[off..off + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}

pub struct PeLoader;

impl PeLoader {
    pub fn map_image(data: &[u8], pe: &PeParserResult, memory: &mut WinVirtualMemoryManager) -> Option<MappedImage> {
        let base = memory
            .reserve_and_commit(
                pe.optional_header.size_of_image as usize,
                WinProtection::new(WIN_PAGE_READWRITE),
            )
            .ok()?;

        let mut mapped = MappedImage::new(base, pe.optional_header.size_of_image as u64);
        for sec in &pe.sections {
            let name = String::from(sec.name_as_str());
            let va = base + sec.virtual_address as u64;
            let size = sec.virtual_size as u64;
            let prot = if sec.is_executable() {
                WIN_PAGE_EXECUTE_READ
            } else if sec.is_writable() {
                WIN_PAGE_READWRITE
            } else {
                WIN_PAGE_READONLY
            };
            mapped.sections.push((name, va, size, prot));
            let raw_off = sec.pointer_to_raw_data as usize;
            let raw_size = sec.size_of_raw_data as usize;
            let mem_off = sec.virtual_address as usize;
            if raw_off + raw_size <= data.len() && mem_off + raw_size <= mapped.memory.len() {
                mapped.memory[mem_off..mem_off + raw_size]
                    .copy_from_slice(&data[raw_off..raw_off + raw_size]);
            }
        }

        console_writeln(&format!(
            "[WIN32] mapped image with {} sections @ 0x{:016X}",
            mapped.sections.len(),
            base
        ));
        Some(mapped)
    }

    pub fn apply_relocations(mapped: &mut MappedImage, pe: &PeParserResult, new_base: u64) {
        let old_base = pe.optional_header.image_base;
        if old_base == new_base {
            return;
        }

        let delta = new_base.wrapping_sub(old_base) as i64;
        for reloc in &pe.relocs {
            let page_rva = reloc.virtual_address as u64;
            for type_offset in &reloc.type_offsets {
                let reloc_type = type_offset >> 12;
                let offset = (type_offset & 0x0FFF) as u64;
                let addr = mapped.base_address + page_rva + offset;
                match reloc_type {
                    3 => {
                        let old = mapped.read_u64(addr) as u32;
                        let new = (old as i64 + delta) as u32;
                        mapped.write_u32(addr, new);
                        mapped.relocs_applied += 1;
                    }
                    10 => {
                        let old = mapped.read_u64(addr);
                        let new = (old as i64 + delta) as u64;
                        mapped.write_u64(addr, new);
                        mapped.relocs_applied += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn resolve_imports(mapped: &mut MappedImage, pe: &PeParserResult) {
        for import in &pe.imports {
            let addr = 0x7FFF_0000_0000 + mapped.imports_resolved.len() as u64;
            mapped.imports_resolved.push((
                String::from(&import.dll_name),
                String::from(&import.func_name),
                addr,
            ));
        }
        console_writeln(&format!(
            "[WIN32] resolved {} imports",
            mapped.imports_resolved.len()
        ));
    }

    pub fn resolve_import_address(mapped: &MappedImage, dll: &str, func: &str) -> Option<u64> {
        mapped
            .imports_resolved
            .iter()
            .find(|(dll_name, func_name, _)| dll_name == dll && func_name == func)
            .map(|(_, _, addr)| *addr)
    }

    pub fn get_proc_address(mapped: &MappedImage, export_rva: u32) -> u64 {
        mapped.base_address + export_rva as u64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageDelayImportDescriptor {
    pub attributes: u32,
    pub name_rva: u32,
    pub module_handle_rva: u32,
    pub delay_iat_rva: u32,
    pub delay_int_rva: u32,
    pub bound_delay_iat_rva: u32,
    pub unload_delay_iat_rva: u32,
    pub time_stamp: u32,
}

impl ImageDelayImportDescriptor {
    pub const fn new() -> Self {
        Self {
            attributes: 0,
            name_rva: 0,
            module_handle_rva: 0,
            delay_iat_rva: 0,
            delay_int_rva: 0,
            bound_delay_iat_rva: 0,
            unload_delay_iat_rva: 0,
            time_stamp: 0,
        }
    }

    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 32 {
            return None;
        }
        Some(Self {
            attributes: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            name_rva: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            module_handle_rva: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            delay_iat_rva: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            delay_int_rva: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            bound_delay_iat_rva: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            unload_delay_iat_rva: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            time_stamp: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
        })
    }
}

pub struct DelayImportParser;

impl DelayImportParser {
    pub fn parse(data: &[u8], sections: &[ImageSectionHeader], rva: u32, _size: u32) -> Vec<PeImport> {
        let mut imports = Vec::new();
        let mut offset = match PeParser::rva_to_offset(sections, rva) {
            Some(value) => value,
            None => return imports,
        };

        loop {
            if data.len() < offset + 32 {
                break;
            }
            let descriptor = match ImageDelayImportDescriptor::parse(&data[offset..]) {
                Some(value) => value,
                None => break,
            };
            if descriptor.name_rva == 0 {
                break;
            }

            let name_off = match PeParser::rva_to_offset(sections, descriptor.name_rva) {
                Some(value) => value,
                None => {
                    offset += 32;
                    continue;
                }
            };
            let dll_name = PeParser::read_ascii(data, name_off);
            let int_off = match PeParser::rva_to_offset(sections, descriptor.delay_int_rva) {
                Some(value) => value,
                None => {
                    offset += 32;
                    continue;
                }
            };

            let mut thunk_off = int_off;
            loop {
                if data.len() < thunk_off + 8 {
                    break;
                }
                let thunk = u64::from_le_bytes([
                    data[thunk_off],
                    data[thunk_off + 1],
                    data[thunk_off + 2],
                    data[thunk_off + 3],
                    data[thunk_off + 4],
                    data[thunk_off + 5],
                    data[thunk_off + 6],
                    data[thunk_off + 7],
                ]);
                if thunk == 0 {
                    break;
                }

                if (thunk & 0x8000_0000_0000_0000) == 0 {
                    let hint_off = match PeParser::rva_to_offset(sections, (thunk & 0xFFFF_FFFF) as u32) {
                        Some(value) => value,
                        None => {
                            thunk_off += 8;
                            continue;
                        }
                    };
                    if data.len() >= hint_off + 2 {
                        let hint = u16::from_le_bytes([data[hint_off], data[hint_off + 1]]);
                        let func_name = PeParser::read_ascii(data, hint_off + 2);
                        imports.push(PeImport {
                            dll_name: String::from(&dll_name),
                            func_name,
                            ordinal: 0,
                            hint,
                        });
                    }
                } else {
                    imports.push(PeImport {
                        dll_name: String::from(&dll_name),
                        func_name: String::new(),
                        ordinal: (thunk & 0xFFFF) as u16,
                        hint: 0,
                    });
                }

                thunk_off += 8;
            }

            offset += 32;
        }

        imports
    }
}
