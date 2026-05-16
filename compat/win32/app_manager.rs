// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Uygulama Yöneticisi
// Dosya Yolu         : apps/system/compat/win32/app_manager.rs
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
//   Win32 uygulama yöneticisi, çalıştırma ve VFS yardımcıları.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

use crate::dos_emulator::console_writeln;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use super::pe_loader::*;
use super::process_mgr::{PeLoader, MappedImage};
use super::registry::{WinProcess, RegistryKey};
use crate::win_layer::window_core::{WindowManager, WindowClass, WindowMessage};
use crate::win_layer::syscall_core::SyscallEmulator;
use crate::win_layer::registry_engine::RegistryHiveParser;
use crate::win_layer::console_core::ConsoleBuffer;
use crate::win_layer::gdi_core::{GdiManager, HdcEntry};
use crate::win_layer::scm_core::ServiceControlManager;
use crate::win_layer::seh_core::SehManager;
use crate::win_layer::pipe_core::PipeManager;
use crate::win_layer::mailslot_core::MailslotManager;
use crate::win_layer::shared_defs::{Win32Section, RegistryValue, WinMsg, ServiceStatus};
use alloc::collections::BTreeSet;
use super::syscall_emu::process_mgr_extra::{FileSearchManager, ThreadScheduler, FindFileEntry, FileSearchHandle};

// ─── Win32 Application Manager ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct WinAppDyn {
    pub name: String, pub running: bool, pub security: i32, pub pe_info: Option<PeParserResult>,
}

impl WinAppDyn {
    pub fn new(name: &str, running: bool, security: i32) -> Self { Self { name: String::from(name), running, security, pe_info: None } }
    pub fn status_label(&self) -> String {
        if self.running { String::from("[WIN32]") } else { String::from("[WIN32]") }
    }
}

#[derive(Debug, Clone)]
pub struct Win32FileHandle {
    pub name: String, pub data: Vec<u8>, pub pos: usize, pub access: u32, pub share: u32,
}

