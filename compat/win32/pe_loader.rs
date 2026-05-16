// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : PE Tipler ve Ortak Yapılar
// Dosya Yolu         : apps/system/compat/win32/pe_loader.rs
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
//   PE (Portable Executable) yükleyici ve ortak tip tanımları.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;


extern "C" {
    fn ozkan_wm_create_window(x: i32, y: i32, w: u32, h: u32, title: *const u8) -> i32;
    fn ozkan_wm_draw_all();
    fn ozkan_gfx_fill_rect(x: i32, y: i32, w: i32, h: i32, color: u32);
    fn ozkan_gfx_draw_text(x: i32, y: i32, text: *const u8, len: u32, color: u32);
    fn ozkan_gfx_swap_buffers();
}


// Public wrappers for extern "C" graphics/WM functions
pub unsafe fn gfx_fill_rect(x: i32, y: i32, w: i32, h: i32, color: u32) { ozkan_gfx_fill_rect(x, y, w, h, color); }
pub unsafe fn gfx_draw_text(x: i32, y: i32, text: *const u8, len: u32, color: u32) { ozkan_gfx_draw_text(x, y, text, len, color); }
pub unsafe fn gfx_swap_buffers() { ozkan_gfx_swap_buffers(); }
pub unsafe fn wm_create_window(x: i32, y: i32, w: u32, h: u32, title: *const u8) -> i32 { ozkan_wm_create_window(x, y, w, h, title) }
pub unsafe fn wm_draw_all() { ozkan_wm_draw_all(); }

pub const WIN_W: i32 = 580;
pub const WIN_H: i32 = 420;
pub const CLR_WIN_BG: u32 = 0xFF2D2D30;
pub const CLR_SIDEBAR: u32 = 0xFF3F3F46;
pub const CLR_ITEM_BG: u32 = 0xFF1E1E1E;
pub const CLR_ACCENT: u32 = 0xFF0078D4;
pub const CLR_TEXT: u32 = 0xFFF1F1F1;
pub const CLR_SUCCESS: u32 = 0xFF107C10;
pub const CLR_WARNING: u32 = 0xFFD83B01;

const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_NT_SIGNATURE: u32 = 0x00004550;
const IMAGE_FILE_MACHINE_I386: u16 = 0x014C;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_MACHINE_ARMNT: u16 = 0x01C4;
const IMAGE_FILE_MACHINE_ARM64: u16 = 0xAA64;
const IMAGE_FILE_MACHINE_RISCV32: u16 = 0x5032;
const IMAGE_FILE_MACHINE_RISCV64: u16 = 0x5064;

#[derive(Debug, Clone, Copy)]
pub struct ImageDosHeader {
    pub e_magic: u16, pub e_cblp: u16, pub e_cp: u16, pub e_crlc: u16,
    pub e_cparhdr: u16, pub e_minalloc: u16, pub e_maxalloc: u16,
    pub e_ss: u16, pub e_sp: u16, pub e_csum: u16, pub e_ip: u16,
    pub e_cs: u16, pub e_lfarlc: u16, pub e_ovno: u16,
    pub e_res: [u16; 4], pub e_oemid: u16, pub e_oeminfo: u16,
    pub e_res2: [u16; 10], pub e_lfanew: i32,
}

