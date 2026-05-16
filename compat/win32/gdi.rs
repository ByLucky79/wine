// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : GDI Nesne Yöneticisi
// Dosya Yolu         : apps/system/compat/win32/gdi.rs
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
//   GDI nesne yöneticisi ve Console API.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

use crate::dos_emulator::console_writeln;
use alloc::format;
use alloc::string::String;
use super::app_manager::Win32Manager;


// ─── GDI Object Manager ────────────────────────────────────────

// ─── Console API (Win32Manager integration) ────────────────────

impl Win32Manager {
    pub fn alloc_console(&mut self) -> bool {
        self.console.allocated = true;
        let msg = format!("{} AllocConsole()", "[WIN32]");
        console_writeln(&msg); true
    }
    pub fn free_console(&mut self) -> bool {
        self.console.allocated = false;
        self.console.clear_output();
        let msg = format!("{} FreeConsole()", "[WIN32]");
        console_writeln(&msg); true
    }
    pub fn get_std_handle(&self, n: u32) -> u64 {
        let handle = 0xFFFFFFF0 - n as u64;
        let msg = format!("{} GetStdHandle({}) -> 0x{:08X}", "[WIN32]", n, handle);
        console_writeln(&msg); handle
    }
    pub fn write_console(&mut self, _handle: u64, text: &str, _written: u64, _reserved: u64) -> bool {
        let n = self.console.write(text);
        let msg = format!("{} WriteConsole(0x{:08X}, \"{}\") -> {} bytes", "[WIN32]", _handle, text, n);
        console_writeln(&msg); true
    }
    pub fn read_console(&mut self, _handle: u64, _buffer: u64, count: u32, _read: u64, _input_control: u64) -> bool {
        let msg = format!("{} ReadConsole(0x{:08X}, {})", "[WIN32]", _handle, count);
        console_writeln(&msg); true
    }
    pub fn set_console_title(&mut self, title: &str) -> bool {
        self.console.set_title(title);
        let msg = format!("{} SetConsoleTitle(\"{}\")", "[WIN32]", title);
        console_writeln(&msg); true
    }
    pub fn set_console_text_attribute(&mut self, _handle: u64, attr: u16) -> bool {
        self.console.set_attributes(attr);
        let msg = format!("{} SetConsoleTextAttribute(0x{:08X}, 0x{:04X})", "[WIN32]", _handle, attr);
        console_writeln(&msg); true
    }
    pub fn set_console_cursor_position(&mut self, _handle: u64, x: i16, y: i16) -> bool {
        self.console.set_cursor(x, y);
        let msg = format!("{} SetConsoleCursorPosition(0x{:08X}, {}, {})", "[WIN32]", _handle, x, y);
        console_writeln(&msg); true
    }
    pub fn get_console_mode(&self, _handle: u64, mode: &mut u32) -> bool {
        *mode = self.console.mode;
        let msg = format!("{} GetConsoleMode() -> 0x{:04X}", "[WIN32]", *mode);
        console_writeln(&msg); true
    }
    pub fn set_console_mode(&mut self, _handle: u64, mode: u32) -> bool {
        self.console.set_mode(mode);
        let msg = format!("{} SetConsoleMode(0x{:04X})", "[WIN32]", mode);
        console_writeln(&msg); true
    }
    pub fn get_console_title_a(&self, title: &mut String) -> u32 {
        title.push_str(&self.console.title);
        let msg = format!("{} GetConsoleTitleA() -> \"{}\"", "[WIN32]", self.console.title);
        console_writeln(&msg); self.console.title.len() as u32
    }
    pub fn get_console_screen_buffer_info(&self, _handle: u64, width: &mut i16, height: &mut i16, cur_x: &mut i16, cur_y: &mut i16) -> bool {
        *width = self.console.width;
        *height = self.console.height;
        *cur_x = self.console.cursor_x;
        *cur_y = self.console.cursor_y;
        let msg = format!("{} GetConsoleScreenBufferInfo() -> {}x{} cursor=({},{})"
            , "[WIN32]", *width, *height, *cur_x, *cur_y);
        console_writeln(&msg); true
    }
    pub fn fill_console_output_character_a(&mut self, _handle: u64, _ch: u16, _count: u32, _coord: u32, written: &mut u32) -> bool {
        *written = 0;
        let msg = format!("{} FillConsoleOutputCharacterA()", "[WIN32]");
        console_writeln(&msg); true
    }
    pub fn scroll_console_screen_buffer(&mut self, _handle: u64, _rect: u64, _clip: u64, _dest: u32, _fill: u64) -> bool {
        let msg = format!("{} ScrollConsoleScreenBuffer()", "[WIN32]");
        console_writeln(&msg); true
    }
    pub fn write_console_a(&mut self, _handle: u64, text: &str, written: &mut u32) -> bool {
        *written = self.console.write(text);
        let msg = format!("{} WriteConsoleA(\"{}\") -> {} bytes", "[WIN32]", text, *written);
        console_writeln(&msg); true
    }
    pub fn read_console_a(&mut self, _handle: u64, buf: &mut [u16], _count: u32, read: &mut u32) -> bool {
        *read = 0;
        if !buf.is_empty() && !self.console.input.is_empty() {
            let n = core::cmp::min(buf.len(), self.console.input.len());
            buf[..n].copy_from_slice(&self.console.input[..n]);
            *read = n as u32;
            self.console.input.drain(0..n);
        } else if !buf.is_empty() {
            buf[0] = 0x0D;
            buf[1] = 0x0A;
            *read = 2;
        }
        let msg = format!("{} ReadConsoleA() -> {} chars", "[WIN32]", *read);
        console_writeln(&msg); true
    }
}