#[derive(Debug, Clone)]
pub struct Win32Module {
    pub name: String, pub base: u64, pub image: MappedImage, pub exports: BTreeMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct Win32Event {
    pub signaled: bool, pub manual_reset: bool,
}

#[derive(Debug, Clone)]
pub struct Win32Mutex {
    pub owned: bool, pub thread_id: u32,
}

pub struct Win32Manager {
    pub apps: Vec<WinAppDyn>, pub registry: BTreeMap<String, RegistryKey>,
    pub processes: Vec<WinProcess>, pub next_pid: u32,
    pub pe_parser: PeParser, pub window_manager: WindowManager,
    pub syscall_emulator: SyscallEmulator, pub hive_parser: RegistryHiveParser,
    pub files: BTreeMap<u64, Win32FileHandle>, pub modules: BTreeMap<u64, Win32Module>,
    pub next_handle: u64, pub next_module_base: u64,
    pub vfs: BTreeMap<String, Vec<u8>>,
    pub vfs_dirs: BTreeSet<String>,
    pub events: BTreeMap<u64, Win32Event>,
    pub mutexes: BTreeMap<u64, Win32Mutex>,
    pub last_error: u32,
    pub current_process: u32,
    pub current_thread: u32,
    pub console: ConsoleBuffer,
    pub scm: ServiceControlManager,
    pub hdc_table: BTreeMap<u64, HdcEntry>,
    pub next_hdc: u64,
    pub gdi: GdiManager,
    pub file_search: FileSearchManager,
    pub sections: BTreeMap<u64, Win32Section>,
    pub seh: SehManager,
    pub pipes: PipeManager,
    pub mailslots: MailslotManager,
    pub env: BTreeMap<String, String>,
    pub scheduler: ThreadScheduler,
    pub global_mem: BTreeMap<u64, Vec<u8>>,
    pub local_mem: BTreeMap<u64, Vec<u8>>,
    pub strings: BTreeMap<u32, String>,
    pub next_global_handle: u64,
    pub next_local_handle: u64,
}

impl Default for Win32Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Win32Manager {
    pub fn new() -> Self {
        let mut mgr = Self {
            apps: Vec::new(), registry: BTreeMap::new(), processes: Vec::new(),
            next_pid: 1000, pe_parser: PeParser, window_manager: WindowManager::new(),
            syscall_emulator: SyscallEmulator, hive_parser: RegistryHiveParser,
            files: BTreeMap::new(), modules: BTreeMap::new(), next_handle: 0x40000000, next_module_base: 0x10000000,
            vfs: BTreeMap::new(), vfs_dirs: BTreeSet::new(), events: BTreeMap::new(), mutexes: BTreeMap::new(),
            last_error: 0, current_process: 0, current_thread: 0,
            console: ConsoleBuffer::new(),
            scm: ServiceControlManager::new(),
            hdc_table: BTreeMap::new(), next_hdc: 0x1000,
            gdi: GdiManager::new(),
            file_search: FileSearchManager::new(),
            sections: BTreeMap::new(),
            seh: SehManager::new(),
            pipes: PipeManager::new(),
            mailslots: MailslotManager::new(),
            env: BTreeMap::new(),
            scheduler: ThreadScheduler::new(),
            global_mem: BTreeMap::new(),
            local_mem: BTreeMap::new(),
            strings: BTreeMap::new(),
            next_global_handle: 0x8000,
            next_local_handle: 0x9000,
        };
        mgr.env.insert(String::from("PATH"), String::from("C:\\WINDOWS;C:\\WINDOWS\\SYSTEM32"));
        mgr.env.insert(String::from("SYSTEMROOT"), String::from("C:\\WINDOWS"));
        mgr.env.insert(String::from("TEMP"), String::from("C:\\TEMP"));
        mgr.env.insert(String::from("WINDIR"), String::from("C:\\WINDOWS"));
        mgr.env.insert(String::from("USERNAME"), String::from("User"));
        mgr.env.insert(String::from("COMPUTERNAME"), String::from("OZKAN-PC"));
        mgr.apps.push(WinAppDyn::new("NOTEPAD.EXE", false, 2));
        mgr.apps.push(WinAppDyn::new("CALC.EXE", true, 1));
        mgr.apps.push(WinAppDyn::new("SOLITAIRE.EXE", false, 0));
        mgr.apps.push(WinAppDyn::new("CMD.EXE", false, 2));
        let hklm = mgr.reg_create_key("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion");
        hklm.set_value("CurrentVersion", RegistryValue::String(String::from("1.0")));
        hklm.set_value("BuildNumber", RegistryValue::Dword(9600));
        hklm.set_value("ProgramFilesDir", RegistryValue::String(String::from("C:\\Program Files")));
        mgr.reg_create_key("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
        mgr.reg_create_key("HKLM\\SYSTEM\\CurrentControlSet\\Control\\Session Manager");
        mgr.reg_create_key("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer");
        mgr.window_manager.register_class(WindowClass::new("Button"));
        mgr.window_manager.register_class(WindowClass::new("Edit"));
        mgr.window_manager.register_class(WindowClass::new("Static"));
        mgr.window_manager.register_class(WindowClass::new("ListBox"));
        mgr.window_manager.register_class(WindowClass::new("ComboBox"));
        mgr.window_manager.register_class(WindowClass::new("ScrollBar"));
        mgr
    }
    pub fn reg_create_key(&mut self, path: &str) -> &mut RegistryKey {
        let key = RegistryKey::new(path);
        self.registry.insert(String::from(path), key);
        self.registry.get_mut(path).unwrap()
    }
    pub fn reg_query_value(&self, path: &str, value_name: &str) -> Option<&RegistryValue> {
        self.registry.get(path)?.get_value(value_name)
    }
    pub fn create_process(&mut self, name: &str) -> u32 {
        let pid = self.next_pid; self.next_pid += 1;
        let mut proc = WinProcess::new(pid, name);
        proc.image_path = format!("C:\\Windows\\System32\\{}", name);
        proc.create_thread(0x401000);
        self.processes.push(proc);
        let msg = format!("{}: {} (PID {})", "[WIN32]", name, pid);
        console_writeln(&msg); pid
    }
    pub fn terminate_process(&mut self, pid: u32) -> bool {
        if let Some(idx) = self.processes.iter().position(|p| p.pid == pid) {
            self.processes[idx].exit_code = 0; self.processes.remove(idx);
            let msg = format!("{}: PID {}", "[WIN32]", pid);
            console_writeln(&msg); true
        } else { false }
    }
    pub fn enum_processes(&self) -> &[WinProcess] { &self.processes }
    #[must_use]
    pub fn find_process(&self, pid: u32) -> Option<&WinProcess> { self.processes.iter().find(|p| p.pid == pid) }
    pub fn find_process_mut(&mut self, pid: u32) -> Option<&mut WinProcess> { self.processes.iter_mut().find(|p| p.pid == pid) }
    #[must_use]
    pub fn load_pe_sections(&self, data: &[u8]) -> Vec<ImageSectionHeader> {
        let result = PeParser::parse(data);
        if result.is_valid {
            let msg = format!("{}: {} section(s)", "[WIN32]", result.sections.len());
            console_writeln(&msg);
            result.sections
        } else {
            for err in &result.errors {
                let msg = format!("{}: {}", "[WIN32]", err);
                console_writeln(&msg);
            }
            Vec::new()
        }
    }

    /// Load a full PE image into memory, apply relocations, resolve imports,
    /// create a process and a thread at the entry point.
    #[must_use]
    pub fn execute_pe(&mut self, data: &[u8]) -> i32 {
        let result = PeParser::parse(data);
        if !result.is_valid {
            for err in &result.errors {
                let msg = format!("{}: {}", "[WIN32]", err);
                console_writeln(&msg);
            }
            return -1;
        }
        let base = self.next_module_base;
        let mut mapped = PeLoader::map_image(data, &result, base);
        PeLoader::apply_relocations(&mut mapped, &result, base);
        // Resolve imports with deterministic hash-based addresses
        for imp in &result.imports {
            let addr = self.resolve_api(&imp.dll_name, &imp.func_name);
            mapped.imports_resolved.push((String::from(&imp.dll_name), String::from(&imp.func_name), addr));
        }
        let entry = mapped.get_entry_point(result.optional_header.address_of_entry_point);
        let pid = self.create_process("PE_APP.EXE");
        let mut tid = 0;
        if let Some(proc) = self.find_process_mut(pid) {
            proc.base_address = base;
            proc.image_size = mapped.image_size;
            tid = proc.create_thread(entry);
        }
        if tid != 0 {
            self.scheduler.add_thread(tid);
        }
        self.next_module_base += mapped.image_size;
        self.modules.insert(base, Win32Module {
            name: String::from("PE_APP.EXE"),
            base,
            image: mapped,
            exports: BTreeMap::new(),
        });
        let msg = format!("{} PE executed PID={} entry=0x{:016X}", "[WIN32]", pid, entry);
        console_writeln(&msg);
        pid as i32
    }

    /// Generate a deterministic pseudo-address for a Win32 API import.
    /// Runtime dispatch uses this address as a unique function ID.
    fn resolve_api(&self, dll: &str, func: &str) -> u64 {
        let key = format!("{}!{}", dll.to_lowercase(), func);
        let mut hash: u64 = 0x7FFF_0000_0000;
        for b in key.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as u64);
        }
        hash
    }
    pub fn parse_pe_file(&self, data: &[u8]) -> PeParserResult { PeParser::parse(data) }
    pub fn post_message(&mut self, hwnd: u64, msg: WinMsg, wparam: u64, lparam: u64) { self.window_manager.post_message(hwnd, msg, wparam, lparam); }
    pub fn peek_message(&mut self, hwnd: u64) -> Option<WindowMessage> { self.window_manager.peek_message(hwnd) }
    pub fn create_window(&mut self, class_name: &str, title: &str, x: i32, y: i32, width: i32, height: i32) -> u64 { self.window_manager.create_window(class_name, title, x, y, width, height) }
    pub fn gfx_fill_rect(&self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        unsafe { gfx_fill_rect(x, y, w, h, color); }
    }
    pub fn gfx_draw_text(&self, x: i32, y: i32, text: &str, color: u32) {
        let buf = format!("{}\0", text);
        unsafe { gfx_draw_text(x, y, buf.as_ptr(), text.len() as u32, color); }
    }
    pub fn gfx_swap_buffers(&self) {
        unsafe { gfx_swap_buffers(); }
    }
    pub fn draw_all_windows(&mut self) {
        self.window_manager.draw_all();
    }
    pub fn show_window(&mut self, hwnd: u64, cmd: i32) -> bool { self.window_manager.show_window(hwnd, cmd) }
    pub fn scm_create_service(&mut self, name: &str, display: &str, start_type: u32, path: &str) -> u64 {
        self.scm.create_service(self.scm.db_handle, name, display, 0xF01FF, 0x10, start_type, 1, path, "")
    }
    pub fn scm_delete_service(&mut self, handle: u64) -> bool { self.scm.delete_service(handle) }
    pub fn scm_start_service(&mut self, handle: u64) -> bool { self.scm.start_service(handle, &[]) }
    pub fn scm_stop_service(&mut self, handle: u64) -> bool { self.scm.stop_service(handle) }
    pub fn scm_control_service(&mut self, handle: u64, control: u32) -> bool { self.scm.control_service(handle, control) }
    pub fn scm_query_status(&self, handle: u64) -> ServiceStatus { self.scm.query_service_status(handle) }
    pub fn scm_enum_services(&self) { self.scm.enum_services(); }
    pub fn scheduler_tick(&mut self) -> Option<u32> {
        let tid = self.scheduler.tick(&mut self.processes);
        if let Some(t) = tid {
            let msg = format!("[SCHED] Tick -> TID={}", t);
            console_writeln(&msg);
        }
        tid
    }
    pub fn scheduler_yield(&mut self) -> Option<u32> {
        let tid = self.scheduler.yield_current(&mut self.processes);
        if let Some(t) = tid {
            let msg = format!("[SCHED] Yield -> TID={}", t);
            console_writeln(&msg);
        }
        tid
    }
    pub fn scheduler_add_thread(&mut self, tid: u32) {
        self.scheduler.add_thread(tid);
    }