impl Default for ImageDosHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDosHeader {
    pub const fn new() -> Self {
        Self { e_magic: 0, e_cblp: 0, e_cp: 0, e_crlc: 0, e_cparhdr: 0, e_minalloc: 0, e_maxalloc: 0, e_ss: 0, e_sp: 0, e_csum: 0, e_ip: 0, e_cs: 0, e_lfarlc: 0, e_ovno: 0, e_res: [0; 4], e_oemid: 0, e_oeminfo: 0, e_res2: [0; 10], e_lfanew: 0 }
    }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 64 { return None; }
        let magic = u16::from_le_bytes([data[0], data[1]]);
        if magic != IMAGE_DOS_SIGNATURE { return None; }
        let lfanew = i32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        Some(Self {
            e_magic: magic, e_cblp: u16::from_le_bytes([data[2], data[3]]),
            e_cp: u16::from_le_bytes([data[4], data[5]]), e_crlc: u16::from_le_bytes([data[6], data[7]]),
            e_cparhdr: u16::from_le_bytes([data[8], data[9]]), e_minalloc: u16::from_le_bytes([data[10], data[11]]),
            e_maxalloc: u16::from_le_bytes([data[12], data[13]]), e_ss: u16::from_le_bytes([data[14], data[15]]),
            e_sp: u16::from_le_bytes([data[16], data[17]]), e_csum: u16::from_le_bytes([data[18], data[19]]),
            e_ip: u16::from_le_bytes([data[20], data[21]]), e_cs: u16::from_le_bytes([data[22], data[23]]),
            e_lfarlc: u16::from_le_bytes([data[24], data[25]]), e_ovno: u16::from_le_bytes([data[26], data[27]]),
            e_res: [u16::from_le_bytes([data[28], data[29]]), u16::from_le_bytes([data[30], data[31]]), u16::from_le_bytes([data[32], data[33]]), u16::from_le_bytes([data[34], data[35]])],
            e_oemid: u16::from_le_bytes([data[36], data[37]]), e_oeminfo: u16::from_le_bytes([data[38], data[39]]),
            e_res2: [u16::from_le_bytes([data[40], data[41]]), u16::from_le_bytes([data[42], data[43]]), u16::from_le_bytes([data[44], data[45]]), u16::from_le_bytes([data[46], data[47]]), u16::from_le_bytes([data[48], data[49]]), u16::from_le_bytes([data[50], data[51]]), u16::from_le_bytes([data[52], data[53]]), u16::from_le_bytes([data[54], data[55]]), u16::from_le_bytes([data[56], data[57]]), u16::from_le_bytes([data[58], data[59]])],
            e_lfanew: lfanew,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageFileHeader {
    pub machine: u16, pub number_of_sections: u16, pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32, pub number_of_symbols: u32,
    pub size_of_optional_header: u16, pub characteristics: u16,
}

impl Default for ImageFileHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageFileHeader {
    pub const fn new() -> Self { Self { machine: 0, number_of_sections: 0, time_date_stamp: 0, pointer_to_symbol_table: 0, number_of_symbols: 0, size_of_optional_header: 0, characteristics: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 20 { return None; }
        Some(Self {
            machine: u16::from_le_bytes([data[0], data[1]]), number_of_sections: u16::from_le_bytes([data[2], data[3]]),
            time_date_stamp: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            pointer_to_symbol_table: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            number_of_symbols: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            size_of_optional_header: u16::from_le_bytes([data[16], data[17]]),
            characteristics: u16::from_le_bytes([data[18], data[19]]),
        })
    }
    pub fn machine_name(&self) -> &'static str {
        match self.machine {
            IMAGE_FILE_MACHINE_I386 => "i386", IMAGE_FILE_MACHINE_AMD64 => "x86_64",
            IMAGE_FILE_MACHINE_ARMNT => "ARM Thumb", IMAGE_FILE_MACHINE_ARM64 => "ARM64",
            IMAGE_FILE_MACHINE_RISCV32 => "RISC-V 32", IMAGE_FILE_MACHINE_RISCV64 => "RISC-V 64",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageDataDirectory { pub virtual_address: u32, pub size: u32 }
impl Default for ImageDataDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDataDirectory {
    pub const fn new() -> Self { Self { virtual_address: 0, size: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        Some(Self { virtual_address: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), size: u32::from_le_bytes([data[4], data[5], data[6], data[7]]) })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageOptionalHeader {
    pub magic: u16, pub major_linker_version: u8, pub minor_linker_version: u8,
    pub size_of_code: u32, pub size_of_initialized_data: u32, pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32, pub base_of_code: u32, pub image_base: u64,
    pub section_alignment: u32, pub file_alignment: u32,
    pub major_operating_system_version: u16, pub minor_operating_system_version: u16,
    pub major_image_version: u16, pub minor_image_version: u16,
    pub major_subsystem_version: u16, pub minor_subsystem_version: u16,
    pub win32_version_value: u32, pub size_of_image: u32, pub size_of_headers: u32,
    pub check_sum: u32, pub subsystem: u16, pub dll_characteristics: u16,
    pub size_of_stack_reserve: u64, pub size_of_stack_commit: u64,
    pub size_of_heap_reserve: u64, pub size_of_heap_commit: u64,
    pub loader_flags: u32, pub number_of_rva_and_sizes: u32,
    pub data_directory: [ImageDataDirectory; 16],
}

impl Default for ImageOptionalHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageOptionalHeader {
    pub fn new() -> Self {
        Self { magic: 0, major_linker_version: 0, minor_linker_version: 0, size_of_code: 0, size_of_initialized_data: 0, size_of_uninitialized_data: 0, address_of_entry_point: 0, base_of_code: 0, image_base: 0, section_alignment: 0, file_alignment: 0, major_operating_system_version: 0, minor_operating_system_version: 0, major_image_version: 0, minor_image_version: 0, major_subsystem_version: 0, minor_subsystem_version: 0, win32_version_value: 0, size_of_image: 0, size_of_headers: 0, check_sum: 0, subsystem: 0, dll_characteristics: 0, size_of_stack_reserve: 0, size_of_stack_commit: 0, size_of_heap_reserve: 0, size_of_heap_commit: 0, loader_flags: 0, number_of_rva_and_sizes: 0, data_directory: [ImageDataDirectory::new(); 16] }
    }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 2 { return None; }
        let magic = u16::from_le_bytes([data[0], data[1]]);
        let is_64 = magic == 0x20B;
        let mut off = 2;
        if data.len() < off + 22 { return None; }
        let major_linker_version = data[off];
        let minor_linker_version = data[off + 1];
        let size_of_code = u32::from_le_bytes([data[off+2], data[off+3], data[off+4], data[off+5]]);
        let size_of_initialized_data = u32::from_le_bytes([data[off+6], data[off+7], data[off+8], data[off+9]]);
        let size_of_uninitialized_data = u32::from_le_bytes([data[off+10], data[off+11], data[off+12], data[off+13]]);
        let address_of_entry_point = u32::from_le_bytes([data[off+14], data[off+15], data[off+16], data[off+17]]);
        let base_of_code = u32::from_le_bytes([data[off+18], data[off+19], data[off+20], data[off+21]]);
        off += 22;
        let image_base = if is_64 { if data.len() < off + 8 { return None; } let v = u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3], data[off+4], data[off+5], data[off+6], data[off+7]]); off += 8; v } else { if data.len() < off + 4 { return None; } let v = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as u64; off += 4; v };
        if data.len() < off + 28 { return None; }
        let section_alignment = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        let file_alignment = u32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
        let major_os = u16::from_le_bytes([data[off+8], data[off+9]]);
        let minor_os = u16::from_le_bytes([data[off+10], data[off+11]]);
        let major_img = u16::from_le_bytes([data[off+12], data[off+13]]);
        let minor_img = u16::from_le_bytes([data[off+14], data[off+15]]);
        let major_sub = u16::from_le_bytes([data[off+16], data[off+17]]);
        let minor_sub = u16::from_le_bytes([data[off+18], data[off+19]]);
        let win32_version = u32::from_le_bytes([data[off+20], data[off+21], data[off+22], data[off+23]]);
        let size_of_image = u32::from_le_bytes([data[off+24], data[off+25], data[off+26], data[off+27]]);
        off += 28;
        if data.len() < off + 16 { return None; }
        let size_of_headers = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        let check_sum = u32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
        let subsystem = u16::from_le_bytes([data[off+8], data[off+9]]);
        let dll_characteristics = u16::from_le_bytes([data[off+10], data[off+11]]);
        off += 12;
        let size_of_stack_reserve = if is_64 { if data.len() < off + 8 { return None; } let v = u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3], data[off+4], data[off+5], data[off+6], data[off+7]]); off += 8; v } else { if data.len() < off + 4 { return None; } let v = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as u64; off += 4; v };
        let size_of_stack_commit = if is_64 { if data.len() < off + 8 { return None; } let v = u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3], data[off+4], data[off+5], data[off+6], data[off+7]]); off += 8; v } else { if data.len() < off + 4 { return None; } let v = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as u64; off += 4; v };
        let size_of_heap_reserve = if is_64 { if data.len() < off + 8 { return None; } let v = u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3], data[off+4], data[off+5], data[off+6], data[off+7]]); off += 8; v } else { if data.len() < off + 4 { return None; } let v = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as u64; off += 4; v };
        let size_of_heap_commit = if is_64 { if data.len() < off + 8 { return None; } let v = u64::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3], data[off+4], data[off+5], data[off+6], data[off+7]]); off += 8; v } else { if data.len() < off + 4 { return None; } let v = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]) as u64; off += 4; v };
        if data.len() < off + 8 { return None; }
        let loader_flags = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
        let number_of_rva = u32::from_le_bytes([data[off+4], data[off+5], data[off+6], data[off+7]]);
        off += 8;
        let mut data_directory = [ImageDataDirectory::new(); 16];
        for i in 0..16usize {
            if data.len() < off + 8 { break; }
            data_directory[i] = ImageDataDirectory::parse(&data[off..off+8]).unwrap_or_default();
            off += 8;
        }
        Some(Self { magic, major_linker_version, minor_linker_version, size_of_code, size_of_initialized_data, size_of_uninitialized_data, address_of_entry_point, base_of_code, image_base, section_alignment, file_alignment, major_operating_system_version: major_os, minor_operating_system_version: minor_os, major_image_version: major_img, minor_image_version: minor_img, major_subsystem_version: major_sub, minor_subsystem_version: minor_sub, win32_version_value: win32_version, size_of_image, size_of_headers, check_sum, subsystem, dll_characteristics, size_of_stack_reserve, size_of_stack_commit, size_of_heap_reserve, size_of_heap_commit, loader_flags, number_of_rva_and_sizes: number_of_rva, data_directory })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageSectionHeader {
    pub name: [u8; 8], pub virtual_size: u32, pub virtual_address: u32,
    pub size_of_raw_data: u32, pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32, pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16, pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

impl Default for ImageSectionHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageSectionHeader {
    pub const fn new() -> Self { Self { name: [0; 8], virtual_size: 0, virtual_address: 0, size_of_raw_data: 0, pointer_to_raw_data: 0, pointer_to_relocations: 0, pointer_to_linenumbers: 0, number_of_relocations: 0, number_of_linenumbers: 0, characteristics: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 40 { return None; }
        let mut name = [0u8; 8];
        name.copy_from_slice(&data[0..8]);
        Some(Self { name, virtual_size: u32::from_le_bytes([data[8], data[9], data[10], data[11]]), virtual_address: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), size_of_raw_data: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), pointer_to_raw_data: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), pointer_to_relocations: u32::from_le_bytes([data[24], data[25], data[26], data[27]]), pointer_to_linenumbers: u32::from_le_bytes([data[28], data[29], data[30], data[31]]), number_of_relocations: u16::from_le_bytes([data[32], data[33]]), number_of_linenumbers: u16::from_le_bytes([data[34], data[35]]), characteristics: u32::from_le_bytes([data[36], data[37], data[38], data[39]]) })
    }
    pub fn name_as_str(&self) -> &str { let len = self.name.iter().position(|&b| b == 0).unwrap_or(8); core::str::from_utf8(&self.name[..len]).unwrap_or("?") }
    pub fn is_executable(&self) -> bool { (self.characteristics & 0x2000_0000) != 0 }
    pub fn is_readable(&self) -> bool { (self.characteristics & 0x4000_0000) != 0 }
    pub fn is_writable(&self) -> bool { (self.characteristics & 0x8000_0000) != 0 }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageImportDescriptor {
    pub original_first_thunk: u32, pub time_date_stamp: u32,
    pub forwarder_chain: u32, pub name_rva: u32, pub first_thunk: u32,
}

