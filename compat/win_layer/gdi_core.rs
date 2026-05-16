// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Win32 GDI Core
// File Path            : apps/system/compat/win_layer/gdi_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   GDI object manager and native graphics backend integration.
//   Handles pens, brushes, bitmaps, and delegates drawing to the
//   ÖZKAN-OS graphics subsystem via FFI. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;
use crate::win_layer::shared_defs::GdiObjectType;

extern "C" {
    fn ozkan_gfx_fill_rect(x: i32, y: i32, w: i32, h: i32, color: u32);
    fn ozkan_gfx_draw_text(x: i32, y: i32, text: *const u8, len: u32, color: u32);
    fn ozkan_gfx_swap_buffers();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GdiObject {
    pub handle: u64, pub obj_type: GdiObjectType, pub color: u32, pub width: i32, pub height: i32,
}

pub struct GdiManager {
    pub objects: Vec<GdiObject>, pub next_handle: u64,
}

impl Default for GdiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GdiManager {
    pub fn new() -> Self { Self { objects: Vec::new(), next_handle: 0x10000 } }
    pub fn create_pen(width: i32, color: u32, gm: &mut GdiManager) -> u64 {
        let handle = gm.next_handle; gm.next_handle += 1;
        gm.objects.push(GdiObject { handle, obj_type: GdiObjectType::Pen, color, width, height: 0 });
        let msg = format!("[GDI] CreatePen(width={}, color=0x{:08X}) -> 0x{:08X}", width, color, handle);
        console_writeln(&msg); handle
    }
    pub fn create_solid_brush(color: u32, gm: &mut GdiManager) -> u64 {
        let handle = gm.next_handle; gm.next_handle += 1;
        gm.objects.push(GdiObject { handle, obj_type: GdiObjectType::Brush, color, width: 0, height: 0 });
        let msg = format!("[GDI] CreateSolidBrush(color=0x{:08X}) -> 0x{:08X}", color, handle);
        console_writeln(&msg); handle
    }
    pub fn create_bitmap(width: i32, height: i32, gm: &mut GdiManager) -> u64 {
        let handle = gm.next_handle; gm.next_handle += 1;
        gm.objects.push(GdiObject { handle, obj_type: GdiObjectType::Bitmap, color: 0, width, height });
        let msg = format!("[GDI] CreateBitmap({}, {}) -> 0x{:08X}", width, height, handle);
        console_writeln(&msg); handle
    }
    pub fn delete_object(handle: u64, gm: &mut GdiManager) -> bool {
        if let Some(idx) = gm.objects.iter().position(|o| o.handle == handle) { gm.objects.remove(idx); true } else { false }
    }
    pub fn get_object(handle: u64, gm: &GdiManager) -> Option<&GdiObject> { gm.objects.iter().find(|o| o.handle == handle) }
    /// Fill a rectangle on the display using the native graphics backend.
    pub fn fill_rect(_gm: &GdiManager, x: i32, y: i32, w: i32, h: i32, color: u32) {
        unsafe { ozkan_gfx_fill_rect(x, y, w, h, color); }
    }
    /// Draw text on the display using the native graphics backend.
    pub fn draw_text(_gm: &GdiManager, x: i32, y: i32, text: &str, color: u32) {
        let buf = format!("{}\0", text);
        unsafe { ozkan_gfx_draw_text(x, y, buf.as_ptr(), text.len() as u32, color); }
    }
    /// Present the back-buffer to the display.
    pub fn swap_buffers(_gm: &GdiManager) {
        unsafe { ozkan_gfx_swap_buffers(); }
    }
}

#[derive(Debug, Clone)]
pub struct HdcEntry {
    pub hwnd: u64, pub selected_pen: u64, pub selected_brush: u64,
    pub selected_font: u64, pub selected_bitmap: u64,
}

impl HdcEntry {
    pub fn new(hwnd: u64) -> Self {
        Self { hwnd, selected_pen: 0, selected_brush: 0, selected_font: 0, selected_bitmap: 0 }
    }
}
