// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : PE Ek Yardimci Yapilar - Delay Import, Bound Import, CLI Header
// Dosya Yolu         : apps/system/compat/win32/pe_helpers.rs
// Yazar              : Ozkan Yildirim
// Lisans             : GPLv3
//
// Destekledigi Islemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64,
//   Alpha, VAX, HPPA, SH-4, IA-64
//
// Aciklama:
//   PE Ek Yardimci Yapilar - Delay Import, Bound Import, CLI Header
//
// Bagimli Dosyalar:
//   1-) apps/system/compat/win32/win32_mod.rs
//
//              Dosyaya Mudahaleler
// 2026-05-13      syscall_emu.rs bolundu
// *******************************************************************

extern crate alloc;
use alloc::vec::Vec;
use crate::win32::pe_loader::ImageDataDirectory;

// ─── PE Delay Import Directory Parser ──────────────────────────

#[derive(Debug, Clone)]
pub struct DelayImportDescriptor {
    pub attributes: u32, pub name: u32, pub module_handle: u32, pub delay_iat: u32, pub delay_int: u32, pub bound_delay_iat: u32, pub unload_delay_iat: u32, pub time_stamp: u32,
}

impl Default for DelayImportDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl DelayImportDescriptor {
    pub const fn new() -> Self { Self { attributes: 0, name: 0, module_handle: 0, delay_iat: 0, delay_int: 0, bound_delay_iat: 0, unload_delay_iat: 0, time_stamp: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 32 { return None; }
        Some(Self { attributes: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), name: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), module_handle: u32::from_le_bytes([data[8], data[9], data[10], data[11]]), delay_iat: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), delay_int: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), bound_delay_iat: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), unload_delay_iat: u32::from_le_bytes([data[24], data[25], data[26], data[27]]), time_stamp: u32::from_le_bytes([data[28], data[29], data[30], data[31]]) })
    }
}

pub struct DelayImportDirectoryParser;

impl DelayImportDirectoryParser {
    pub fn parse(data: &[u8], offset: u32, size: u32) -> Vec<DelayImportDescriptor> {
        let mut entries = Vec::new();
        if data.len() < (offset + size) as usize { return entries; }
        let mut off = offset as usize;
        let end = (offset + size) as usize;
        while off + 32 <= end {
            if let Some(entry) = DelayImportDescriptor::parse(&data[off..]) {
                if entry.name == 0 && entry.attributes == 0 { break; }
                entries.push(entry);
            }
            off += 32;
        }
        entries
    }
}

// ─── PE Bound Import Directory Parser ──────────────────────────

#[derive(Debug, Clone)]
pub struct BoundImportDescriptor {
    pub time_stamp: u32, pub offset_module_name: u16, pub number_of_module_forwarder_refs: u16,
}

impl Default for BoundImportDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundImportDescriptor {
    pub const fn new() -> Self { Self { time_stamp: 0, offset_module_name: 0, number_of_module_forwarder_refs: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        Some(Self { time_stamp: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), offset_module_name: u16::from_le_bytes([data[4], data[5]]), number_of_module_forwarder_refs: u16::from_le_bytes([data[6], data[7]]) })
    }
}

pub struct BoundImportDirectoryParser;

impl BoundImportDirectoryParser {
    pub fn parse(data: &[u8], offset: u32, size: u32) -> Vec<BoundImportDescriptor> {
        let mut entries = Vec::new();
        if data.len() < (offset + size) as usize { return entries; }
        let mut off = offset as usize;
        let end = (offset + size) as usize;
        while off + 8 <= end {
            if let Some(entry) = BoundImportDescriptor::parse(&data[off..]) {
                if entry.time_stamp == 0 && entry.offset_module_name == 0 { break; }
                entries.push(entry);
            }
            off += 8;
        }
        entries
    }
}

// ─── PE .NET Metadata (CLI Header) Parser ──────────────────────

#[derive(Debug, Clone, Copy)]
pub struct CliHeader {
    pub cb: u32, pub major_runtime_version: u16, pub minor_runtime_version: u16,
    pub metadata: ImageDataDirectory, pub flags: u32, pub entry_point_token: u32,
    pub resources: ImageDataDirectory, pub strong_name_signature: ImageDataDirectory,
    pub code_manager_table: ImageDataDirectory, pub vtable_fixups: ImageDataDirectory,
    pub export_address_table_jumps: ImageDataDirectory, pub managed_native_header: ImageDataDirectory,
}

impl Default for CliHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl CliHeader {
    pub const fn new() -> Self { Self { cb: 0, major_runtime_version: 0, minor_runtime_version: 0, metadata: ImageDataDirectory { virtual_address: 0, size: 0 }, flags: 0, entry_point_token: 0, resources: ImageDataDirectory { virtual_address: 0, size: 0 }, strong_name_signature: ImageDataDirectory { virtual_address: 0, size: 0 }, code_manager_table: ImageDataDirectory { virtual_address: 0, size: 0 }, vtable_fixups: ImageDataDirectory { virtual_address: 0, size: 0 }, export_address_table_jumps: ImageDataDirectory { virtual_address: 0, size: 0 }, managed_native_header: ImageDataDirectory { virtual_address: 0, size: 0 } } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 72 { return None; }
        Some(Self { cb: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), major_runtime_version: u16::from_le_bytes([data[4], data[5]]), minor_runtime_version: u16::from_le_bytes([data[6], data[7]]), metadata: Self::parse_dir(data, 8), flags: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), entry_point_token: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), resources: Self::parse_dir(data, 24), strong_name_signature: Self::parse_dir(data, 32), code_manager_table: Self::parse_dir(data, 40), vtable_fixups: Self::parse_dir(data, 48), export_address_table_jumps: Self::parse_dir(data, 56), managed_native_header: Self::parse_dir(data, 64) })
    }
    fn parse_dir(data: &[u8], off: usize) -> ImageDataDirectory {
        if data.len() < off + 8 { return ImageDataDirectory { virtual_address: 0, size: 0 }; }
        ImageDataDirectory { virtual_address: u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]), size: u32::from_le_bytes([data[off + 4], data[off + 5], data[off + 6], data[off + 7]]) }
    }
}