impl Default for ImageImportDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageImportDescriptor {
    pub const fn new() -> Self { Self { original_first_thunk: 0, time_date_stamp: 0, forwarder_chain: 0, name_rva: 0, first_thunk: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 20 { return None; }
        Some(Self { original_first_thunk: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), time_date_stamp: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), forwarder_chain: u32::from_le_bytes([data[8], data[9], data[10], data[11]]), name_rva: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), first_thunk: u32::from_le_bytes([data[16], data[17], data[18], data[19]]) })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImageExportDirectory {
    pub characteristics: u32, pub time_date_stamp: u32, pub major_version: u16, pub minor_version: u16,
    pub name_rva: u32, pub ordinal_base: u32, pub number_of_functions: u32, pub number_of_names: u32,
    pub address_of_functions: u32, pub address_of_names: u32, pub address_of_name_ordinals: u32,
}

impl Default for ImageExportDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageExportDirectory {
    pub const fn new() -> Self { Self { characteristics: 0, time_date_stamp: 0, major_version: 0, minor_version: 0, name_rva: 0, ordinal_base: 0, number_of_functions: 0, number_of_names: 0, address_of_functions: 0, address_of_names: 0, address_of_name_ordinals: 0 } }
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 40 { return None; }
        Some(Self { characteristics: u32::from_le_bytes([data[0], data[1], data[2], data[3]]), time_date_stamp: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), major_version: u16::from_le_bytes([data[8], data[9]]), minor_version: u16::from_le_bytes([data[10], data[11]]), name_rva: u32::from_le_bytes([data[12], data[13], data[14], data[15]]), ordinal_base: u32::from_le_bytes([data[16], data[17], data[18], data[19]]), number_of_functions: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), number_of_names: u32::from_le_bytes([data[24], data[25], data[26], data[27]]), address_of_functions: u32::from_le_bytes([data[28], data[29], data[30], data[31]]), address_of_names: u32::from_le_bytes([data[32], data[33], data[34], data[35]]), address_of_name_ordinals: u32::from_le_bytes([data[36], data[37], data[38], data[39]]) })
    }
}

