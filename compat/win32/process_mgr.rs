// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Proses/Thread Yöneticisi
// Dosya Yolu         : apps/system/compat/win32/process_mgr.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64,
//   Alpha, VAX, HPPA, SH-4, IA-64
//
// Açıklama:
//   Win32 proses bellek yöneticisi ve syscall emülatörü.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

use crate::dos_emulator::console_writeln;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use super::pe_loader::{PeParser, PeParserResult, ImageSectionHeader, PeImport};

// ─── PE Loader (In-Memory Mapping) ─────────────────────────────

#[derive(Debug, Clone)]
pub struct MappedImage {
    pub base_address: u64, pub image_size: u64,
    pub sections: Vec<(String, u64, u64, u32)>,
    pub imports_resolved: Vec<(String, String, u64)>,
    pub relocs_applied: u32,
    pub memory: Vec<u8>,
}

impl MappedImage {
    pub fn new(base: u64, size: u64) -> Self { Self { base_address: base, image_size: size, sections: Vec::new(), imports_resolved: Vec::new(), relocs_applied: 0, memory: vec![0u8; size as usize] } }
    pub fn read_u64(&self, addr: u64) -> u64 {
        let off = (addr - self.base_address) as usize;
        if off + 8 <= self.memory.len() { u64::from_le_bytes([self.memory[off], self.memory[off+1], self.memory[off+2], self.memory[off+3], self.memory[off+4], self.memory[off+5], self.memory[off+6], self.memory[off+7]]) } else { 0 }
    }
    pub fn write_u64(&mut self, addr: u64, val: u64) {
        let off = (addr - self.base_address) as usize;
        if off + 8 <= self.memory.len() { let b = val.to_le_bytes(); self.memory[off..off+8].copy_from_slice(&b); }
    }
    pub fn write_u32(&mut self, addr: u64, val: u32) {
        let off = (addr - self.base_address) as usize;
        if off + 4 <= self.memory.len() { let b = val.to_le_bytes(); self.memory[off..off+4].copy_from_slice(&b); }
    }
    pub fn write_u16(&mut self, addr: u64, val: u16) {
        let off = (addr - self.base_address) as usize;
        if off + 2 <= self.memory.len() { let b = val.to_le_bytes(); self.memory[off..off+2].copy_from_slice(&b); }
    }
    pub fn get_entry_point(&self, rva: u32) -> u64 { self.base_address + rva as u64 }
    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) -> usize {
        let off = (addr.saturating_sub(self.base_address)) as usize;
        let len = data.len().min(self.memory.len().saturating_sub(off));
        self.memory[off..off+len].copy_from_slice(&data[..len]);
        len
    }
    pub fn read_bytes(&self, addr: u64, out: &mut [u8]) -> usize {
        let off = (addr.saturating_sub(self.base_address)) as usize;
        let len = out.len().min(self.memory.len().saturating_sub(off));
        out[..len].copy_from_slice(&self.memory[off..off+len]);
        len
    }
}

pub struct PeLoader;

