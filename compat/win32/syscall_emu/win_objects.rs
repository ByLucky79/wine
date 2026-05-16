// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Nesne Yoneticileri - Timer, Metrics, Env, Heap, Sync, Atom, Clipboard, Hook
// Dosya Yolu         : apps/system/compat/win32/win_objects.rs
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
//   Win32 Nesne Yoneticileri - Timer, Metrics, Env, Heap, Sync, Atom, Clipboard, Hook
//
// Bagimli Dosyalar:
//   1-) apps/system/compat/win32/win32_mod.rs
//
//              Dosyaya Mudahaleler
// 2026-05-13      syscall_emu.rs bolundu
// *******************************************************************

#![allow(dead_code)]

extern crate alloc;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use crate::dos_emulator::console_writeln;


// ─── Window Timer Manager ──────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct WinTimer {
    pub hwnd: u64, pub id: u64, pub interval: u32, pub elapsed: u32,
}

pub struct TimerManager {
    pub timers: Vec<WinTimer>,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerManager {
    pub fn new() -> Self { Self { timers: Vec::new() } }
    pub fn set_timer(&mut self, hwnd: u64, id: u64, interval: u32) -> u64 {
        self.timers.push(WinTimer { hwnd, id, interval, elapsed: 0 });
        let msg = format!("{} SetTimer(hwnd=0x{:X}, id={}, interval={}ms)", "[WIN32]", hwnd, id, interval);
        console_writeln(&msg); id
    }
    pub fn kill_timer(&mut self, hwnd: u64, id: u64) -> bool {
        let before = self.timers.len();
        self.timers.retain(|t| !(t.hwnd == hwnd && t.id == id));
        let after = self.timers.len();
        let msg = format!("{} KillTimer(hwnd=0x{:X}, id={}) -> {}", "[WIN32]", hwnd, id, before != after);
        console_writeln(&msg); before != after
    }
    pub fn tick(&mut self, delta_ms: u32) {
        for t in &mut self.timers { t.elapsed += delta_ms; }
        let expired: Vec<u64> = self.timers.iter().filter(|t| t.elapsed >= t.interval).map(|t| t.id).collect();
        for id in expired {
            let msg = format!("{} WM_TIMER(id={})", "[WIN32]", id);
            console_writeln(&msg);
            for t in &mut self.timers { if t.id == id { t.elapsed = 0; } }
        }
    }
    pub fn dump(&self) {
        let msg = format!("{} Active timers:", "[WIN32]");
        console_writeln(&msg);
        for t in &self.timers { let msg = format!("  hwnd=0x{:X} id={} interval={}ms", t.hwnd, t.id, t.interval); console_writeln(&msg); }
    }
}

// ─── System Metrics Emulator ───────────────────────────────────

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

// ─── Environment Variable Manager ──────────────────────────────

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

// ─── Heap Manager ──────────────────────────────────────────────

pub struct HeapBlock {
    pub base: u64, pub size: u64, pub flags: u32,
}

pub struct HeapManager {
    pub blocks: Vec<HeapBlock>, pub base: u64, pub next: u64,
}

impl HeapManager {
    pub fn new(base: u64, _initial_size: u64) -> Self { Self { blocks: Vec::new(), base, next: base } }
    pub fn alloc(&mut self, size: u64, flags: u32) -> u64 {
        let aligned = (size + 0xF) & !0xF;
        let addr = self.next;
        self.blocks.push(HeapBlock { base: addr, size: aligned, flags });
        self.next += aligned;
        let msg = format!("{} HeapAlloc(addr=0x{:016X}, size=0x{:X}, flags=0x{:X})", "[WIN32]", addr, size, flags);
        console_writeln(&msg); addr
    }
    pub fn free(&mut self, addr: u64) -> bool {
        let before = self.blocks.len();
        self.blocks.retain(|b| b.base != addr);
        let after = self.blocks.len();
        let msg = format!("{} HeapFree(addr=0x{:016X}) -> {}", "[WIN32]", addr, before != after);
        console_writeln(&msg); before != after
    }
    pub fn realloc(&mut self, addr: u64, size: u64) -> u64 {
        let new_addr = self.alloc(size, 0);
        let msg = format!("{} HeapReAlloc(old=0x{:016X}, new=0x{:016X}, size=0x{:X})", "[WIN32]", addr, new_addr, size);
        console_writeln(&msg);
        self.free(addr); new_addr
    }
    pub fn size(&self, addr: u64) -> u64 { self.blocks.iter().find(|b| b.base == addr).map_or(0, |b| b.size) }
    pub fn dump(&self) {
        let msg = format!("{} Active blocks:", "[WIN32]");
        console_writeln(&msg);
        for b in &self.blocks { let msg = format!("  addr=0x{:016X} size=0x{:X} flags=0x{:X}", b.base, b.size, b.flags); console_writeln(&msg); }
    }
}

// ─── Synchronization Primitives Emulator ───────────────────────

#[derive(Debug, Clone, Copy)]
pub enum SyncObjectType { CriticalSection, Mutex, Semaphore, Event }

#[derive(Debug, Clone)]
pub struct SyncObject {
    pub handle: u64, pub obj_type: SyncObjectType, pub name: String,
    pub locked: bool, pub owner_tid: u32, pub count: u32, pub signaled: bool, pub manual_reset: bool,
}

pub struct SyncManager {
    pub objects: Vec<SyncObject>, pub next_handle: u64,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncManager {
    pub fn new() -> Self { Self { objects: Vec::new(), next_handle: 0x8000 } }
    fn alloc_handle(&mut self) -> u64 { let h = self.next_handle; self.next_handle += 1; h }
    pub fn create_mutex(&mut self, name: &str, initial_owner: bool) -> u64 {
        let h = self.alloc_handle();
        self.objects.push(SyncObject { handle: h, obj_type: SyncObjectType::Mutex, name: String::from(name), locked: initial_owner, owner_tid: 0, count: 1, signaled: false, manual_reset: false });
        let msg = format!("[SYNC] CreateMutex(\"{}\", owner={}) -> 0x{:08X}", name, initial_owner, h);
        console_writeln(&msg); h
    }
    pub fn create_semaphore(&mut self, name: &str, initial: u32, maximum: u32) -> u64 {
        let h = self.alloc_handle();
        self.objects.push(SyncObject { handle: h, obj_type: SyncObjectType::Semaphore, name: String::from(name), locked: false, owner_tid: 0, count: initial, signaled: initial > 0, manual_reset: false });
        let msg = format!("[SYNC] CreateSemaphore(\"{}\", init={}, max={}) -> 0x{:08X}", name, initial, maximum, h);
        console_writeln(&msg); h
    }
    pub fn create_event(&mut self, name: &str, manual_reset: bool, initial_state: bool) -> u64 {
        let h = self.alloc_handle();
        self.objects.push(SyncObject { handle: h, obj_type: SyncObjectType::Event, name: String::from(name), locked: false, owner_tid: 0, count: 0, signaled: initial_state, manual_reset });
        let msg = format!("[SYNC] CreateEvent(\"{}\", manual={}, signaled={}) -> 0x{:08X}", name, manual_reset, initial_state, h);
        console_writeln(&msg); h
    }
    pub fn wait_for_single_object(&mut self, handle: u64, _timeout_ms: u32) -> u32 {
        let msg = format!("[SYNC] WaitForSingleObject(0x{:08X})", handle);
        console_writeln(&msg); 0 // WAIT_OBJECT_0
    }
    pub fn set_event(&mut self, handle: u64) -> bool {
        if let Some(obj) = self.objects.iter_mut().find(|o| o.handle == handle) {
            obj.signaled = true;
            let msg = format!("[SYNC] SetEvent(0x{:08X})", handle);
            console_writeln(&msg); return true;
        }
        false
    }
    pub fn reset_event(&mut self, handle: u64) -> bool {
        if let Some(obj) = self.objects.iter_mut().find(|o| o.handle == handle) {
            obj.signaled = false;
            let msg = format!("[SYNC] ResetEvent(0x{:08X})", handle);
            console_writeln(&msg); return true;
        }
        false
    }
    pub fn release_mutex(&mut self, handle: u64) -> bool {
        let msg = format!("[SYNC] ReleaseMutex(0x{:08X})", handle);
        console_writeln(&msg); true
    }
    pub fn release_semaphore(&mut self, handle: u64, count: u32) -> bool {
        let msg = format!("[SYNC] ReleaseSemaphore(0x{:08X}, count={})", handle, count);
        console_writeln(&msg); true
    }
    pub fn close_handle(&mut self, handle: u64) -> bool {
        let before = self.objects.len();
        self.objects.retain(|o| o.handle != handle);
        let after = self.objects.len();
        let msg = format!("[SYNC] CloseHandle(0x{:08X}) -> {}", handle, before != after);
        console_writeln(&msg); before != after
    }
}

pub struct CriticalSectionEmulator;

impl CriticalSectionEmulator {
    pub fn initialize(_cs: u64) { let msg = format!("{} InitializeCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn delete(_cs: u64) { let msg = format!("{} DeleteCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn enter(_cs: u64) { let msg = format!("{} EnterCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn leave(_cs: u64) { let msg = format!("{} LeaveCriticalSection()", "[WIN32]"); console_writeln(&msg); }
    pub fn try_enter(_cs: u64) -> bool { let msg = format!("{} TryEnterCriticalSection() -> true", "[WIN32]"); console_writeln(&msg); true }
}

// ─── Atom Table Manager ────────────────────────────────────────

pub struct AtomTable {
    pub atoms: Vec<(u16, String)>, pub next_id: u16,
}

impl Default for AtomTable {
    fn default() -> Self {
        Self::new()
    }
}

impl AtomTable {
    pub fn new() -> Self { Self { atoms: Vec::new(), next_id: 0xC000 } }
    pub fn add(&mut self, name: &str) -> u16 {
        if let Some((id, _)) = self.atoms.iter().find(|(_, n)| n == name) { return *id; }
        let id = self.next_id; self.next_id += 1;
        self.atoms.push((id, String::from(name)));
        let msg = format!("[ATOM] GlobalAddAtom(\"{}\") -> 0x{:04X}", name, id);
        console_writeln(&msg); id
    }
    pub fn delete(&mut self, atom: u16) -> bool {
        let before = self.atoms.len();
        self.atoms.retain(|(id, _)| *id != atom);
        let after = self.atoms.len();
        let msg = format!("[ATOM] GlobalDeleteAtom(0x{:04X}) -> {}", atom, before != after);
        console_writeln(&msg); before != after
    }
    pub fn find(&self, atom: u16) -> Option<String> {
        self.atoms.iter().find(|(id, _)| *id == atom).map(|(_, n)| n.clone())
    }
    pub fn find_by_name(&self, name: &str) -> Option<u16> {
        self.atoms.iter().find(|(_, n)| n == name).map(|(id, _)| *id)
    }
}

// ─── Clipboard Manager ─────────────────────────────────────────

pub struct ClipboardManager {
    pub data: BTreeMap<u32, Vec<u8>>, pub open: bool, pub owner: u64,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardManager {
    pub fn new() -> Self { Self { data: BTreeMap::new(), open: false, owner: 0 } }
    pub fn open_clipboard(&mut self, owner: u64) -> bool {
        if self.open { return false; }
        self.open = true; self.owner = owner;
        let msg = format!("[CLIPBOARD] OpenClipboard(hwnd=0x{:08X})", owner);
        console_writeln(&msg); true
    }
    pub fn close_clipboard(&mut self) -> bool {
        if !self.open { return false; }
        self.open = false; self.owner = 0;
        console_writeln("[CLIPBOARD] CloseClipboard()"); true
    }
    pub fn empty_clipboard(&mut self) -> bool {
        self.data.clear();
        console_writeln("[CLIPBOARD] EmptyClipboard()"); true
    }
    pub fn set_clipboard_data(&mut self, format: u32, data: &[u8]) -> u64 {
        self.data.insert(format, data.to_vec());
        let handle = 0x9000 + format as u64;
        let msg = format!("[CLIPBOARD] SetClipboardData(fmt={}) -> 0x{:08X}", format, handle);
        console_writeln(&msg); handle
    }
    pub fn get_clipboard_data(&self, format: u32) -> Option<&Vec<u8>> {
        let msg = format!("[CLIPBOARD] GetClipboardData(fmt={})", format);
        console_writeln(&msg); self.data.get(&format)
    }
    pub fn enum_formats(&self) -> Vec<u32> { self.data.keys().copied().collect() }
}

// ─── Windows Hook Manager ──────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum HookType {
    CallWndProc = 4, CallWndProcRet = 12, Cbt = 5, Debug = 9,
    ForegroundIdle = 11, GetMessage = 3, JournalPlayback = 1, JournalRecord = 0,
    Keyboard = 2, KeyboardLl = 13, Mouse = 7, MouseLl = 14,
    MsgFilter = -1isize, Shell = 10, SysMsgFilter = 6,
}

#[derive(Debug, Clone)]
pub struct WinHook {
    pub id: u64, pub hook_type: i32, pub proc: u64, pub module: u64, pub thread_id: u32,
}

pub struct HookManager {
    pub hooks: Vec<WinHook>, pub next_id: u64,
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HookManager {
    pub fn new() -> Self { Self { hooks: Vec::new(), next_id: 0xA000 } }
    pub fn set_windows_hook_ex(&mut self, hook_type: i32, proc: u64, module: u64, thread_id: u32) -> u64 {
        let id = self.next_id; self.next_id += 1;
        self.hooks.push(WinHook { id, hook_type, proc, module, thread_id });
        let msg = format!("[HOOK] SetWindowsHookEx(type={}, proc=0x{:016X}, mod=0x{:016X}, tid={}) -> 0x{:08X}", hook_type, proc, module, thread_id, id);
        console_writeln(&msg); id
    }
    pub fn unhook_windows_hook_ex(&mut self, id: u64) -> bool {
        let before = self.hooks.len();
        self.hooks.retain(|h| h.id != id);
        let after = self.hooks.len();
        let msg = format!("[HOOK] UnhookWindowsHookEx(0x{:08X}) -> {}", id, before != after);
        console_writeln(&msg); before != after
    }
    pub fn call_next_hook_ex(&self, _id: u64, _code: i32, _wparam: u64, _lparam: u64) -> i64 {
        console_writeln("[HOOK] CallNextHookEx()"); 0
    }
    pub fn enum_hooks(&self) {
        console_writeln("[HOOK] Installed hooks:");
        for h in &self.hooks { let msg = format!("  id=0x{:08X} type={} tid={}", h.id, h.hook_type, h.thread_id); console_writeln(&msg); }
    }
}

// ─── Process Environment Block (PEB) Emulator ──────────────────

#[derive(Debug, Clone)]
pub struct ProcessEnvironmentBlock {
    pub image_base:     usize,
    pub ldr:            usize,
    pub process_params: usize,
    pub os_major:       u32,
    pub os_minor:       u32,
    pub os_build:       u32,
    pub being_debugged: bool,
}

impl ProcessEnvironmentBlock {
    pub fn new(image_base: usize) -> Self {
        Self {
            image_base,
            ldr:            0,
            process_params: 0,
            os_major:       10,
            os_minor:       0,
            os_build:       19045,
            being_debugged: false,
        }
    }
}
