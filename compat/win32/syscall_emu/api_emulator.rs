// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 API Emulasyon - Win32ApiEmulator impl ve testler
// Dosya Yolu         : apps/system/compat/win32/api_emulator.rs
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
//   Win32 API Emulasyon - Win32ApiEmulator impl ve testler
//
// Bagimli Dosyalar:
//   1-) apps/system/compat/win32/win_objects.rs
//   2-) apps/system/compat/win32/process_mgr_extra.rs
//   3-) apps/system/compat/win32/pe_helpers.rs
//
//              Dosyaya Mudahaleler
// 2026-05-13      syscall_emu.rs bolundu
// *******************************************************************

extern crate alloc;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use crate::dos_emulator::console_writeln;
use crate::win_layer::window_core::{WindowManager, WindowClass, WindowMessage};
use crate::win32::app_manager::{Win32Manager, Win32Module, Win32FileHandle, Win32Event, Win32Mutex};
use crate::win32::process_mgr::{MappedImage, ProcessMemoryManager, MemoryRegion};
use crate::win32::registry::{WinProcess, RegistryKey};
use crate::win32::pe_loader::{CLR_WIN_BG, CLR_ACCENT, CLR_TEXT};
use crate::win_layer::shared_defs::{WinMsg, RegistryValue, SystemInfo, OsVersionInfo, GdiObjectType, ExceptionDisposition};
use crate::win_layer::gdi_core::{GdiManager, HdcEntry};
use crate::win_layer::seh_core::{ExceptionRecord, ContextRecord, ExceptionHandlerFn};

pub struct Win32ApiEmulator;