    pub fn find_first_file(&mut self, pattern: &str) -> u64 {
        let h = self.file_search.next_handle; self.file_search.next_handle += 1;
        let mut entries = Vec::new();
        entries.push(FindFileEntry { name: String::from("."), attrs: 0x10, size: 0, creation: 0, access: 0, write: 0 });
        entries.push(FindFileEntry { name: String::from(".."), attrs: 0x10, size: 0, creation: 0, access: 0, write: 0 });
        let (dir, wild) = split_find_pattern(pattern);
        let items = self.vfs_list_dir(&dir);
        for name in items {
            if wildcard_match(&name, &wild) {
                let full = if dir.is_empty() { name.clone() } else { format!("{}\\{}", dir, name) };
                let attrs = self.vfs_get_attr(&full);
                let size = if attrs != 0x10 { self.vfs.get(&full.to_ascii_uppercase()).map(|v| v.len() as u64).unwrap_or(0) } else { 0 };
                entries.push(FindFileEntry { name, attrs, size, creation: 0, access: 0, write: 0 });
            }
        }
        self.file_search.handles.push(FileSearchHandle { handle: h, pattern: String::from(pattern), entries, index: 0 });
        let msg = format!("{} FindFirstFile(\"{}\") -> 0x{:08X} ({} entries)", "[WIN32]", pattern, h, self.file_search.handles.last().unwrap().entries.len());
        console_writeln(&msg); h
    }
    pub fn find_next_file(&mut self, handle: u64, entry: &mut FindFileEntry) -> bool {
        self.file_search.find_next_file(handle, entry)
    }
    pub fn find_close(&mut self, handle: u64) -> bool {
        self.file_search.find_close(handle)
    }

