// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : System Metrics & Environment Variables
// File Path            : apps/system/compat/win_layer/system_env.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::dos_emulator::console_writeln;
use alloc::format;


pub struct SystemMetrics;

impl SystemMetrics {
    pub fn get_system_metrics(index: i32) -> i32 {
        let val = match index {
            0 => 1920,   // SM_CXSCREEN
            1 => 1080,   // SM_CYSCREEN
            2 => 1,      // SM_CXVSCROLL
            3 => 1,      // SM_CYHSCROLL
            4 => 1,      // SM_CYCAPTION
            5 => 2,      // SM_CXBORDER
            6 => 2,      // SM_CYBORDER
            7 => 4,      // SM_CXDLGFRAME
            8 => 4,      // SM_CYDLGFRAME
            16 => 1,     // SM_MOUSEPRESENT
            17 => 2,     // SM_CYVSCROLL
            19 => 1,     // SM_CYMENUSIZE
            20 => 0,     // SM_ARRANGE
            30 => 1,     // SM_CXMIN
            31 => 1,     // SM_CYMIN
            32 => 0,     // SM_CXSIZE
            33 => 0,     // SM_CYSIZE
            34 => 1,     // SM_CXSIZEFRAME
            35 => 1,     // SM_CYSIZEFRAME
            43 => 256,   // SM_CXICON
            44 => 256,   // SM_CYICON
            47 => 16,    // SM_CXICONSPACING
            48 => 16,    // SM_CYICONSPACING
            49 => 1,     // SM_MENUDROPALIGNMENT
            61 => 96,    // SM_CXDRAG
            62 => 96,    // SM_CYDRAG
            67 => 3,     // SM_CXDOUBLECLK
            68 => 3,     // SM_CYDOUBLECLK
            70 => 0,     // SM_CXEDGE
            71 => 0,     // SM_CYEDGE
            76 => 4,     // SM_CXMINSPACING
            77 => 4,     // SM_CYMINSPACING
            80 => 32,    // SM_CXSMICON
            81 => 32,    // SM_CYSMICON
            85 => 1,     // SM_CXMINTRACK
            86 => 1,     // SM_CYMINTRACK
            89 => 1280,  // SM_CXMAXTRACK
            90 => 1024,  // SM_CYMAXTRACK
            91 => 0x10000, // SM_CXMAXIMIZED
            92 => 0x10000, // SM_CYMAXIMIZED
            _ => 0,
        };
        let msg = format!("[METRICS] GetSystemMetrics({}) -> {}", index, val);
        console_writeln(&msg); val
    }
}

pub struct EnvVarManager {
    pub vars: BTreeMap<String, String>,
}

impl Default for EnvVarManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvVarManager {
    pub fn new() -> Self {
        let mut m = Self { vars: BTreeMap::new() };
        m.vars.insert(String::from("PATH"), String::from("C:\\Windows;C:\\Windows\\System32"));
        m.vars.insert(String::from("SystemRoot"), String::from("C:\\Windows"));
        m.vars.insert(String::from("TEMP"), String::from("C:\\Users\\Local\\Temp"));
        m.vars.insert(String::from("TMP"), String::from("C:\\Users\\Local\\Temp"));
        m.vars.insert(String::from("USERPROFILE"), String::from("C:\\Users\\User"));
        m.vars.insert(String::from("APPDATA"), String::from("C:\\Users\\User\\AppData\\Roaming"));
        m.vars.insert(String::from("LOCALAPPDATA"), String::from("C:\\Users\\User\\AppData\\Local"));
        m.vars.insert(String::from("PROCESSOR_ARCHITECTURE"), String::from("AMD64"));
        m.vars.insert(String::from("NUMBER_OF_PROCESSORS"), String::from("8"));
        m.vars.insert(String::from("COMPUTERNAME"), String::from("OZKN-PC"));
        m.vars.insert(String::from("OS"), String::from("Windows_NT"));
        m.vars.insert(String::from("HOMEDRIVE"), String::from("C:"));
        m.vars.insert(String::from("HOMEPATH"), String::from("\\Users\\User"));
        m.vars.insert(String::from("ProgramFiles"), String::from("C:\\Program Files"));
        m.vars.insert(String::from("ProgramFiles(x86)"), String::from("C:\\Program Files (x86)"));
        m.vars.insert(String::from("ProgramData"), String::from("C:\\ProgramData"));
        m.vars.insert(String::from("PUBLIC"), String::from("C:\\Users\\Public"));
        m.vars.insert(String::from("SESSIONNAME"), String::from("Console"));
        m.vars.insert(String::from("WINDIR"), String::from("C:\\Windows"));
        m
    }
    pub fn get(&self, name: &str) -> Option<String> { self.vars.get(name).cloned() }
    pub fn set(&mut self, name: &str, value: &str) { self.vars.insert(String::from(name), String::from(value)); }
    pub fn remove(&mut self, name: &str) -> bool { self.vars.remove(name).is_some() }
    pub fn get_all(&self) -> Vec<(String, String)> { self.vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect() }
    pub fn expand(&self, input: &str) -> String {
        let mut result = String::from(input);
        for (k, v) in &self.vars {
            let pat = format!("%{name}%", name = k);
            if result.contains(&pat) {
                let mut new = String::new();
                new.push_str(&result.replace(&pat, v));
                result = new;
            }
        }
        result
    }
}