impl Win32ApiEmulator {
    pub fn create_window_ex(ex_style: u32, class_name: &str, window_name: &str, style: u32, x: i32, y: i32, width: i32, height: i32, parent: u64, _menu: u64, h_instance: u64, lp_param: u64, wm: &mut WindowManager) -> u64 {
        let hwnd = wm.create_window(class_name, window_name, x, y, width, height);
        if let Some(w) = wm.get_window_mut(hwnd) { w.ex_style = ex_style; w.style = style; w.parent = parent; w.h_instance = h_instance; w.user_data = lp_param; }
        let msg = format!("[USER32] CreateWindowEx -> hwnd=0x{:08X}", hwnd);
        console_writeln(&msg); hwnd
    }
    pub fn register_class_ex(class: WindowClass, wm: &mut WindowManager) -> u16 {
        wm.register_class(class);
        let atom = wm.classes.len() as u16;
        let msg = format!("[USER32] RegisterClassEx -> atom=0x{:04X}", atom);
        console_writeln(&msg); atom
    }
    pub fn get_module_handle(mgr: &Win32Manager, name: &str) -> u64 {
        if name.is_empty() { return 0x400000; }
        for (base, module) in &mgr.modules { if module.name.eq_ignore_ascii_case(name) { return *base; } }
        0
    }
    pub fn get_proc_address(mgr: &Win32Manager, module: u64, name: &str) -> u64 {
        if let Some(mod_) = mgr.modules.get(&module) {
            if let Some(&addr) = mod_.exports.get(name) { return addr; }
        }
        module + 0x1000 + name.len() as u64
    }
    pub fn load_library(mgr: &mut Win32Manager, name: &str) -> u64 {
        let base = mgr.next_module_base; mgr.next_module_base += 0x100000;
        let mut exports = BTreeMap::new();
        exports.insert(String::from("Init"), base + 0x1000);
        mgr.modules.insert(base, Win32Module { name: String::from(name), base, image: MappedImage::new(base, 0x100000), exports });
        let msg = format!("[KERNEL32] LoadLibrary({}) -> base=0x{:08X}", name, base);
        console_writeln(&msg); base
    }
    pub fn virtual_alloc(pmm: &mut ProcessMemoryManager, address: u64, size: u64, alloc_type: u32, protect: u32) -> u64 {
        let addr = if address == 0 { pmm.allocate(size, protect) } else { address };
        let msg = format!("[KERNEL32] VirtualAlloc(0x{:016X}, 0x{:08X}, type=0x{:08X}, prot=0x{:08X})", addr, size, alloc_type, protect);
        console_writeln(&msg); addr
    }
    pub fn virtual_free(pmm: &mut ProcessMemoryManager, address: u64, _size: u64, free_type: u32) -> bool {
        let msg = format!("[KERNEL32] VirtualFree(0x{:016X}, type=0x{:08X})", address, free_type);
        console_writeln(&msg); pmm.free(address)
    }
    pub fn virtual_protect(pmm: &mut ProcessMemoryManager, address: u64, _size: u64, new_prot: u32) -> bool {
        let msg = format!("[KERNEL32] VirtualProtect(0x{:016X}, prot=0x{:08X})", address, new_prot);
        console_writeln(&msg); pmm.protect(address, new_prot)
    }
    pub fn create_file(mgr: &mut Win32Manager, name: &str, desired_access: u32, share_mode: u32, creation_disposition: u32, _flags: u32) -> u64 {
        let mut data = Vec::new();
        match creation_disposition {
            1 => { // CREATE_NEW
                if mgr.vfs.contains_key(name) { mgr.last_error = 80; return 0xFFFFFFFF; }
                mgr.vfs.insert(String::from(name), Vec::new());
            }
            2 => { // CREATE_ALWAYS
                mgr.vfs.insert(String::from(name), Vec::new());
            }
            3 => { // OPEN_EXISTING
                if let Some(v) = mgr.vfs.get(name) { data = v.clone(); }
                else { mgr.last_error = 2; return 0xFFFFFFFF; }
            }
            4 => { // OPEN_ALWAYS
                if let Some(v) = mgr.vfs.get(name) { data = v.clone(); }
                else { mgr.vfs.insert(String::from(name), Vec::new()); }
            }
            5 => { // TRUNCATE_EXISTING
                if !mgr.vfs.contains_key(name) { mgr.last_error = 2; return 0xFFFFFFFF; }
                mgr.vfs.insert(String::from(name), Vec::new());
            }
            _ => {}
        }
        let handle = mgr.next_handle; mgr.next_handle += 1;
        mgr.files.insert(handle, Win32FileHandle { name: String::from(name), data, pos: 0, access: desired_access, share: share_mode });
        mgr.last_error = 0;
        let msg = format!("[KERNEL32] CreateFile({}) -> handle=0x{:08X}", name, handle);
        console_writeln(&msg); handle
    }
    pub fn read_file(mgr: &mut Win32Manager, handle: u64, buffer: &mut [u8]) -> (bool, u32) {
        if let Some(fh) = mgr.files.get_mut(&handle) {
            let read = buffer.len().min(fh.data.len().saturating_sub(fh.pos));
            buffer[..read].copy_from_slice(&fh.data[fh.pos..fh.pos + read]);
            fh.pos += read;
            mgr.last_error = 0;
            let msg = format!("[KERNEL32] ReadFile(handle=0x{:08X}, {} bytes)", handle, read);
            console_writeln(&msg); (true, read as u32)
        } else {
            mgr.last_error = 6;
            let msg = format!("[KERNEL32] ReadFile(handle=0x{:08X}) -> INVALID_HANDLE", handle);
            console_writeln(&msg); (false, 0)
        }
    }
    pub fn write_file(mgr: &mut Win32Manager, handle: u64, buffer: &[u8]) -> (bool, u32) {
        if let Some(fh) = mgr.files.get_mut(&handle) {
            let write = buffer.len();
            if fh.pos + write > fh.data.len() { fh.data.resize(fh.pos + write, 0); }
            fh.data[fh.pos..fh.pos + write].copy_from_slice(buffer);
            fh.pos += write;
            mgr.vfs.insert(fh.name.clone(), fh.data.clone());
            mgr.last_error = 0;
            let msg = format!("[KERNEL32] WriteFile(handle=0x{:08X}, {} bytes)", handle, write);
            console_writeln(&msg); (true, write as u32)
        } else { mgr.last_error = 6; (false, 0) }
    }
    pub fn set_file_pointer(mgr: &mut Win32Manager, handle: u64, distance: i32, method: u32) -> u32 {
        if let Some(fh) = mgr.files.get_mut(&handle) {
            let new_pos = match method {
                0 => distance as usize,
                1 => (fh.pos as i64 + distance as i64) as usize,
                2 => (fh.data.len() as i64 + distance as i64) as usize,
                _ => fh.pos,
            };
            fh.pos = new_pos.min(fh.data.len());
            mgr.last_error = 0;
            fh.pos as u32
        } else { mgr.last_error = 6; 0xFFFFFFFF }
    }
    pub fn get_file_size(mgr: &Win32Manager, handle: u64) -> u32 {
        if let Some(fh) = mgr.files.get(&handle) {
            fh.data.len() as u32
        } else { 0xFFFFFFFF }
    }
    pub fn close_handle(mgr: &mut Win32Manager, handle: u64) -> bool {
        let removed = mgr.files.remove(&handle).is_some()
            || mgr.modules.remove(&handle).is_some()
            || mgr.events.remove(&handle).is_some()
            || mgr.mutexes.remove(&handle).is_some();
        if removed { mgr.last_error = 0; } else { mgr.last_error = 6; }
        let msg = format!("[KERNEL32] CloseHandle(0x{:08X}) -> {}", handle, removed);
        console_writeln(&msg); removed
    }
    pub fn virtual_query(pmm: &ProcessMemoryManager, address: u64) -> Option<MemoryRegion> {
        pmm.regions.iter().find(|r| r.base <= address && address < r.base + r.size).cloned()
    }
    pub fn reg_open_key_ex(mgr: &mut Win32Manager, path: &str) -> u64 {
        if mgr.registry.contains_key(path) {
            let handle = mgr.next_handle; mgr.next_handle += 1;
            mgr.last_error = 0; handle
        } else { mgr.last_error = 2; 0 }
    }
    pub fn reg_query_value_ex(mgr: &Win32Manager, path: &str, value_name: &str) -> Option<RegistryValue> {
        mgr.registry.get(path)?.get_value(value_name).cloned()
    }
    pub fn reg_close_key(_mgr: &mut Win32Manager, _handle: u64) -> bool {
        true
    }
    pub fn reg_create_key_ex(mgr: &mut Win32Manager, path: &str) -> u64 {
        if !mgr.registry.contains_key(path) {
            mgr.registry.insert(String::from(path), RegistryKey::new(path));
        }
        let handle = mgr.next_handle; mgr.next_handle += 1;
        let msg = format!("[ADVAPI32] RegCreateKeyEx(\"{}\") -> 0x{:08X}", path, handle);
        console_writeln(&msg); handle
    }
    pub fn reg_set_value_ex(mgr: &mut Win32Manager, path: &str, value_name: &str, value: RegistryValue) -> bool {
        if let Some(key) = mgr.registry.get_mut(path) {
            key.set_value(value_name, value);
            let msg = format!("[ADVAPI32] RegSetValueEx(\"{}\", \"{}\") -> true", path, value_name);
            console_writeln(&msg); true
        } else {
            let msg = format!("[ADVAPI32] RegSetValueEx(\"{}\", \"{}\") -> false (key not found)", path, value_name);
            console_writeln(&msg); false
        }
    }
    pub fn reg_delete_value(mgr: &mut Win32Manager, path: &str, value_name: &str) -> bool {
        if let Some(key) = mgr.registry.get_mut(path) {
            let ok = key.values.remove(value_name).is_some();
            let msg = format!("[ADVAPI32] RegDeleteValue(\"{}\", \"{}\") -> {}", path, value_name, ok);
            console_writeln(&msg); ok
        } else {
            let msg = format!("[ADVAPI32] RegDeleteValue(\"{}\", \"{}\") -> false (key not found)", path, value_name);
            console_writeln(&msg); false
        }
    }
    pub fn reg_enum_value(mgr: &Win32Manager, path: &str, index: u32) -> Option<(String, RegistryValue)> {
        mgr.registry.get(path)?.values.iter().nth(index as usize).map(|(k, v)| (k.clone(), v.clone()))
    }
    pub fn reg_enum_key_ex(mgr: &Win32Manager, path: &str, index: u32) -> Option<String> {
        mgr.registry.get(path)?.subkeys.get(index as usize).cloned()
    }
    pub fn reg_delete_key(mgr: &mut Win32Manager, path: &str) -> bool {
        let ok = mgr.registry.remove(path).is_some();
        let msg = format!("[ADVAPI32] RegDeleteKey(\"{}\") -> {}", path, ok);
        console_writeln(&msg); ok
    }
    pub fn create_thread(proc: &mut WinProcess, entry: u64, _param: u64) -> u32 {
        proc.create_thread(entry)
    }
    pub fn suspend_thread(proc: &mut WinProcess, tid: u32) -> bool {
        if let Some(t) = proc.threads.iter_mut().find(|t| t.tid == tid) {
            t.suspend(); true
        } else { false }
    }
    pub fn resume_thread(proc: &mut WinProcess, tid: u32) -> bool {
        if let Some(t) = proc.threads.iter_mut().find(|t| t.tid == tid) {
            t.resume(); true
        } else { false }
    }
    pub fn wait_for_single_object(mgr: &Win32Manager, handle: u64, timeout: u32) -> u32 {
        if let Some(ev) = mgr.events.get(&handle) {
            if ev.signaled { 0 } else if timeout == 0 { 0x102 } else { 0 }
        } else if let Some(mx) = mgr.mutexes.get(&handle) {
            if !mx.owned { 0 } else if timeout == 0 { 0x102 } else { 0 }
        } else { 0xFFFFFFFF }
    }
    pub fn create_event(mgr: &mut Win32Manager, manual_reset: bool, initial_state: bool) -> u64 {
        let handle = mgr.next_handle; mgr.next_handle += 1;
        mgr.events.insert(handle, Win32Event { signaled: initial_state, manual_reset });
        let msg = format!("[KERNEL32] CreateEvent -> handle=0x{:08X}", handle);
        console_writeln(&msg); handle
    }
    pub fn set_event(mgr: &mut Win32Manager, handle: u64) -> bool {
        if let Some(ev) = mgr.events.get_mut(&handle) {
            ev.signaled = true; true
        } else { false }
    }
    pub fn reset_event(mgr: &mut Win32Manager, handle: u64) -> bool {
        if let Some(ev) = mgr.events.get_mut(&handle) {
            ev.signaled = false; true
        } else { false }
    }
    pub fn create_mutex(mgr: &mut Win32Manager, owned: bool) -> u64 {
        let handle = mgr.next_handle; mgr.next_handle += 1;
        mgr.mutexes.insert(handle, Win32Mutex { owned, thread_id: 0 });
        let msg = format!("[KERNEL32] CreateMutex -> handle=0x{:08X}", handle);
        console_writeln(&msg); handle
    }
    pub fn release_mutex(mgr: &mut Win32Manager, handle: u64) -> bool {
        if let Some(mx) = mgr.mutexes.get_mut(&handle) {
            mx.owned = false; true
        } else { false }
    }
    pub fn get_current_process_id(mgr: &Win32Manager) -> u32 { mgr.current_process }
    pub fn get_current_thread_id(mgr: &Win32Manager) -> u32 { mgr.current_thread }
    pub fn exit_process(mgr: &mut Win32Manager, pid: u32, code: u32) -> bool {
        if let Some(p) = mgr.find_process_mut(pid) { p.exit_code = code; }
        mgr.terminate_process(pid)
    }
    pub fn exit_thread(proc: &mut WinProcess, tid: u32, code: u32) -> bool {
        if let Some(idx) = proc.threads.iter().position(|t| t.tid == tid) {
            proc.threads[idx].exit_code = code as i32;
            proc.threads.remove(idx); true
        } else { false }
    }
    pub fn get_last_error(mgr: &Win32Manager) -> u32 { mgr.last_error }
    pub fn set_last_error(mgr: &mut Win32Manager, err: u32) { mgr.last_error = err; }

