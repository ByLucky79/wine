// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Windows Registry Hive Parser
// File Path            : apps/system/compat/win_layer/registry_engine.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Windows Registry hive file parser. Parses REGF headers,
//   key nodes (nk), and value keys (vk) from raw binary hive data.
//   No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

const REGF_SIGNATURE: u32 = 0x66676572;
const HBIN_SIGNATURE: u32 = 0x6E696268;
const KEY_CELL_SIGNATURE: u16 = 0x6B6E;
const VALUE_CELL_SIGNATURE: u16 = 0x6B76;

#[derive(Debug, Clone, Copy)]
pub struct HiveHeader {
    pub signature: u32, pub primary_sequence: u32, pub secondary_sequence: u32,
    pub timestamp: u64, pub major_version: u32, pub minor_version: u32,
    pub file_type: u32, pub format: u32, pub root_cell_offset: u32,
    pub hive_data_size: u32, pub clustering_factor: u32,
}

impl HiveHeader {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 512 { return None; }
        let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if sig != REGF_SIGNATURE { return None; }
        Some(Self { signature: sig, primary_sequence: u32::from_le_bytes([data[4], data[5], data[6], data[7]]), secondary_sequence: u32::from_le_bytes([data[8], data[9], data[10], data[11]]), timestamp: u64::from_le_bytes([data[12], data[13], data[14], data[15], data[16], data[17], data[18], data[19]]), major_version: u32::from_le_bytes([data[20], data[21], data[22], data[23]]), minor_version: u32::from_le_bytes([data[24], data[25], data[26], data[27]]), file_type: u32::from_le_bytes([data[28], data[29], data[30], data[31]]), format: u32::from_le_bytes([data[32], data[33], data[34], data[35]]), root_cell_offset: u32::from_le_bytes([data[36], data[37], data[38], data[39]]), hive_data_size: u32::from_le_bytes([data[40], data[41], data[42], data[43]]), clustering_factor: u32::from_le_bytes([data[44], data[45], data[46], data[47]]) })
    }
}

#[derive(Debug, Clone)]
pub struct HiveKeyNode {
    pub signature: u16, pub flags: u16, pub last_write_time: u64, pub access_bits: u32,
    pub parent_offset: i32, pub subkey_count: u32, pub volatile_subkey_count: u32,
    pub subkey_list_offset: i32, pub volatile_subkey_list_offset: i32,
    pub value_count: u32, pub value_list_offset: i32, pub security_offset: i32,
    pub class_name_offset: i32, pub max_subkey_name_length: u16, pub max_class_name_length: u16,
    pub max_value_name_length: u16, pub max_value_data_length: u32, pub key_name: String,
}

impl HiveKeyNode {
    pub fn parse(data: &[u8], offset: usize) -> Option<Self> {
        if data.len() < offset + 76 { return None; }
        let sig = u16::from_le_bytes([data[offset], data[offset+1]]);
        if sig != KEY_CELL_SIGNATURE { return None; }
        let name_len = u16::from_le_bytes([data[offset+72], data[offset+73]]) as usize;
        let mut key_name = String::new();
        for i in 0..name_len { if offset + 76 + i >= data.len() { break; } key_name.push(data[offset + 76 + i] as char); }
        Some(Self { signature: sig, flags: u16::from_le_bytes([data[offset+2], data[offset+3]]), last_write_time: u64::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7], data[offset+8], data[offset+9], data[offset+10], data[offset+11]]), access_bits: u32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]), parent_offset: i32::from_le_bytes([data[offset+16], data[offset+17], data[offset+18], data[offset+19]]), subkey_count: u32::from_le_bytes([data[offset+20], data[offset+21], data[offset+22], data[offset+23]]), volatile_subkey_count: u32::from_le_bytes([data[offset+24], data[offset+25], data[offset+26], data[offset+27]]), subkey_list_offset: i32::from_le_bytes([data[offset+28], data[offset+29], data[offset+30], data[offset+31]]), volatile_subkey_list_offset: i32::from_le_bytes([data[offset+32], data[offset+33], data[offset+34], data[offset+35]]), value_count: u32::from_le_bytes([data[offset+36], data[offset+37], data[offset+38], data[offset+39]]), value_list_offset: i32::from_le_bytes([data[offset+40], data[offset+41], data[offset+42], data[offset+43]]), security_offset: i32::from_le_bytes([data[offset+44], data[offset+45], data[offset+46], data[offset+47]]), class_name_offset: i32::from_le_bytes([data[offset+48], data[offset+49], data[offset+50], data[offset+51]]), max_subkey_name_length: u16::from_le_bytes([data[offset+52], data[offset+53]]), max_class_name_length: u16::from_le_bytes([data[offset+54], data[offset+55]]), max_value_name_length: u16::from_le_bytes([data[offset+56], data[offset+57]]), max_value_data_length: u32::from_le_bytes([data[offset+60], data[offset+61], data[offset+62], data[offset+63]]), key_name })
    }
}

#[derive(Debug, Clone)]
pub struct HiveValueKey {
    pub signature: u16, pub name_length: u16, pub data_size: u32, pub data_offset: i32,
    pub data_type: u32, pub flags: u16, pub value_name: String, pub value_data: Vec<u8>,
}

impl HiveValueKey {
    pub fn parse(data: &[u8], offset: usize) -> Option<Self> {
        if data.len() < offset + 20 { return None; }
        let sig = u16::from_le_bytes([data[offset], data[offset+1]]);
        if sig != VALUE_CELL_SIGNATURE { return None; }
        let name_len = u16::from_le_bytes([data[offset+2], data[offset+3]]) as usize;
        let data_size = u32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
        let data_offset = i32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]);
        let data_type = u32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]);
        let flags = u16::from_le_bytes([data[offset+16], data[offset+17]]);
        let mut value_name = String::new();
        for i in 0..name_len { if offset + 20 + i >= data.len() { break; } value_name.push(data[offset + 20 + i] as char); }
        Some(Self { signature: sig, name_length: name_len as u16, data_size, data_offset, data_type, flags, value_name, value_data: Vec::new() })
    }
    pub fn data_as_string(&self) -> Option<String> {
        if self.data_type == 1 || self.data_type == 2 {
            let mut s = String::new();
            for i in (0..self.value_data.len()).step_by(2) { if i + 1 >= self.value_data.len() { break; } let c = u16::from_le_bytes([self.value_data[i], self.value_data[i+1]]); if c == 0 { break; } else if let Some(ch) = core::char::from_u32(c as u32) { s.push(ch); } }
            Some(s)
        } else if self.data_type == 4 || self.data_type == 5 {
            if self.value_data.len() >= 4 { Some(format!("{}", u32::from_le_bytes([self.value_data[0], self.value_data[1], self.value_data[2], self.value_data[3]]))) } else { None }
        } else { None }
    }
}

pub struct RegistryHiveParser;
impl RegistryHiveParser {
    pub fn parse(data: &[u8]) -> Option<HiveHeader> { HiveHeader::parse(data) }
    pub fn parse_key_node(data: &[u8], offset: usize) -> Option<HiveKeyNode> { HiveKeyNode::parse(data, offset) }
    pub fn parse_value_key(data: &[u8], offset: usize) -> Option<HiveValueKey> { HiveValueKey::parse(data, offset) }
}