impl PeLoader {
    pub fn map_image(data: &[u8], pe: &PeParserResult, base: u64) -> MappedImage {
        let mut mapped = MappedImage::new(base, pe.optional_header.size_of_image as u64);
        for sec in &pe.sections {
            let name = String::from(sec.name_as_str());
            let va = base + sec.virtual_address as u64;
            let size = sec.virtual_size as u64;
            let prot = if sec.is_executable() { 0x40 } else if sec.is_writable() { 0x04 } else { 0x02 };
            mapped.sections.push((name.clone(), va, size, prot));
            let raw_off = sec.pointer_to_raw_data as usize;
            let raw_size = sec.size_of_raw_data as usize;
            let mem_off = sec.virtual_address as usize;
            if raw_off + raw_size <= data.len() && mem_off + raw_size <= mapped.memory.len() {
                mapped.memory[mem_off..mem_off + raw_size].copy_from_slice(&data[raw_off..raw_off + raw_size]);
            }
        }
        let msg = format!("{}: {} sections @ base=0x{:016X}", "[WIN32]", mapped.sections.len(), base);
        console_writeln(&msg); mapped
    }
    pub fn apply_relocations(mapped: &mut MappedImage, pe: &PeParserResult, new_base: u64) {
        let old_base = pe.optional_header.image_base;
        if old_base == new_base { return; }
        let delta = new_base.wrapping_sub(old_base) as i64;
        for reloc in &pe.relocs {
            let page_rva = reloc.virtual_address as u64;
            for type_offset in &reloc.type_offsets {
                let type_ = type_offset >> 12;
                let offset = (type_offset & 0xFFF) as u64;
                let addr = mapped.base_address + page_rva + offset;
                match type_ {
                    3 => { // IMAGE_REL_BASED_HIGHLOW
                        let old = mapped.read_u64(addr) as u32;
                        let new = (old as i64 + delta) as u32;
                        mapped.write_u32(addr, new); mapped.relocs_applied += 1;
                    }
                    10 => { // IMAGE_REL_BASED_DIR64
                        let old = mapped.read_u64(addr);
                        let new = (old as i64 + delta) as u64;
                        mapped.write_u64(addr, new); mapped.relocs_applied += 1;
                    }
                    1 => { // HIGH
                        let old = mapped.read_u64(addr) as u32;
                        let new = ((old as i64 + delta) >> 16) as u16 as u32;
                        mapped.write_u32(addr, (old & 0xFFFF) | (new << 16)); mapped.relocs_applied += 1;
                    }
                    2 => { // LOW
                        let old = mapped.read_u64(addr) as u32;
                        let new = ((old as i64 + delta) & 0xFFFF) as u16 as u32;
                        mapped.write_u32(addr, (old & 0xFFFF0000) | new); mapped.relocs_applied += 1;
                    }
                    0 => {},
                    _ => {}
                }
            }
        }
        let msg = format!("{}: {} (delta=0x{:016X})", "[WIN32]", mapped.relocs_applied, delta);
        console_writeln(&msg);
    }
    pub fn resolve_imports(mapped: &mut MappedImage, pe: &PeParserResult) {
        for imp in &pe.imports {
            let addr = 0x7FFF_0000_0000 + mapped.imports_resolved.len() as u64;
            mapped.imports_resolved.push((String::from(&imp.dll_name), String::from(&imp.func_name), addr));
        }
        let msg = format!("{}: {}", "[WIN32]", mapped.imports_resolved.len());
        console_writeln(&msg);
    }
    pub fn resolve_import_address(mapped: &MappedImage, dll: &str, func: &str) -> Option<u64> {
        mapped.imports_resolved.iter().find(|(d, f, _)| d == dll && f == func).map(|(_, _, addr)| *addr)
    }
    pub fn get_proc_address(mapped: &MappedImage, export_rva: u32) -> u64 {
        mapped.base_address + export_rva as u64
    }
}

// ─── Process Memory Manager ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: u64, pub size: u64, pub prot: u32, pub committed: bool, pub reserved: bool,
}

pub struct ProcessMemoryManager {
    pub regions: Vec<MemoryRegion>, pub next_alloc: u64,
}

impl Default for ProcessMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessMemoryManager {
    pub fn new() -> Self { Self { regions: Vec::new(), next_alloc: 0x1000_0000 } }
    pub fn allocate(&mut self, size: u64, prot: u32) -> u64 {
        let addr = self.next_alloc;
        self.next_alloc += size;
        self.regions.push(MemoryRegion { base: addr, size, prot, committed: true, reserved: true });
        let msg = format!("{}: 0x{:016X} (size=0x{:08X}, prot=0x{:08X})", "[WIN32]", addr, size, prot);
        console_writeln(&msg); addr
    }
    pub fn free(&mut self, base: u64) -> bool {
        if let Some(idx) = self.regions.iter().position(|r| r.base == base) {
            self.regions.remove(idx);
            let msg = format!("{}: 0x{:016X}", "[WIN32]", base);
            console_writeln(&msg); true
        } else { false }
    }
    pub fn protect(&mut self, base: u64, new_prot: u32) -> bool {
        if let Some(r) = self.regions.iter_mut().find(|r| r.base == base) { r.prot = new_prot; true } else { false }
    }
    pub fn query(&self, base: u64) -> Option<&MemoryRegion> { self.regions.iter().find(|r| r.base == base) }
}

// ─── User32 / Kernel32 Syscall Emulator ────────────────────────