    // ─── VFS Path Helpers ─────────────────────────────────────

    pub fn vfs_create_dir(&mut self, path: &str) -> bool {
        let norm = path.trim_end_matches('\\').to_ascii_uppercase();
        if self.vfs.contains_key(&norm) || self.vfs_dirs.contains(&norm) { return false; }
        self.vfs_dirs.insert(norm);
        let msg = format!("[VFS] CreateDirectory(\"{}\") -> true", path);
        console_writeln(&msg); true
    }
    pub fn vfs_remove_dir(&mut self, path: &str) -> bool {
        let norm = path.trim_end_matches('\\').to_ascii_uppercase();
        if !self.vfs_dirs.contains(&norm) { return false; }
        let prefix = format!("{}\\", norm);
        for k in self.vfs.keys() { if k.starts_with(&prefix) { return false; } }
        self.vfs_dirs.remove(&norm);
        let msg = format!("[VFS] RemoveDirectory(\"{}\") -> true", path);
        console_writeln(&msg); true
    }
    pub fn vfs_is_dir(&self, path: &str) -> bool {
        let norm = path.trim_end_matches('\\').to_ascii_uppercase();
        self.vfs_dirs.contains(&norm)
    }
    pub fn vfs_delete_file(&mut self, path: &str) -> bool {
        let norm = path.to_ascii_uppercase();
        if self.vfs.remove(&norm).is_some() {
            let msg = format!("[VFS] DeleteFile(\"{}\") -> true", path);
            console_writeln(&msg); true
        } else {
            let msg = format!("[VFS] DeleteFile(\"{}\") -> false (not found)", path);
            console_writeln(&msg); false
        }
    }
    pub fn vfs_copy_file(&mut self, src: &str, dst: &str) -> bool {
        let src_norm = src.to_ascii_uppercase();
        if let Some(data) = self.vfs.get(&src_norm).cloned() {
            self.vfs.insert(dst.to_ascii_uppercase(), data);
            let msg = format!("[VFS] CopyFile(\"{}\", \"{}\") -> true", src, dst);
            console_writeln(&msg); true
        } else {
            let msg = format!("[VFS] CopyFile(\"{}\", \"{}\") -> false (src not found)", src, dst);
            console_writeln(&msg); false
        }
    }
    pub fn vfs_move_file(&mut self, src: &str, dst: &str) -> bool {
        let src_norm = src.to_ascii_uppercase();
        if let Some(data) = self.vfs.remove(&src_norm) {
            self.vfs.insert(dst.to_ascii_uppercase(), data);
            let msg = format!("[VFS] MoveFile(\"{}\", \"{}\") -> true", src, dst);
            console_writeln(&msg); true
        } else {
            let msg = format!("[VFS] MoveFile(\"{}\", \"{}\") -> false (src not found)", src, dst);
            console_writeln(&msg); false
        }
    }
    pub fn vfs_get_attr(&self, path: &str) -> u32 {
        let norm = path.trim_end_matches('\\').to_ascii_uppercase();
        if self.vfs_dirs.contains(&norm) { return 0x10; } // FILE_ATTRIBUTE_DIRECTORY
        if self.vfs.contains_key(&norm) { return 0x80; } // FILE_ATTRIBUTE_NORMAL
        0xFFFFFFFF // INVALID_FILE_ATTRIBUTES
    }
    pub fn vfs_list_dir(&self, path: &str) -> Vec<String> {
        let norm = path.trim_end_matches('\\').to_ascii_uppercase();
        let prefix = if norm.is_empty() { String::new() } else { format!("{}\\", norm) };
        let mut out = Vec::new();
        for d in &self.vfs_dirs {
            if d.starts_with(&prefix) && d.len() > prefix.len() {
                let rest = &d[prefix.len()..];
                if !rest.contains('\\') && !out.contains(&String::from(rest)) {
                    out.push(String::from(rest));
                }
            }
        }
        for f in self.vfs.keys() {
            if f.starts_with(&prefix) && f.len() > prefix.len() {
                let rest = &f[prefix.len()..];
                if !rest.contains('\\') && !out.contains(&String::from(rest)) {
                    out.push(String::from(rest));
                }
            }
        }
        out
    }
}