    // ─── GDI API Wrappers ──────────────────────────────────────

    pub fn def_window_proc(mgr: &mut Win32Manager, hwnd: u64, msg: WinMsg, wparam: u64, lparam: u64) -> u64 {
        match msg {
            WinMsg::Close => {
                let m = format!("[USER32] DefWindowProc WM_CLOSE hwnd=0x{:08X}", hwnd);
                console_writeln(&m);
                mgr.window_manager.destroy_window(hwnd); 0
            }
            WinMsg::Destroy => {
                let m = format!("[USER32] DefWindowProc WM_DESTROY hwnd=0x{:08X}", hwnd);
                console_writeln(&m); 0
            }
            WinMsg::Size => {
                let (width, height) = ((lparam & 0xFFFF) as i32, ((lparam >> 16) & 0xFFFF) as i32);
                if let Some(w) = mgr.window_manager.get_window_mut(hwnd) { w.width = width; w.height = height; }
                let m = format!("[USER32] DefWindowProc WM_SIZE hwnd=0x{:08X}, {}x{}", hwnd, width, height);
                console_writeln(&m); 0
            }
            WinMsg::Paint => {
                let m = format!("[USER32] DefWindowProc WM_PAINT hwnd=0x{:08X}", hwnd);
                console_writeln(&m); 0
            }
            WinMsg::ShowWindow => {
                let m = format!("[USER32] DefWindowProc WM_SHOWWINDOW hwnd=0x{:08X}, show={}", hwnd, wparam);
                console_writeln(&m); 0
            }
            WinMsg::SetFocus | WinMsg::KillFocus => {
                let m = format!("[USER32] DefWindowProc {:?} hwnd=0x{:08X}", msg, hwnd);
                console_writeln(&m); 0
            }
            _ => 0
        }
    }
    pub fn dispatch_message(mgr: &mut Win32Manager, msg: &WindowMessage) -> u64 {
        let wnd_proc = if let Some(win) = mgr.window_manager.get_window(msg.hwnd) {
            mgr.window_manager.classes.iter().find(|c| c.class_name == win.class_name).map(|c| c.wnd_proc).unwrap_or(0)
        } else { 0 };
        if wnd_proc != 0 {
            let m = format!("[USER32] DispatchMessage hwnd=0x{:08X}, msg={:?}, wnd_proc=0x{:016X}", msg.hwnd, msg.msg, wnd_proc);
            console_writeln(&m);
        }
        Win32ApiEmulator::def_window_proc(mgr, msg.hwnd, msg.msg, msg.wparam, msg.lparam)
    }
    pub fn send_message(mgr: &mut Win32Manager, hwnd: u64, msg: WinMsg, wparam: u64, lparam: u64) -> u64 {
        let m = format!("[USER32] SendMessage hwnd=0x{:08X}, msg={:?}", hwnd, msg);
        console_writeln(&m);
        Win32ApiEmulator::def_window_proc(mgr, hwnd, msg, wparam, lparam)
    }
    pub fn post_quit_message(mgr: &mut Win32Manager, exit_code: u32) {
        let m = format!("[USER32] PostQuitMessage({})", exit_code);
        console_writeln(&m);
        mgr.window_manager.post_message(0, WinMsg::Quit, exit_code as u64, 0);
    }
    pub fn raise_exception(mgr: &mut Win32Manager, code: u32, flags: u32, params: &[u64]) -> u64 {
        let mut record = ExceptionRecord::new(code, 0x401000);
        record.exception_flags = flags;
        record.number_parameters = params.len() as u32;
        for (i, &p) in params.iter().enumerate() {
            if i < 15 { record.exception_information[i] = p; }
        }
        let mut context = ContextRecord::new();
        let result = mgr.seh.raise_exception(&record, &mut context);
        let m = format!("[KERNEL32] RaiseException(code=0x{:08X}, flags={}) -> {:?}", code, flags, result);
        console_writeln(&m);
        match result {
            ExceptionDisposition::ExceptionContinueExecution => 0,
            _ => 1,
        }
    }
    pub fn set_unhandled_exception_filter(mgr: &mut Win32Manager, filter: ExceptionHandlerFn) {
        mgr.seh.set_unhandled_filter(filter);
        let m = String::from("[KERNEL32] SetUnhandledExceptionFilter -> ok");
        console_writeln(&m);
    }
    pub fn get_dc(mgr: &mut Win32Manager, hwnd: u64) -> u64 {
        let hdc = mgr.next_hdc; mgr.next_hdc += 1;
        mgr.hdc_table.insert(hdc, HdcEntry::new(hwnd));
        let msg = format!("[GDI] GetDC(hwnd=0x{:08X}) -> hdc=0x{:08X}", hwnd, hdc);
        console_writeln(&msg); hdc
    }
    pub fn release_dc(mgr: &mut Win32Manager, hdc: u64) -> bool {
        let ok = mgr.hdc_table.remove(&hdc).is_some();
        let msg = format!("[GDI] ReleaseDC(hdc=0x{:08X}) -> {}", hdc, ok);
        console_writeln(&msg); ok
    }
    pub fn select_object(mgr: &mut Win32Manager, hdc: u64, obj: u64) -> u64 {
        if let Some(entry) = mgr.hdc_table.get_mut(&hdc) {
            if let Some(gobj) = GdiManager::get_object(obj, &mgr.gdi) {
                let prev = match gobj.obj_type {
                    GdiObjectType::Pen => { let p = entry.selected_pen; entry.selected_pen = obj; p }
                    GdiObjectType::Brush => { let p = entry.selected_brush; entry.selected_brush = obj; p }
                    GdiObjectType::Font => { let p = entry.selected_font; entry.selected_font = obj; p }
                    GdiObjectType::Bitmap => { let p = entry.selected_bitmap; entry.selected_bitmap = obj; p }
                    _ => 0,
                };
                let msg = format!("[GDI] SelectObject(hdc=0x{:08X}, obj=0x{:08X}) -> prev=0x{:08X}", hdc, obj, prev);
                console_writeln(&msg); return prev;
            }
        }
        let msg = format!("[GDI] SelectObject(hdc=0x{:08X}, obj=0x{:08X}) -> 0 (invalid)", hdc, obj);
        console_writeln(&msg); 0
    }
    pub fn create_pen(mgr: &mut Win32Manager, width: i32, color: u32) -> u64 {
        GdiManager::create_pen(width, color, &mut mgr.gdi)
    }
    pub fn create_solid_brush(mgr: &mut Win32Manager, color: u32) -> u64 {
        GdiManager::create_solid_brush(color, &mut mgr.gdi)
    }
    pub fn create_bitmap(mgr: &mut Win32Manager, width: i32, height: i32) -> u64 {
        GdiManager::create_bitmap(width, height, &mut mgr.gdi)
    }
    pub fn delete_object(mgr: &mut Win32Manager, obj: u64) -> bool {
        // Deselect if currently selected in any HDC
        for entry in mgr.hdc_table.values_mut() {
            if entry.selected_pen == obj { entry.selected_pen = 0; }
            if entry.selected_brush == obj { entry.selected_brush = 0; }
            if entry.selected_font == obj { entry.selected_font = 0; }
            if entry.selected_bitmap == obj { entry.selected_bitmap = 0; }
        }
        GdiManager::delete_object(obj, &mut mgr.gdi)
    }
    pub fn bit_blt(mgr: &mut Win32Manager, hdc: u64, x: i32, y: i32, w: i32, h: i32, _src: u64, _sx: i32, _sy: i32, _rop: u32) -> bool {
        if let Some(entry) = mgr.hdc_table.get(&hdc) {
            if let Some(win) = mgr.window_manager.get_window(entry.hwnd) {
                let abs_x = win.x + x;
                let abs_y = win.y + y;
                mgr.gfx_fill_rect(abs_x, abs_y, w, h, CLR_WIN_BG);
                let msg = format!("[GDI] BitBlt(hdc=0x{:08X}, {}, {}, {}, {}) -> ok", hdc, x, y, w, h);
                console_writeln(&msg); return true;
            }
        }
        let msg = format!("[GDI] BitBlt(hdc=0x{:08X}) -> invalid hdc", hdc);
        console_writeln(&msg); false
    }
    pub fn rectangle(mgr: &mut Win32Manager, hdc: u64, left: i32, top: i32, right: i32, bottom: i32) -> bool {
        if let Some(entry) = mgr.hdc_table.get(&hdc) {
            if let Some(win) = mgr.window_manager.get_window(entry.hwnd) {
                let abs_x = win.x + left;
                let abs_y = win.y + top;
                let color = if let Some(gobj) = GdiManager::get_object(entry.selected_brush, &mgr.gdi) { gobj.color } else { CLR_ACCENT };
                mgr.gfx_fill_rect(abs_x, abs_y, right - left, bottom - top, color);
                let msg = format!("[GDI] Rectangle(hdc=0x{:08X}, {}, {}, {}, {}) -> ok", hdc, left, top, right, bottom);
                console_writeln(&msg); return true;
            }
        }
        let msg = format!("[GDI] Rectangle(hdc=0x{:08X}) -> invalid hdc", hdc);
        console_writeln(&msg); false
    }
    pub fn text_out_a(mgr: &mut Win32Manager, hdc: u64, x: i32, y: i32, text: &str) -> bool {
        if let Some(entry) = mgr.hdc_table.get(&hdc) {
            if let Some(win) = mgr.window_manager.get_window(entry.hwnd) {
                let abs_x = win.x + x;
                let abs_y = win.y + y;
                let color = if let Some(gobj) = GdiManager::get_object(entry.selected_brush, &mgr.gdi) { gobj.color } else { CLR_TEXT };
                mgr.gfx_draw_text(abs_x, abs_y, text, color);
                let msg = format!("[GDI] TextOutA(hdc=0x{:08X}, \"{}\", {}, {}) -> ok", hdc, text, x, y);
                console_writeln(&msg); return true;
            }
        }
        let msg = format!("[GDI] TextOutA(hdc=0x{:08X}) -> invalid hdc", hdc);
        console_writeln(&msg); false
    }
    pub fn get_file_attributes(mgr: &Win32Manager, path: &str) -> u32 {
        let attr = mgr.vfs_get_attr(path);
        let msg = format!("[KERNEL32] GetFileAttributes(\"{}\") -> 0x{:08X}", path, attr);
        console_writeln(&msg); attr
    }
    pub fn set_file_attributes(_mgr: &mut Win32Manager, _path: &str, _attrs: u32) -> bool {
        true // stub: attributes not persisted beyond VFS
    }
    pub fn create_directory(mgr: &mut Win32Manager, path: &str) -> bool {
        mgr.vfs_create_dir(path)
    }
    pub fn remove_directory(mgr: &mut Win32Manager, path: &str) -> bool {
        mgr.vfs_remove_dir(path)
    }
    pub fn copy_file(mgr: &mut Win32Manager, src: &str, dst: &str) -> bool {
        mgr.vfs_copy_file(src, dst)
    }
    pub fn move_file(mgr: &mut Win32Manager, src: &str, dst: &str) -> bool {
        mgr.vfs_move_file(src, dst)
    }
    pub fn delete_file(mgr: &mut Win32Manager, path: &str) -> bool {
        mgr.vfs_delete_file(path)
    }
    pub fn get_system_info() -> SystemInfo {
        let msg = String::from("[KERNEL32] GetSystemInfo()");
        console_writeln(&msg);
        SystemInfo {
            processor_architecture: 9, page_size: 4096,
            minimum_application_address: 0x10000, maximum_application_address: 0x7FFFFFFFFFFF,
            active_processor_mask: 1, number_of_processors: 1, processor_type: 8664,
            allocation_granularity: 0x10000, processor_level: 6, processor_revision: 0,
        }
    }
    pub fn get_version_ex() -> OsVersionInfo {
        let msg = String::from("[KERNEL32] GetVersionEx() -> 10.0.19045");
        console_writeln(&msg);
        OsVersionInfo { major_version: 10, minor_version: 0, build_number: 19045, platform_id: 2, csd_version: String::from("") }
    }
    pub fn get_computer_name() -> String {
        let msg = String::from("[KERNEL32] GetComputerName() -> OZKAN-PC");
        console_writeln(&msg); String::from("OZKAN-PC")
    }
    pub fn get_user_name() -> String {
        let msg = String::from("[KERNEL32] GetUserName() -> User");
        console_writeln(&msg); String::from("User")
    }
    pub fn get_tick_count() -> u32 {
        let msg = String::from("[KERNEL32] GetTickCount()");
        console_writeln(&msg); 0x1234
    }
    pub fn create_pipe(mgr: &mut Win32Manager, read_size: u32, write_size: u32) -> (u64, u64) {
        let (r, w) = mgr.pipes.create_pipe(read_size, write_size);
        let msg = format!("[KERNEL32] CreatePipe() -> read=0x{:08X}, write=0x{:08X}", r, w);
        console_writeln(&msg); (r, w)
    }
    pub fn create_named_pipe(mgr: &mut Win32Manager, name: &str, open_mode: u32, pipe_mode: u32, max_instances: u32, out_buf: u32, in_buf: u32, timeout: u32) -> u64 {
        let h = mgr.pipes.create_named_pipe(name, open_mode, pipe_mode, max_instances, out_buf, in_buf, timeout);
        let msg = format!("[KERNEL32] CreateNamedPipe(\"{}\") -> 0x{:08X}", name, h);
        console_writeln(&msg); h
    }
    pub fn connect_named_pipe(mgr: &mut Win32Manager, handle: u64) -> bool {
        let ok = mgr.pipes.connect_named_pipe(handle);
        let msg = format!("[KERNEL32] ConnectNamedPipe(0x{:08X}) -> {}", handle, ok);
        console_writeln(&msg); ok
    }
    pub fn write_pipe(mgr: &mut Win32Manager, handle: u64, data: &[u8]) -> u32 {
        let n = mgr.pipes.write_pipe(handle, data);
        let msg = format!("[KERNEL32] WriteFile(pipe=0x{:08X}, {} bytes)", handle, n);
        console_writeln(&msg); n
    }
    pub fn read_pipe(mgr: &mut Win32Manager, handle: u64, buf: &mut [u8]) -> u32 {
        let n = mgr.pipes.read_pipe(handle, buf);
        let msg = format!("[KERNEL32] ReadFile(pipe=0x{:08X}, {} bytes)", handle, n);
        console_writeln(&msg); n
    }
    pub fn close_pipe(mgr: &mut Win32Manager, handle: u64) -> bool {
        mgr.pipes.close_pipe(handle)
    }
    pub fn create_mailslot(mgr: &mut Win32Manager, name: &str, max_msg: u32, timeout: u32) -> u64 {
        let h = mgr.mailslots.create_mailslot(name, max_msg, timeout);
        let msg = format!("[KERNEL32] CreateMailslot(\"{}\") -> 0x{:08X}", name, h);
        console_writeln(&msg); h
    }
    pub fn get_mailslot_info(mgr: &mut Win32Manager, handle: u64, max_msg: u64, next_size: u64, msg_count: u64, read_timeout: u64) -> bool {
        mgr.mailslots.get_mailslot_info(handle, max_msg, next_size, msg_count, read_timeout)
    }
    pub fn write_mailslot(mgr: &mut Win32Manager, name: &str, data: &[u8]) -> bool {
        mgr.mailslots.write_mailslot(name, data)
    }
    pub fn read_mailslot(mgr: &mut Win32Manager, handle: u64, buf: &mut [u8]) -> u32 {
        mgr.mailslots.read_mailslot(handle, buf)
    }
    pub fn get_disk_free_space(_path: &str) -> (u32, u32, u32, u32) {
        let msg = format!("[KERNEL32] GetDiskFreeSpace(\"{}\") -> sectors=64, bytes=512, free=100000, total=200000", _path);
        console_writeln(&msg); (64, 512, 100000, 200000)
    }
    pub fn get_drive_type(path: &str) -> u32 {
        let t = if path.starts_with("C:") { 3 } else if path.starts_with("A:") || path.starts_with("B:") { 2 } else if path.starts_with("D:") { 5 } else { 1 };
        let msg = format!("[KERNEL32] GetDriveType(\"{}\") -> {}", path, t);
        console_writeln(&msg); t
    }
    pub fn get_logical_drives() -> u32 {
        let msg = String::from("[KERNEL32] GetLogicalDrives() -> 0x1F");
        console_writeln(&msg); 0x1F
    }
    pub fn global_alloc(mgr: &mut Win32Manager, _flags: u32, size: usize) -> u64 {
        let h = mgr.next_global_handle; mgr.next_global_handle += 1;
        mgr.global_mem.insert(h, vec![0; size]);
        let msg = format!("[KERNEL32] GlobalAlloc(size={}) -> 0x{:08X}", size, h);
        console_writeln(&msg); h
    }
    pub fn global_free(mgr: &mut Win32Manager, handle: u64) -> bool {
        let ok = mgr.global_mem.remove(&handle).is_some();
        let msg = format!("[KERNEL32] GlobalFree(0x{:08X}) -> {}", handle, ok);
        console_writeln(&msg); ok
    }
    pub fn local_alloc(mgr: &mut Win32Manager, _flags: u32, size: usize) -> u64 {
        let h = mgr.next_local_handle; mgr.next_local_handle += 1;
        mgr.local_mem.insert(h, vec![0; size]);
        let msg = format!("[KERNEL32] LocalAlloc(size={}) -> 0x{:08X}", size, h);
        console_writeln(&msg); h
    }
    pub fn local_free(mgr: &mut Win32Manager, handle: u64) -> bool {
        let ok = mgr.local_mem.remove(&handle).is_some();
        let msg = format!("[KERNEL32] LocalFree(0x{:08X}) -> {}", handle, ok);
        console_writeln(&msg); ok
    }
    pub fn load_string(mgr: &mut Win32Manager, id: u32, text: &str) -> u64 {
        mgr.strings.insert(id, String::from(text));
        let msg = format!("[USER32] LoadString(id={}) -> \"{}\"", id, text);
        console_writeln(&msg); id as u64
    }
    pub fn load_icon(_mgr: &mut Win32Manager, name: &str) -> u64 {
        let msg = format!("[USER32] LoadIcon(\"{}\") -> 0x10001", name);
        console_writeln(&msg); 0x10001
    }
    pub fn load_cursor(_mgr: &mut Win32Manager, name: &str) -> u64 {
        let msg = format!("[USER32] LoadCursor(\"{}\") -> 0x10002", name);
        console_writeln(&msg); 0x10002
    }
    pub fn get_environment_variable(mgr: &Win32Manager, name: &str) -> String {
        let val = mgr.env.get(name).cloned().unwrap_or_default();
        let msg = format!("[KERNEL32] GetEnvironmentVariable(\"{}\") -> \"{}\"", name, val);
        console_writeln(&msg); val
    }
    pub fn set_environment_variable(mgr: &mut Win32Manager, name: &str, value: &str) -> bool {
        mgr.env.insert(String::from(name), String::from(value));
        let msg = format!("[KERNEL32] SetEnvironmentVariable(\"{}\", \"{}\")", name, value);
        console_writeln(&msg); true
    }
    pub fn expand_environment_strings(mgr: &Win32Manager, src: &str) -> String {
        let mut out = String::from(src);
        for (k, v) in &mgr.env {
            let pat = format!("%{}%", k);
            out = out.replace(&pat, v);
        }
        let msg = format!("[KERNEL32] ExpandEnvironmentStrings(\"{}\") -> \"{}\"", src, out);
        console_writeln(&msg); out
    }
    pub fn get_windows_directory() -> String {
        let msg = String::from("[KERNEL32] GetWindowsDirectory() -> C:\\WINDOWS");
        console_writeln(&msg); String::from("C:\\WINDOWS")
    }
    pub fn get_system_directory() -> String {
        let msg = String::from("[KERNEL32] GetSystemDirectory() -> C:\\WINDOWS\\SYSTEM32");
        console_writeln(&msg); String::from("C:\\WINDOWS\\SYSTEM32")
    }
    pub fn get_temp_path() -> String {
        let msg = String::from("[KERNEL32] GetTempPath() -> C:\\TEMP");
        console_writeln(&msg); String::from("C:\\TEMP")
    }
    pub fn get_system_time() -> (u16, u16, u16, u16, u16, u16, u16, u16) {
        let msg = String::from("[KERNEL32] GetSystemTime()");
        console_writeln(&msg); (2026, 4, 27, 4, 19, 30, 0, 0)
    }
}

#[cfg(test)]
#[path = "api_emulator_impl.rs"]
mod api_emulator_impl;