#[derive(Debug, Clone)]
pub struct ImageBaseRelocation {
    pub virtual_address: u32, pub size_of_block: u32, pub type_offsets: Vec<u16>,
}

impl ImageBaseRelocation {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        let virtual_address = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let size_of_block = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let num_entries = (size_of_block as usize - 8) / 2;
        let mut type_offsets = Vec::new();
        for i in 0..num_entries {
            let off = 8 + i * 2;
            if data.len() < off + 2 { break; }
            type_offsets.push(u16::from_le_bytes([data[off], data[off+1]]));
        }
        Some(Self { virtual_address, size_of_block, type_offsets })
    }
}

#[derive(Debug, Clone)]
pub struct PeImport { pub dll_name: String, pub func_name: String, pub ordinal: u16, pub hint: u16 }

#[derive(Debug, Clone)]
pub struct PeExport { pub name: String, pub ordinal: u32, pub rva: u32 }

#[derive(Debug, Clone)]
pub struct PeParserResult {
    pub dos_header: ImageDosHeader, pub coff_header: ImageFileHeader, pub optional_header: ImageOptionalHeader,
    pub sections: Vec<ImageSectionHeader>, pub imports: Vec<PeImport>, pub exports: Vec<PeExport>,
    pub relocs: Vec<ImageBaseRelocation>, pub is_valid: bool, pub errors: Vec<String>,
}