fn security_level_label(level: i32) -> String {
    match level {
        0 => String::from("[WIN32]"),
        1 => String::from("[WIN32]"),
        _ => String::from("[WIN32]"),
    }
}

fn split_find_pattern(pattern: &str) -> (String, String) {
    let p = pattern.to_ascii_uppercase();
    if let Some(idx) = p.rfind('\\') {
        (String::from(&p[..idx]), String::from(&p[idx + 1..]))
    } else {
        (String::new(), p)
    }
}

fn wildcard_match(name: &str, pattern: &str) -> bool {
    let mut name_chars = name.chars().peekable();
    let mut pat_chars = pattern.chars().peekable();
    while let Some(pc) = pat_chars.next() {
        match pc {
            '*' => {
                let next = pat_chars.peek().copied();
                if next.is_none() { return true; }
                for nc in name_chars.by_ref() {
                    if Some(nc) == next {
                        break;
                    }
                }
            }
            '?' => { name_chars.next(); }
            c => {
                if name_chars.next() != Some(c) { return false; }
            }
        }
    }
    name_chars.next().is_none()
}

// ─── Helper Functions ─────────────────────────────────────

fn draw_str(x: i32, y: i32, s: &str, color: u32) {
    if s.is_empty() { return; }
    unsafe { gfx_draw_text(x, y, s.as_ptr(), s.len() as u32, color); }
}

