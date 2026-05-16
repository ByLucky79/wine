// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : PE Kaynak, TLS, Debug Ayrıştırıcı
// Dosya Yolu         : apps/system/compat/win32/pe_resources.rs
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
//   PE Resource, TLS, Debug, Exception, Load Config, Delay Import, Bound Import ayrıştırıcılar.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

use alloc::vec::Vec;
use super::pe_loader::{ImageSectionHeader, PeParser};

// ─── PE Resource Directory Parser ──────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ImageResourceDirectory {
    pub characteristics: u32, pub time_date_stamp: u32, pub major_version: u16,
    pub minor_version: u16, pub number_of_named_entries: u16, pub number_of_id_entries: u16,
}

impl Default for ImageResourceDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageResourceDirectory {
    pub const fn new() -> Self { Self { characteristics: 0, time_date_stamp: 0, major_version: 0, minor_version: 0, number_of_named_entries: 0, number_of_id_entries: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { return None; }
        Some(Self { characteristics: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), time_date_stamp: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), major_version: u16::from_le_bytes([data[8], data[9]]), minor_version: u16::from_le_bytes([data[10], data[11]]), number_of_named_entries: u16::from_le_bytes([data[12], data[13]]), number_of_id_entries: u16::from_le_bytes([data[14], data[15]]) })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageResourceDirectoryEntry {
    pub name_offset: u32, pub id: u32, pub offset_to_data: u32,
    pub data_is_directory: bool, pub data_rva: u32,
}

impl Default for ImageResourceDirectoryEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageResourceDirectoryEntry {
    pub const fn new() -> Self { Self { name_offset: 0, id: 0, offset_to_data: 0, data_is_directory: false, data_rva: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        let name_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let offset_to_data = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let data_is_directory = (offset_to_data & 0x80000000) != 0;
        let data_rva = offset_to_data & 0x7FFFFFFF;
        Some(Self { name_offset, id: name_offset & 0xFFFF, offset_to_data, data_is_directory, data_rva })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageResourceDataEntry {
    pub offset_to_data: u32, pub size: u32, pub code_page: u32, pub reserved: u32,
}

impl Default for ImageResourceDataEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageResourceDataEntry {
    pub const fn new() -> Self { Self { offset_to_data: 0, size: 0, code_page: 0, reserved: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 16 { return None; }
        Some(Self { offset_to_data: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), size: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), code_page: u32::from_le_bytes([data[8], data[9], data[10], data[11]]), reserved: u32::from_le_bytes([data[12], data[13], data[14], data[15]]) })
    }
}

#[derive(Debug, Clone)]
pub struct PeResource {
    pub type_id: u32, pub name_id: u32, pub lang_id: u32,
    pub data_rva: u32, pub size: u32, pub code_page: u32,
}

pub struct ResourceParser;

impl ResourceParser {
    pub fn parse(data: &[u8], sections: &[ImageSectionHeader], rva: u32, size: u32) -> Vec<PeResource> {
        let mut resources = Vec::new();
        let offset = match PeParser::rva_to_offset(sections, rva) { Some(o) => o, None => return resources };
        if data.len() < offset + size as usize { return resources; }
        Self::parse_directory(data, sections, offset, offset, 0, 0, 0, &mut resources);
        resources
    }
    fn parse_directory(data: &[u8], sections: &[ImageSectionHeader], base: usize, dir_offset: usize, type_id: u32, name_id: u32, lang_id: u32, out: &mut Vec<PeResource>) {
        if data.len() < dir_offset + 16 { return; }
        let dir = match ImageResourceDirectory::parse(&data[dir_offset..]) { Some(d) => d, None => return };
        let entry_count = dir.number_of_named_entries as usize + dir.number_of_id_entries as usize;
        let entry_offset = dir_offset + 16;
        for i in 0..entry_count {
            let off = entry_offset + i * 8;
            if data.len() < off + 8 { break; }
            if let Some(entry) = ImageResourceDirectoryEntry::parse(&data[off..]) {
                let current_id = entry.id;
                if entry.data_is_directory {
                    let child_offset = base + entry.data_rva as usize;
                    if type_id == 0 { Self::parse_directory(data, sections, base, child_offset, current_id, name_id, lang_id, out); }
                    else if name_id == 0 { Self::parse_directory(data, sections, base, child_offset, type_id, current_id, lang_id, out); }
                    else { Self::parse_directory(data, sections, base, child_offset, type_id, name_id, current_id, out); }
                } else {
                    let data_offset = base + entry.data_rva as usize;
                    if data.len() >= data_offset + 16 { if let Some(data_entry) = ImageResourceDataEntry::parse(&data[data_offset..]) { out.push(PeResource { type_id, name_id, lang_id, data_rva: data_entry.offset_to_data, size: data_entry.size, code_page: data_entry.code_page }); } }
                }
            }
        }
    }
}

// ─── PE TLS Directory Parser ───────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ImageTlsDirectory64 {
    pub start_address_of_raw_data: u64, pub end_address_of_raw_data: u64,
    pub address_of_index: u64, pub address_of_callbacks: u64,
    pub size_of_zero_fill: u32, pub characteristics: u32,
}

impl Default for ImageTlsDirectory64 {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageTlsDirectory64 {
    pub const fn new() -> Self { Self { start_address_of_raw_data: 0, end_address_of_raw_data: 0, address_of_index: 0, address_of_callbacks: 0, size_of_zero_fill: 0, characteristics: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 40 { return None; }
        Some(Self { start_address_of_raw_data: u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]), end_address_of_raw_data: u64::from_le_bytes([data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15]]), address_of_index: u64::from_le_bytes([data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23]]), address_of_callbacks: u64::from_le_bytes([data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31]]), size_of_zero_fill: u32::from_le_bytes([data[32], data[33], data[34], data[35]]), characteristics: u32::from_le_bytes([data[36], data[37], data[38], data[39]]) })
    }
}

// ─── PE Debug Directory Parser ─────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ImageDebugDirectory {
    pub characteristics: u32, pub time_date_stamp: u32, pub major_version: u16,
    pub minor_version: u16, pub type_: u32, pub size_of_data: u32,
    pub address_of_raw_data: u32, pub pointer_to_raw_data: u32,
}

impl Default for ImageDebugDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDebugDirectory {
    pub const fn new() -> Self { Self { characteristics: 0, time_date_stamp: 0, major_version: 0, minor_version: 0, type_: 0, size_of_data: 0, address_of_raw_data: 0, pointer_to_raw_data: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 28 { return None; }
        Some(Self { characteristics: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), time_date_stamp: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), major_version: u16::from_le_bytes([data[8], data[9]]), minor_version: u16::from_le_bytes([data[10], data[11]]), type_: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), size_of_data: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), address_of_raw_data: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), pointer_to_raw_data: u32::from_le_bytes([data[24], data[25], data[26], data[27]]) })
    }
    pub fn type_name(&self) -> &'static str {
        match self.type_ {
            1 => "COFF", 2 => "CODEVIEW", 3 => "FPO", 4 => "MISC", 5 => "EXCEPTION",
            6 => "FIXUP", 7 => "OMAP_TO_SRC", 8 => "OMAP_FROM_SRC", 9 => "BORLAND", 10 => "RESERVED10",
            11 => "CLSID", 12 => "VC_FEATURE", 13 => "POGO", 14 => "ILTCG", 15 => "MPX",
            16 => "REPRO", 20 => "EX_DLLCHARACTERISTICS", _ => "UNKNOWN",
        }
    }
}