// ─── PE Delay Import Directory Parser ──────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ImageDelayImportDescriptor {
    pub attributes: u32, pub name_rva: u32, pub module_handle_rva: u32,
    pub delay_iat_rva: u32, pub delay_int_rva: u32, pub bound_delay_iat_rva: u32,
    pub unload_delay_iat_rva: u32, pub time_stamp: u32,
}

impl Default for ImageDelayImportDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDelayImportDescriptor {
    pub const fn new() -> Self { Self { attributes: 0, name_rva: 0, module_handle_rva: 0, delay_iat_rva: 0, delay_int_rva: 0, bound_delay_iat_rva: 0, unload_delay_iat_rva: 0, time_stamp: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 32 { return None; }
        Some(Self { attributes: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), name_rva: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), module_handle_rva: u32::from_le_bytes([data[8], data[9], data[10], data[11]]), delay_iat_rva: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), delay_int_rva: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), bound_delay_iat_rva: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), unload_delay_iat_rva: u32::from_le_bytes([data[24], data[25], data[26], data[27]]), time_stamp: u32::from_le_bytes([data[28], data[29], data[30], data[31]]) })
    }
}

pub struct DelayImportParser;

impl DelayImportParser {
    pub fn parse(data: &[u8], sections: &[ImageSectionHeader], rva: u32, _size: u32) -> Vec<PeImport> {
        let mut imports = Vec::new();
        let mut offset = match PeParser::rva_to_offset(sections, rva) { Some(o) => o, None => return imports };
        loop {
            if data.len() < offset + 32 { break; }
            let desc = match ImageDelayImportDescriptor::parse(&data[offset..]) { Some(d) => d, None => break };
            if desc.name_rva == 0 { break; }
            let name_off = match PeParser::rva_to_offset(sections, desc.name_rva) { Some(o) => o, None => { offset += 32; continue; } };
            let dll_name = PeParser::read_ascii(data, name_off);
            let int_off = match PeParser::rva_to_offset(sections, desc.delay_int_rva) { Some(o) => o, None => { offset += 32; continue; } };
            let mut thunk_off = int_off;
            loop {
                if data.len() < thunk_off + 8 { break; }
                let thunk = u64::from_le_bytes([data[thunk_off], data[thunk_off+1], data[thunk_off+2], data[thunk_off+3], data[thunk_off+4], data[thunk_off+5], data[thunk_off+6], data[thunk_off+7]]);
                if thunk == 0 { break; }
                if (thunk & 0x8000_0000_0000_0000) == 0 {
                    let hint_off = match PeParser::rva_to_offset(sections, (thunk & 0xFFFFFFFF) as u32) { Some(o) => o, None => { thunk_off += 8; continue; } };
                    if data.len() >= hint_off + 2 { let hint = u16::from_le_bytes([data[hint_off], data[hint_off+1]]); let func_name = PeParser::read_ascii(data, hint_off + 2); imports.push(PeImport { dll_name: String::from(&dll_name), func_name, ordinal: 0, hint }); }
                } else { let ordinal = (thunk & 0xFFFF) as u16; imports.push(PeImport { dll_name: String::from(&dll_name), func_name: String::from(""), ordinal, hint: 0 }); }
                thunk_off += 8;
            }
            offset += 32;
        }
        imports
    }
}

// ─── PE Load Config Directory Parser ───────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ImageLoadConfigDirectory64 {
    pub size: u32, pub time_date_stamp: u32, pub major_version: u16, pub minor_version: u16,
    pub global_flags_clear: u32, pub global_flags_set: u32, pub critical_section_default_timeout: u32,
    pub decommit_free_block_threshold: u64, pub decommit_total_free_threshold: u64,
    pub lock_prefix_table: u64, pub maximum_allocation_size: u64, pub virtual_memory_threshold: u64,
    pub process_affinity_mask: u64, pub process_heap_flags: u32, pub csd_version: u16,
    pub reserved1: u16, pub edit_list: u64, pub security_cookie: u64,
    pub se_handler_table: u64, pub se_handler_count: u64,
    pub guard_cf_check_function_pointer: u64, pub guard_cf_dispatch_function_pointer: u64,
    pub guard_cf_function_table: u64, pub guard_cf_function_count: u64, pub guard_flags: u32,
}