fn draw_app_row(x: i32, y: i32, apps: &[WinAppDyn], idx: usize) {
    if idx >= apps.len() { return; }
    let app = &apps[idx];
    let bg = if idx.is_multiple_of(2) { CLR_ITEM_BG } else { CLR_WIN_BG };
    unsafe { gfx_fill_rect(x, y, WIN_W - 180, 45, bg); }
    unsafe { gfx_fill_rect(x + 10, y + 8, 30, 30, CLR_ACCENT); }
    draw_str(x + 18, y + 15, "W", CLR_TEXT);
    draw_str(x + 50, y + 5, &app.name, CLR_TEXT);
    let stat_col = if app.running { CLR_SUCCESS } else { CLR_TEXT };
    draw_str(x + 50, y + 25, &app.status_label(), stat_col);
    let lev_col = if app.security == 2 { CLR_WARNING } else { CLR_ACCENT };
    draw_str(x + WIN_W - 280, y + 15, &security_level_label(app.security), lev_col);
}

// ─── Main Functions ──────────────────────────────────────────

pub fn gui_main(manager: &Win32Manager) {
    let title = "[WIN32]";
    let mut title_buf: [u8; 64] = [0u8; 64];
    let tb = title.as_bytes();
    let tlen = tb.len().min(63);
    title_buf[..tlen].copy_from_slice(&tb[..tlen]);
    unsafe {
        wm_create_window(150, 100, WIN_W as u32, WIN_H as u32, title_buf.as_ptr());
        gfx_fill_rect(150, 100, WIN_W, WIN_H, CLR_WIN_BG);
        gfx_fill_rect(150, 100, 160, WIN_H, CLR_SIDEBAR);
    }
    draw_str(165, 120, "[WIN32]", CLR_ACCENT);
    draw_str(165, 150, "[WIN32]", CLR_TEXT);
    draw_str(165, 180, "[WIN32]", CLR_TEXT);
    draw_str(165, 210, "[WIN32]", CLR_TEXT);
    draw_str(165, WIN_H + 60, "v2.0 Sandbox", 0xFF8A8A8A);
    draw_str(320, 120, "[WIN32]", CLR_TEXT);
    unsafe { gfx_fill_rect(320, 140, WIN_W - 180, 2, CLR_ACCENT); }
    let mut cur_y = 155;
    for i in 0..manager.apps.len() { draw_app_row(320, cur_y, &manager.apps, i); cur_y += 50; }
    unsafe {
        gfx_fill_rect(320, 360, WIN_W - 180, 50, CLR_SIDEBAR);
        gfx_fill_rect(330, 370, 100, 30, CLR_SUCCESS);
    }
    draw_str(345, 378, "[WIN32]", CLR_TEXT);
    unsafe { gfx_fill_rect(440, 370, 100, 30, CLR_WARNING); }
    draw_str(455, 378, "[WIN32]", CLR_TEXT);
    unsafe { wm_draw_all(); gfx_swap_buffers(); }
}

pub fn cmd_runwin(manager: &mut Win32Manager, args: &[&str]) {
    if args.len() < 2 { console_writeln("[WIN32]"); return; }
    let msg = format!("{}: {}...", "[WIN32]", args[1]);
    console_writeln(&msg);
    manager.create_process(args[1]);
    gui_main(manager);
}

