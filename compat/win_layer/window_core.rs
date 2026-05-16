// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Win32 Window Manager Core
// File Path            : apps/system/compat/win_layer/window_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Window class registration, window instance creation, and
//   message queue management for the Win32 compatibility layer.
//   Integrates with the native ÖZKAN-OS window manager via FFI.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::win_layer::shared_defs::WinMsg;

extern "C" {
    fn ozkan_wm_create_window(x: i32, y: i32, w: u32, h: u32, title: *const u8) -> i32;
    fn ozkan_wm_draw_all();
}

#[derive(Debug, Clone)]
pub struct WindowMessage {
    pub hwnd: u64, pub msg: WinMsg, pub wparam: u64, pub lparam: u64,
    pub time: u64, pub pt_x: i32, pub pt_y: i32,
}

impl WindowMessage {
    pub fn new(hwnd: u64, msg: WinMsg, wparam: u64, lparam: u64) -> Self {
        Self { hwnd, msg, wparam, lparam, time: 0, pt_x: 0, pt_y: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct WindowClass {
    pub class_name: String, pub style: u32, pub wnd_proc: u64, pub h_instance: u64,
    pub h_icon: u64, pub h_cursor: u64, pub hbr_background: u64,
    pub menu_name: String, pub class_extra: i32, pub window_extra: i32,
}

impl WindowClass {
    pub fn new(class_name: &str) -> Self {
        Self { class_name: String::from(class_name), style: 0, wnd_proc: 0, h_instance: 0, h_icon: 0, h_cursor: 0, hbr_background: 0, menu_name: String::from(""), class_extra: 0, window_extra: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct WindowInstance {
    pub hwnd: u64, pub class_name: String, pub title: String, pub style: u32, pub ex_style: u32,
    pub x: i32, pub y: i32, pub width: i32, pub height: i32, pub parent: u64,
    pub h_instance: u64, pub user_data: u64, pub visible: bool, pub enabled: bool, pub focus: bool,
}

impl WindowInstance {
    pub fn new(hwnd: u64, class_name: &str, title: &str, x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { hwnd, class_name: String::from(class_name), title: String::from(title), style: 0x10CF0000, ex_style: 0, x, y, width, height, parent: 0, h_instance: 0, user_data: 0, visible: false, enabled: true, focus: false }
    }
}

#[derive(Debug, Clone)]
pub struct WindowManager {
    pub classes: Vec<WindowClass>, pub windows: Vec<WindowInstance>,
    pub messages: Vec<WindowMessage>, pub next_hwnd: u64,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager {
    pub fn new() -> Self { Self { classes: Vec::new(), windows: Vec::new(), messages: Vec::new(), next_hwnd: 0x10000 } }
    pub fn register_class(&mut self, class: WindowClass) { self.classes.push(class); }
    pub fn create_window(&mut self, class_name: &str, title: &str, x: i32, y: i32, width: i32, height: i32) -> u64 {
        let hwnd = self.next_hwnd; self.next_hwnd += 1;
        self.windows.push(WindowInstance::new(hwnd, class_name, title, x, y, width, height));
        let title_c = format!("{}\0", title);
        unsafe { ozkan_wm_create_window(x, y, width as u32, height as u32, title_c.as_ptr()); }
        hwnd
    }
    pub fn draw_all(&mut self) {
        unsafe { ozkan_wm_draw_all(); }
    }
    pub fn destroy_window(&mut self, hwnd: u64) -> bool {
        if let Some(idx) = self.windows.iter().position(|w| w.hwnd == hwnd) { self.windows.remove(idx); true } else { false }
    }
    pub fn post_message(&mut self, hwnd: u64, msg: WinMsg, wparam: u64, lparam: u64) {
        self.messages.push(WindowMessage::new(hwnd, msg, wparam, lparam));
    }
    pub fn peek_message(&mut self, hwnd: u64) -> Option<WindowMessage> {
        if let Some(idx) = self.messages.iter().position(|m| m.hwnd == hwnd) { Some(self.messages.remove(idx)) } else { None }
    }
    pub fn get_window(&self, hwnd: u64) -> Option<&WindowInstance> { self.windows.iter().find(|w| w.hwnd == hwnd) }
    pub fn get_window_mut(&mut self, hwnd: u64) -> Option<&mut WindowInstance> { self.windows.iter_mut().find(|w| w.hwnd == hwnd) }
    pub fn show_window(&mut self, hwnd: u64, cmd: i32) -> bool { if let Some(w) = self.get_window_mut(hwnd) { w.visible = cmd != 0; true } else { false } }
    pub fn set_window_text(&mut self, hwnd: u64, text: &str) -> bool { if let Some(w) = self.get_window_mut(hwnd) { w.title = String::from(text); true } else { false } }
}
