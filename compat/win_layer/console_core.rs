// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Win32 Console Buffer
// File Path            : apps/system/compat/win_layer/console_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Windows console buffer emulation for the compatibility layer.
//   Manages console title, dimensions, cursor position, text
//   attributes, and input/output buffers. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct ConsoleBuffer {
    pub title: String,
    pub width: i16,
    pub height: i16,
    pub cursor_x: i16,
    pub cursor_y: i16,
    pub mode: u32,
    pub attributes: u16,
    pub output: Vec<u8>,
    pub input: Vec<u16>,
    pub allocated: bool,
}

impl ConsoleBuffer {
    pub fn new() -> Self {
        Self {
            title: String::from("OZ-DOS Console"),
            width: 80,
            height: 25,
            cursor_x: 0,
            cursor_y: 0,
            mode: 0x0007,
            attributes: 0x0007,
            output: Vec::new(),
            input: Vec::new(),
            allocated: false,
        }
    }
    pub fn write(&mut self, text: &str) -> u32 {
        let n = text.len() as u32;
        self.output.extend_from_slice(text.as_bytes());
        n
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
    pub fn set_cursor(&mut self, x: i16, y: i16) {
        self.cursor_x = x.max(0).min(self.width - 1);
        self.cursor_y = y.max(0).min(self.height - 1);
    }
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }
    pub fn set_attributes(&mut self, attr: u16) {
        self.attributes = attr;
    }
    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}