impl Default for ImageLoadConfigDirectory64 {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageLoadConfigDirectory64 {
    pub const fn new() -> Self { Self { size: 0, time_date_stamp: 0, major_version: 0, minor_version: 0, global_flags_clear: 0, global_flags_set: 0, critical_section_default_timeout: 0, decommit_free_block_threshold: 0, decommit_total_free_threshold: 0, lock_prefix_table: 0, maximum_allocation_size: 0, virtual_memory_threshold: 0, process_affinity_mask: 0, process_heap_flags: 0, csd_version: 0, reserved1: 0, edit_list: 0, security_cookie: 0, se_handler_table: 0, se_handler_count: 0, guard_cf_check_function_pointer: 0, guard_cf_dispatch_function_pointer: 0, guard_cf_function_table: 0, guard_cf_function_count: 0, guard_flags: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 148 { return None; }
        Some(Self { size: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), time_date_stamp: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), major_version: u16::from_le_bytes([data[8], data[9]]), minor_version: u16::from_le_bytes([data[10], data[11]]), global_flags_clear: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), global_flags_set: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), critical_section_default_timeout: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), decommit_free_block_threshold: u64::from_le_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]), decommit_total_free_threshold: u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]), lock_prefix_table: u64::from_le_bytes([data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47]]), maximum_allocation_size: u64::from_le_bytes([data[48], data[49], data[50], data[51], data[52], data[53], data[54], data[55]]), virtual_memory_threshold: u64::from_le_bytes([data[56], data[57], data[58], data[59], data[60], data[61], data[62], data[63]]), process_affinity_mask: u64::from_le_bytes([data[64], data[65], data[66], data[67], data[68], data[69], data[70], data[71]]), process_heap_flags: u32::from_le_bytes([data[72], data[73], data[74], data[75]]), csd_version: u16::from_le_bytes([data[76], data[77]]), reserved1: u16::from_le_bytes([data[78], data[79]]), edit_list: u64::from_le_bytes([data[80], data[81], data[82], data[83], data[84], data[85], data[86], data[87]]), security_cookie: u64::from_le_bytes([data[88], data[89], data[90], data[91], data[92], data[93], data[94], data[95]]), se_handler_table: u64::from_le_bytes([data[96], data[97], data[98], data[99], data[100], data[101], data[102], data[103]]), se_handler_count: u64::from_le_bytes([data[104], data[105], data[106], data[107], data[108], data[109], data[110], data[111]]), guard_cf_check_function_pointer: u64::from_le_bytes([data[112], data[113], data[114], data[115], data[116], data[117], data[118], data[119]]), guard_cf_dispatch_function_pointer: u64::from_le_bytes([data[120], data[121], data[122], data[123], data[124], data[125], data[126], data[127]]), guard_cf_function_table: u64::from_le_bytes([data[128], data[129], data[130], data[131], data[132], data[133], data[134], data[135]]), guard_cf_function_count: u64::from_le_bytes([data[136], data[137], data[138], data[139], data[140], data[141], data[142], data[143]]), guard_flags: u32::from_le_bytes([data[144], data[145], data[146], data[147]]) })
    }
}

// ─── PE Exception Directory Parser (x64 Unwind) ────────────────

#[derive(Debug, Clone, Copy)]
pub struct ImageRuntimeFunctionEntry {
    pub begin_address: u32, pub end_address: u32, pub unwind_data: u32,
}

impl Default for ImageRuntimeFunctionEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageRuntimeFunctionEntry {
    pub const fn new() -> Self { Self { begin_address: 0, end_address: 0, unwind_data: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 12 { return None; }
        Some(Self { begin_address: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), end_address: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), unwind_data: u32::from_le_bytes([data[8], data[9], data[10], data[11]]) })
    }
}

pub struct ExceptionDirectoryParser;

impl ExceptionDirectoryParser {
    pub fn parse(data: &[u8], sections: &[ImageSectionHeader], rva: u32, size: u32) -> Vec<ImageRuntimeFunctionEntry> {
        let mut entries = Vec::new();
        let offset = match PeParser::rva_to_offset(sections, rva) { Some(o) => o, None => return entries };
        let count = size as usize / 12;
        for i in 0..count {
            let off = offset + i * 12;
            if data.len() < off + 12 { break; }
            if let Some(entry) = ImageRuntimeFunctionEntry::parse(&data[off..]) { entries.push(entry); }
        }
        entries
    }
}