impl Default for PeParserResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PeParserResult {
    pub fn new() -> Self { Self { dos_header: ImageDosHeader::new(), coff_header: ImageFileHeader::new(), optional_header: ImageOptionalHeader::new(), sections: Vec::new(), imports: Vec::new(), exports: Vec::new(), relocs: Vec::new(), is_valid: false, errors: Vec::new() } }
}

pub struct PeParser;

impl PeParser {
    pub fn parse(data: &[u8]) -> PeParserResult {
        let mut result = PeParserResult::new();
        if data.len() < 64 { result.errors.push(String::from("PE: Data too small for DOS header")); return result; }
        let dos = match ImageDosHeader::parse(data) { Some(d) => d, None => { result.errors.push(String::from("PE: Invalid DOS header")); return result; } };
        result.dos_header = dos;
        let pe_offset = dos.e_lfanew as usize;
        if data.len() < pe_offset + 4 { result.errors.push(String::from("PE: Invalid PE offset")); return result; }
        let pe_sig = u32::from_le_bytes([data[pe_offset], data[pe_offset+1], data[pe_offset+2], data[pe_offset+3]]);
        if pe_sig != IMAGE_NT_SIGNATURE { result.errors.push(format!("PE: Invalid PE signature: 0x{:08X}", pe_sig)); return result; }
        let coff_offset = pe_offset + 4;
        let coff = match ImageFileHeader::parse(&data[coff_offset..]) { Some(c) => c, None => { result.errors.push(String::from("PE: Invalid COFF header")); return result; } };
        result.coff_header = coff;
        let opt_offset = coff_offset + 20;
        let opt = match ImageOptionalHeader::parse(&data[opt_offset..]) { Some(o) => o, None => { result.errors.push(String::from("PE: Invalid Optional header")); return result; } };
        result.optional_header = opt;
        let sec_offset = opt_offset + if opt.magic == 0x20B { 240 } else { 224 };
        for i in 0..coff.number_of_sections as usize {
            let off = sec_offset + i * 40;
            if data.len() < off + 40 { break; }
            if let Some(sec) = ImageSectionHeader::parse(&data[off..]) { result.sections.push(sec); }
        }
        let import_dir = opt.data_directory[1];
        if import_dir.virtual_address != 0 && import_dir.size != 0 { Self::parse_imports(data, &result.sections, import_dir.virtual_address, import_dir.size, &mut result.imports); }
        let export_dir = opt.data_directory[0];
        if export_dir.virtual_address != 0 && export_dir.size != 0 { Self::parse_exports(data, &result.sections, export_dir.virtual_address, &mut result.exports); }
        let reloc_dir = opt.data_directory[5];
        if reloc_dir.virtual_address != 0 && reloc_dir.size != 0 { Self::parse_relocations(data, &result.sections, reloc_dir.virtual_address, reloc_dir.size, &mut result.relocs); }
        result.is_valid = true; result
    }
    pub fn rva_to_offset(sections: &[ImageSectionHeader], rva: u32) -> Option<usize> {
        for sec in sections { if rva >= sec.virtual_address && rva < sec.virtual_address + sec.virtual_size { return Some((rva - sec.virtual_address + sec.pointer_to_raw_data) as usize); } }
        None
    }
    fn parse_imports(data: &[u8], sections: &[ImageSectionHeader], rva: u32, _size: u32, out: &mut Vec<PeImport>) {
        let mut offset = match Self::rva_to_offset(sections, rva) { Some(o) => o, None => return };
        loop {
            if data.len() < offset + 20 { break; }
            let desc = match ImageImportDescriptor::parse(&data[offset..]) { Some(d) => d, None => break };
            if desc.original_first_thunk == 0 && desc.first_thunk == 0 { break; }
            let name_off = match Self::rva_to_offset(sections, desc.name_rva) { Some(o) => o, None => { offset += 20; continue; } };
            let dll_name = Self::read_ascii(data, name_off);
            let thunk_rva = if desc.original_first_thunk != 0 { desc.original_first_thunk } else { desc.first_thunk };
            let mut thunk_off = match Self::rva_to_offset(sections, thunk_rva) { Some(o) => o, None => { offset += 20; continue; } };
            loop {
                if data.len() < thunk_off + 8 { break; }
                let thunk = u64::from_le_bytes([data[thunk_off], data[thunk_off+1], data[thunk_off+2], data[thunk_off+3], data[thunk_off+4], data[thunk_off+5], data[thunk_off+6], data[thunk_off+7]]);
                if thunk == 0 { break; }
                if (thunk & 0x8000_0000_0000_0000) == 0 {
                    let hint_off = match Self::rva_to_offset(sections, (thunk & 0xFFFFFFFF) as u32) { Some(o) => o, None => { thunk_off += 8; continue; } };
                    if data.len() >= hint_off + 2 { let hint = u16::from_le_bytes([data[hint_off], data[hint_off+1]]); let func_name = Self::read_ascii(data, hint_off + 2); out.push(PeImport { dll_name: String::from(&dll_name), func_name, ordinal: 0, hint }); }
                } else { let ordinal = (thunk & 0xFFFF) as u16; out.push(PeImport { dll_name: String::from(&dll_name), func_name: String::from(""), ordinal, hint: 0 }); }
                thunk_off += 8;
            }
            offset += 20;
        }
    }
    fn parse_exports(data: &[u8], sections: &[ImageSectionHeader], rva: u32, out: &mut Vec<PeExport>) {
        let offset = match Self::rva_to_offset(sections, rva) { Some(o) => o, None => return };
        if data.len() < offset + 40 { return; }
        let export_dir = match ImageExportDirectory::parse(&data[offset..]) { Some(d) => d, None => return };
        let names_off = match Self::rva_to_offset(sections, export_dir.address_of_names) { Some(o) => o, None => return };
        let ordinals_off = match Self::rva_to_offset(sections, export_dir.address_of_name_ordinals) { Some(o) => o, None => return };
        let funcs_off = match Self::rva_to_offset(sections, export_dir.address_of_functions) { Some(o) => o, None => return };
        for i in 0..export_dir.number_of_names as usize {
            if data.len() < names_off + i * 4 + 4 { break; }
            let name_rva = u32::from_le_bytes([data[names_off + i*4], data[names_off + i*4 + 1], data[names_off + i*4 + 2], data[names_off + i*4 + 3]]);
            let name_off = match Self::rva_to_offset(sections, name_rva) { Some(o) => o, None => continue };
            let name = Self::read_ascii(data, name_off);
            if data.len() < ordinals_off + i * 2 + 2 { continue; }
            let ordinal = u16::from_le_bytes([data[ordinals_off + i*2], data[ordinals_off + i*2 + 1]]) as u32;
            let func_idx = (ordinal - export_dir.ordinal_base) as usize;
            if data.len() < funcs_off + func_idx * 4 + 4 { continue; }
            let func_rva = u32::from_le_bytes([data[funcs_off + func_idx*4], data[funcs_off + func_idx*4 + 1], data[funcs_off + func_idx*4 + 2], data[funcs_off + func_idx*4 + 3]]);
            out.push(PeExport { name, ordinal: ordinal + 1, rva: func_rva });
        }
    }
    fn parse_relocations(data: &[u8], sections: &[ImageSectionHeader], rva: u32, size: u32, out: &mut Vec<ImageBaseRelocation>) {
        let offset = match Self::rva_to_offset(sections, rva) { Some(o) => o, None => return };
        let mut cur = offset; let end = offset + size as usize;
        while cur + 8 <= end && cur + 8 <= data.len() { if let Some(rel) = ImageBaseRelocation::parse(&data[cur..]) { if rel.size_of_block == 0 { break; } cur += rel.size_of_block as usize; out.push(rel); } else { break; } }
    }
    pub fn read_ascii(data: &[u8], offset: usize) -> String {
        let mut s = String::new();
        for i in offset..data.len() { if data[i] == 0 { break; } else if data[i].is_ascii() { s.push(data[i] as char); } }
        s
    }
}

