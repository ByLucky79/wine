// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Wine Sunucu Katmanı
// Dosya Yolu         : apps/system/compat/src/wine_server.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32, RISC-V 64, MIPS 32, MIPS 64, PowerPC 32, PowerPC 64, m68k, SPARC, LoongArch64
//
// Açıklama:
//   Wine NT sunucu katmanı öykünücüsü. Wine istemcileri ile sunucu
//   arasındaki Unix soketi üzerinden geçen protokolü işler.
//   Bileşenler:
//     - WineObjectManager  : NT çekirdek nesne tablosu (process/thread/
//       file/mutex/semaphore/event/timer/socket/pipe/window/registry)
//     - WineRegistryHive   : HKLM/HKCU/HKCR kayıt defteri öykünücüsü
//     - WineNtPathConverter : NT \\??\\C:\\ yollarını POSIX'e dönüştürür
//     - WineServerProtocol : istek başlığı ayrıştırma ve opcode tablosu
//     - WineDllManager     : DLL yükleme/boşaltma tablosu
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs  (console_writeln)
//   2-) kernel/graphics/ui/src/lang/lang_flags.rs (MsgId)
//   3-) kernel/graphics/ui/src/lang/lang_manager.rs (Lang::get)
//
//              Dosyaya Müdahaleler
// 2026-04-17      C → Rust çevirisi (no_std)
// 2026-04-18      Lang sistemi, hata tipleri, DLL tablosu, NT yol
//                 dönüştürücü, kayıt defteri iyileştirmeleri
// *******************************************************************

#![allow(dead_code)]

use crate::dos_emulator::console_writeln;
use kernel_ui::{Lang, MsgId};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_has_atomic = "64")]
use core::sync::atomic::AtomicU64;
#[cfg(not(target_has_atomic = "64"))]
#[derive(Debug)]
struct AtomicU64 {
    lo: core::sync::atomic::AtomicU32,
    hi: core::sync::atomic::AtomicU32,
    seq: core::sync::atomic::AtomicU32,
}
#[cfg(not(target_has_atomic = "64"))]
impl AtomicU64 {
    const fn new(v: u64) -> Self {
        Self {
            lo: core::sync::atomic::AtomicU32::new(v as u32),
            hi: core::sync::atomic::AtomicU32::new((v >> 32) as u32),
            seq: core::sync::atomic::AtomicU32::new(0),
        }
    }
    fn load(&self, _: core::sync::atomic::Ordering) -> u64 {
        use core::sync::atomic::Ordering;
        loop {
            let s1 = self.seq.load(Ordering::Acquire);
            if s1 & 1 != 0 { continue; }
            let lo = self.lo.load(Ordering::Relaxed);
            let hi = self.hi.load(Ordering::Relaxed);
            if self.seq.load(Ordering::Acquire) == s1 {
                return ((hi as u64) << 32) | (lo as u64);
            }
        }
    }
    fn store(&self, v: u64, _: core::sync::atomic::Ordering) {
        use core::sync::atomic::Ordering;
        self.seq.fetch_add(1, Ordering::Release);
        self.lo.store(v as u32, Ordering::Relaxed);
        self.hi.store((v >> 32) as u32, Ordering::Relaxed);
        self.seq.fetch_add(1, Ordering::Release);
    }
    fn fetch_add(&self, v: u64, o: core::sync::atomic::Ordering) -> u64 { let x=self.load(o); self.store(x.wrapping_add(v),o); x }
    fn fetch_sub(&self, v: u64, o: core::sync::atomic::Ordering) -> u64 { let x=self.load(o); self.store(x.wrapping_sub(v),o); x }
    fn fetch_or(&self, v: u64, o: core::sync::atomic::Ordering) -> u64 { let x=self.load(o); self.store(x|v,o); x }
    fn fetch_and(&self, v: u64, o: core::sync::atomic::Ordering) -> u64 { let x=self.load(o); self.store(x&v,o); x }
    fn fetch_xor(&self, v: u64, o: core::sync::atomic::Ordering) -> u64 { let x=self.load(o); self.store(x^v,o); x }
    fn swap(&self, v: u64, o: core::sync::atomic::Ordering) -> u64 { let x=self.load(o); self.store(v,o); x }
    fn compare_exchange(&self, cur: u64, new: u64, _s: core::sync::atomic::Ordering, _f: core::sync::atomic::Ordering) -> Result<u64,u64> {
        let old = self.load(core::sync::atomic::Ordering::SeqCst);
        if old==cur { self.store(new, core::sync::atomic::Ordering::SeqCst); Ok(old) } else { Err(old) }
    }
}

// ─── Hata Tipi ──────────────────────────────────────────────────

/// Wine sunucu katmanı hata türleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WineError {
    /// Sunucu başlatılmadan işlem yapıldı.
    NotInitialized,
    /// Belirtilen handle geçersiz.
    InvalidHandle,
    /// DLL daha önce yüklenmiş.
    DllAlreadyLoaded,
    /// DLL adı boş ya da geçersiz.
    InvalidDllName,
    /// NT yol biçimi tanınmıyor.
    InvalidNtPath,
    /// Kayıt defteri yolu geçersiz.
    InvalidRegistryPath,
    /// İstek verisi çok kısa.
    RequestTooShort,
}

impl WineError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotInitialized     => "wine server not initialized",
            Self::InvalidHandle      => "invalid handle",
            Self::DllAlreadyLoaded   => "dll already loaded",
            Self::InvalidDllName     => "invalid dll name",
            Self::InvalidNtPath      => "invalid NT path",
            Self::InvalidRegistryPath=> "invalid registry path",
            Self::RequestTooShort    => "request data too short",
        }
    }
}

// ─── Sunucu Başlatma ────────────────────────────────────────────

static WINE_INITIALIZED: AtomicBool  = AtomicBool::new(false);
static WINE_SESSION_ID:  AtomicU64   = AtomicU64::new(0);

/// Wine sunucu katmanını başlat. Yeniden çağrılabilir (idempotent).
pub fn wine_server_init() {
    let session = WINE_SESSION_ID.fetch_add(1, Ordering::SeqCst);
    WINE_INITIALIZED.store(true, Ordering::SeqCst);
    let msg = format!("{} (oturum #{})",
        &Lang::get(MsgId::CompatWineServerInit), session);
    console_writeln(&msg);
}

/// Wine sunucu katmanını kapat.
pub fn wine_server_shutdown() {
    WINE_INITIALIZED.store(false, Ordering::SeqCst);
    console_writeln(&Lang::get(MsgId::CompatWineServerShutdown));
}

/// Sunucunun başlatılıp başlatılmadığını sorgula.
#[must_use]
pub fn is_initialized() -> bool {
    WINE_INITIALIZED.load(Ordering::SeqCst)
}

// ─── Wine Sunucu Protokolü ───────────────────────────────────────

/// Wine istemci→sunucu istek başlığı (12 byte, little-endian).
#[derive(Debug, Clone, Copy)]
pub struct WineServerRequestHeader {
    /// İstek opcode'u.
    pub req:          u32,
    /// İstemci taraflı istek kanalı fd.
    pub request_pipe: u32,
    /// İstemci taraflı yanıt kanalı fd.
    pub reply_pipe:   u32,
}

impl WineServerRequestHeader {
    /// Ham byte diliminden başlık ayrıştır.
    /// Veri 12 byte'tan kısaysa `Err(WineError::RequestTooShort)` döner.
    #[must_use]
    pub fn parse(data: &[u8]) -> Result<Self, WineError> {
        if data.len() < 12 { return Err(WineError::RequestTooShort); }
        Ok(Self {
            req:          u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            request_pipe: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            reply_pipe:   u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        })
    }

    /// Opcode'u ayrıştır.
    #[must_use]
    pub fn opcode(&self) -> WineRequestOpcode {
        WineRequestOpcode::from_u32(self.req)
    }
}

/// Wine NT sunucu istek opcode tablosu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WineRequestOpcode {
    // Süreç/iş parçacığı yönetimi
    NewProcess        = 0,
    GetNewProcessInfo = 1,
    NewThread         = 2,
    GetStartupInfo    = 3,
    InitProcessDone   = 4,
    InitThread        = 5,
    TerminateProcess  = 6,
    TerminateThread   = 7,
    GetProcessInfo    = 8,
    SetProcessInfo    = 9,
    GetThreadInfo     = 10,
    SetThreadInfo     = 11,
    GetDllInfo        = 12,
    SuspendThread     = 13,
    ResumeThread      = 14,
    LoadDll           = 15,
    UnloadDll         = 16,
    // Dosya işlemleri
    CreateFile        = 17,
    AllocFileHandle   = 18,
    GetHandleFd       = 19,
    FlushFile         = 20,
    GetFileInfo       = 21,
    SetFileInfo       = 22,
    SetEndOfFile      = 23,
    SetFilePointer    = 24,
    GetFileTime       = 25,
    SetFileTime       = 26,
    CreateDirectory   = 27,
    OpenDirectory     = 28,
    GetDirectoryEntry = 29,
    CreateSymlink     = 30,
    OpenSymlink       = 31,
    ReadSymlink       = 32,
    // Soket işlemleri
    CreateSocket      = 33,
    AcceptSocket      = 34,
    GetSocketInfo     = 35,
    EnableSocketEvent = 36,
    SetSocketEvent    = 37,
    // Senkronizasyon nesneleri
    CreateEvent       = 38,
    EventOp           = 39,
    OpenEvent         = 40,
    CreateMutex       = 41,
    ReleaseMutex      = 42,
    OpenMutex         = 43,
    CreateSemaphore   = 44,
    ReleaseSemaphore  = 45,
    OpenSemaphore     = 46,
    // Zamanlayıcı
    CreateTimer       = 47,
    SetTimer          = 48,
    CancelTimer       = 49,
    OpenTimer         = 50,
    // İsimli boru
    CreateNamedPipe   = 51,
    OpenNamedPipe     = 52,
    ConnectNamedPipe  = 53,
    WaitNamedPipe     = 54,
    DisconnectNamedPipe=55,
    // GUI
    CreateWindowStation=56,
    OpenWindowStation  =57,
    CreateDesktop     = 58,
    OpenDesktop       = 59,
    CloseDesktop      = 60,
    SetThreadDesktop  = 61,
    GetThreadDesktop  = 62,
    CreateWindow      = 63,
    DestroyWindow     = 64,
    SetWindowParent   = 65,
    // Sürüm / güncelleme
    GetVersion        = 66,
    // Bellek / bölüm
    CreateMapping     = 67,
    OpenMapping       = 68,
    MapView           = 69,
    UnmapView         = 70,
    // Token / güvenlik
    OpenToken         = 71,
    SetSecurityObject = 72,
    GetSecurityObject = 73,
    // Kayıt defteri
    CreateKey         = 74,
    OpenKey           = 75,
    DeleteKey         = 76,
    EnumKey           = 77,
    QueryKey          = 78,
    SetKeyValue       = 79,
    GetKeyValue       = 80,
    EnumKeyValue      = 81,
    DeleteKeyValue    = 82,
    // Bilinmiyor
    Unknown           = 0xFFFF,
}

impl WineRequestOpcode {
    #[must_use]
    pub fn from_u32(v: u32) -> Self {
        match v {
            0  => Self::NewProcess,        1  => Self::GetNewProcessInfo,
            2  => Self::NewThread,         3  => Self::GetStartupInfo,
            4  => Self::InitProcessDone,   5  => Self::InitThread,
            6  => Self::TerminateProcess,  7  => Self::TerminateThread,
            8  => Self::GetProcessInfo,    9  => Self::SetProcessInfo,
            10 => Self::GetThreadInfo,     11 => Self::SetThreadInfo,
            12 => Self::GetDllInfo,        13 => Self::SuspendThread,
            14 => Self::ResumeThread,      15 => Self::LoadDll,
            16 => Self::UnloadDll,         17 => Self::CreateFile,
            18 => Self::AllocFileHandle,   19 => Self::GetHandleFd,
            20 => Self::FlushFile,         21 => Self::GetFileInfo,
            22 => Self::SetFileInfo,       23 => Self::SetEndOfFile,
            24 => Self::SetFilePointer,    25 => Self::GetFileTime,
            26 => Self::SetFileTime,       27 => Self::CreateDirectory,
            28 => Self::OpenDirectory,     29 => Self::GetDirectoryEntry,
            30 => Self::CreateSymlink,     31 => Self::OpenSymlink,
            32 => Self::ReadSymlink,       33 => Self::CreateSocket,
            34 => Self::AcceptSocket,      35 => Self::GetSocketInfo,
            36 => Self::EnableSocketEvent, 37 => Self::SetSocketEvent,
            38 => Self::CreateEvent,       39 => Self::EventOp,
            40 => Self::OpenEvent,         41 => Self::CreateMutex,
            42 => Self::ReleaseMutex,      43 => Self::OpenMutex,
            44 => Self::CreateSemaphore,   45 => Self::ReleaseSemaphore,
            46 => Self::OpenSemaphore,     47 => Self::CreateTimer,
            48 => Self::SetTimer,          49 => Self::CancelTimer,
            50 => Self::OpenTimer,         51 => Self::CreateNamedPipe,
            52 => Self::OpenNamedPipe,     53 => Self::ConnectNamedPipe,
            54 => Self::WaitNamedPipe,     55 => Self::DisconnectNamedPipe,
            56 => Self::CreateWindowStation,57=> Self::OpenWindowStation,
            58 => Self::CreateDesktop,     59 => Self::OpenDesktop,
            60 => Self::CloseDesktop,      61 => Self::SetThreadDesktop,
            62 => Self::GetThreadDesktop,  63 => Self::CreateWindow,
            64 => Self::DestroyWindow,     65 => Self::SetWindowParent,
            66 => Self::GetVersion,        67 => Self::CreateMapping,
            68 => Self::OpenMapping,       69 => Self::MapView,
            70 => Self::UnmapView,         71 => Self::OpenToken,
            72 => Self::SetSecurityObject, 73 => Self::GetSecurityObject,
            74 => Self::CreateKey,         75 => Self::OpenKey,
            76 => Self::DeleteKey,         77 => Self::EnumKey,
            78 => Self::QueryKey,          79 => Self::SetKeyValue,
            80 => Self::GetKeyValue,       81 => Self::EnumKeyValue,
            82 => Self::DeleteKeyValue,
            _  => Self::Unknown,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NewProcess         => "new_process",
            Self::GetNewProcessInfo  => "get_new_process_info",
            Self::NewThread          => "new_thread",
            Self::GetStartupInfo     => "get_startup_info",
            Self::InitProcessDone    => "init_process_done",
            Self::InitThread         => "init_thread",
            Self::TerminateProcess   => "terminate_process",
            Self::TerminateThread    => "terminate_thread",
            Self::GetProcessInfo     => "get_process_info",
            Self::SetProcessInfo     => "set_process_info",
            Self::GetThreadInfo      => "get_thread_info",
            Self::SetThreadInfo      => "set_thread_info",
            Self::GetDllInfo         => "get_dll_info",
            Self::SuspendThread      => "suspend_thread",
            Self::ResumeThread       => "resume_thread",
            Self::LoadDll            => "load_dll",
            Self::UnloadDll          => "unload_dll",
            Self::CreateFile         => "create_file",
            Self::AllocFileHandle    => "alloc_file_handle",
            Self::GetHandleFd        => "get_handle_fd",
            Self::FlushFile          => "flush_file",
            Self::GetFileInfo        => "get_file_info",
            Self::SetFileInfo        => "set_file_info",
            Self::SetEndOfFile       => "set_eof",
            Self::SetFilePointer     => "set_file_ptr",
            Self::GetFileTime        => "get_file_time",
            Self::SetFileTime        => "set_file_time",
            Self::CreateDirectory    => "create_dir",
            Self::OpenDirectory      => "open_dir",
            Self::GetDirectoryEntry  => "get_dir_entry",
            Self::CreateSymlink      => "create_symlink",
            Self::OpenSymlink        => "open_symlink",
            Self::ReadSymlink        => "read_symlink",
            Self::CreateSocket       => "create_socket",
            Self::AcceptSocket       => "accept_socket",
            Self::GetSocketInfo      => "get_socket_info",
            Self::EnableSocketEvent  => "enable_socket_event",
            Self::SetSocketEvent     => "set_socket_event",
            Self::CreateEvent        => "create_event",
            Self::EventOp            => "event_op",
            Self::OpenEvent          => "open_event",
            Self::CreateMutex        => "create_mutex",
            Self::ReleaseMutex       => "release_mutex",
            Self::OpenMutex          => "open_mutex",
            Self::CreateSemaphore    => "create_semaphore",
            Self::ReleaseSemaphore   => "release_semaphore",
            Self::OpenSemaphore      => "open_semaphore",
            Self::CreateTimer        => "create_timer",
            Self::SetTimer           => "set_timer",
            Self::CancelTimer        => "cancel_timer",
            Self::OpenTimer          => "open_timer",
            Self::CreateNamedPipe    => "create_named_pipe",
            Self::OpenNamedPipe      => "open_named_pipe",
            Self::ConnectNamedPipe   => "connect_named_pipe",
            Self::WaitNamedPipe      => "wait_named_pipe",
            Self::DisconnectNamedPipe=> "disconnect_named_pipe",
            Self::CreateWindowStation=> "create_winsta",
            Self::OpenWindowStation  => "open_winsta",
            Self::CreateDesktop      => "create_desktop",
            Self::OpenDesktop        => "open_desktop",
            Self::CloseDesktop       => "close_desktop",
            Self::SetThreadDesktop   => "set_thread_desktop",
            Self::GetThreadDesktop   => "get_thread_desktop",
            Self::CreateWindow       => "create_window",
            Self::DestroyWindow      => "destroy_window",
            Self::SetWindowParent    => "set_window_parent",
            Self::GetVersion         => "get_version",
            Self::CreateMapping      => "create_mapping",
            Self::OpenMapping        => "open_mapping",
            Self::MapView            => "map_view",
            Self::UnmapView          => "unmap_view",
            Self::OpenToken          => "open_token",
            Self::SetSecurityObject  => "set_security_object",
            Self::GetSecurityObject  => "get_security_object",
            Self::CreateKey          => "create_key",
            Self::OpenKey            => "open_key",
            Self::DeleteKey          => "delete_key",
            Self::EnumKey            => "enum_key",
            Self::QueryKey           => "query_key",
            Self::SetKeyValue        => "set_key_value",
            Self::GetKeyValue        => "get_key_value",
            Self::EnumKeyValue       => "enum_key_value",
            Self::DeleteKeyValue     => "delete_key_value",
            Self::Unknown            => "unknown",
        }
    }

    /// İşlem kategorisi.
    #[must_use]
    pub fn category(self) -> &'static str {
        match self as u32 {
            0..=16  => "process/thread",
            17..=32 => "file",
            33..=37 => "socket",
            38..=50 => "synchronization",
            51..=55 => "named_pipe",
            56..=65 => "gui",
            66..=70 => "memory",
            71..=73 => "security",
            74..=82 => "registry",
            _       => "unknown",
        }
    }
}

// ─── Wine Nesne Yöneticisi ───────────────────────────────────────

/// NT çekirdek nesne türleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WineObjectType {
    Process, Thread, File, Event, Mutex, Semaphore, Timer,
    NamedPipe, Socket, WindowStation, Desktop, Window, Symlink,
    Directory, RegistryKey, Section, Token,
}

impl WineObjectType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Process       => "Process",
            Self::Thread        => "Thread",
            Self::File          => "File",
            Self::Event         => "Event",
            Self::Mutex         => "Mutex",
            Self::Semaphore     => "Semaphore",
            Self::Timer         => "Timer",
            Self::NamedPipe     => "NamedPipe",
            Self::Socket        => "Socket",
            Self::WindowStation => "WinSta",
            Self::Desktop       => "Desktop",
            Self::Window        => "Window",
            Self::Symlink       => "Symlink",
            Self::Directory     => "Directory",
            Self::RegistryKey   => "RegKey",
            Self::Section       => "Section",
            Self::Token         => "Token",
        }
    }
}

/// Tek bir NT nesne kaydı.
#[derive(Debug, Clone)]
pub struct WineObject {
    pub handle:     u64,
    pub obj_type:   WineObjectType,
    pub name:       String,
    pub access:     u32,
    pub ref_count:  u32,
}

/// NT çekirdek nesne tablosu — handle tahsisi ve arama.
pub struct WineObjectManager {
    objects:     Vec<WineObject>,
    next_handle: u64,
}

impl WineObjectManager {
    /// Boş nesne tablosu oluştur. Handle tahsisi 0x10000'dan başlar.
    #[must_use]
    pub fn new() -> Self {
        Self { objects: Vec::new(), next_handle: 0x0001_0000 }
    }

    /// Yeni nesne oluştur; handle döndür.
    pub fn create(&mut self, obj_type: WineObjectType, name: &str, access: u32) -> u64 {
        let h = self.next_handle;
        self.next_handle += 4; // 4'ün katları (Windows uyumu)
        self.objects.push(WineObject {
            handle: h,
            obj_type,
            name: name.to_string(),
            access,
            ref_count: 1,
        });
        let msg = format!("{}: {} \"{}\" → 0x{:08X}",
            &Lang::get(MsgId::CompatWineObjCreated),
            obj_type.as_str(), name, h);
        console_writeln(&msg);
        h
    }

    /// Handle kapat (ref_count azalt; sıfıra gelince nesneyi sil).
    /// Handle bulunamazsa `Err(WineError::InvalidHandle)` döner.
    #[must_use]
    pub fn close(&mut self, handle: u64) -> Result<(), WineError> {
        let pos = self.objects.iter().position(|o| o.handle == handle)
            .ok_or(WineError::InvalidHandle)?;
        self.objects[pos].ref_count -= 1;
        let msg = format!("{}: 0x{:08X} (ref={})",
            &Lang::get(MsgId::CompatWineHandleClosed),
            handle, self.objects[pos].ref_count);
        console_writeln(&msg);
        if self.objects[pos].ref_count == 0 {
            self.objects.remove(pos);
        }
        Ok(())
    }

    /// Handle'a karşılık gelen nesneyi döndür.
    #[must_use]
    pub fn lookup(&self, handle: u64) -> Option<&WineObject> {
        self.objects.iter().find(|o| o.handle == handle)
    }

    /// Handle'a karşılık gelen nesneyi değiştirilebilir döndür.
    #[must_use]
    pub fn lookup_mut(&mut self, handle: u64) -> Option<&mut WineObject> {
        self.objects.iter_mut().find(|o| o.handle == handle)
    }

    /// Açık nesne sayısı.
    #[must_use]
    pub fn count(&self) -> usize { self.objects.len() }

    /// Belirli türdeki nesneleri listele.
    #[must_use]
    pub fn list_by_type(&self, t: WineObjectType) -> Vec<&WineObject> {
        self.objects.iter().filter(|o| o.obj_type == t).collect()
    }

    /// Tüm nesneleri konsola döküm et.
    pub fn dump(&self) {
        let hdr = format!("{} ({} nesne):",
            &Lang::get(MsgId::CompatWineObjects),
            self.objects.len());
        console_writeln(&hdr);
        for o in &self.objects {
            let msg = format!("  0x{:08X}  {:12}  ref={}  acc=0x{:08X}  \"{}\"",
                o.handle, o.obj_type.as_str(), o.ref_count, o.access, o.name);
            console_writeln(&msg);
        }
    }
}

impl Default for WineObjectManager {
    fn default() -> Self { Self::new() }
}

// ─── Wine NT Yol Dönüştürücü ─────────────────────────────────────

/// NT / DOS yollarını Wine POSIX prefixine dönüştürür.
pub struct WineNtPathConverter;

impl WineNtPathConverter {
    /// NT yolunu Wine Unix prefix'indeki gerçek dosya yoluna çevir.
    ///
    /// # Desteklenen biçimler
    /// - `\\?\C:\Windows\...`     → `<prefix>/drive_c/Windows/...`
    /// - `\\.\pipe\name`          → `<prefix>/dosdevices/pipe/name`
    /// - `\??\C:\...`             → `<prefix>/drive_c/...`
    /// - `C:\Windows\...`         → `<prefix>/drive_c/Windows/...`
    /// - `\\server\share\...`     → `<prefix>/dosdevices/server/share/...`
    #[must_use]
    pub fn nt_to_unix(path: &str, wine_prefix: &str) -> Result<String, WineError> {
        if path.is_empty() { return Err(WineError::InvalidNtPath); }

        let mut base = wine_prefix.trim_end_matches('/').to_string();
        base.push('/');

        let stripped = if path.starts_with("\\\\?\\") || path.starts_with("\\??\\") {
            &path[4..]
        } else if path.starts_with("\\\\.\\") {
            // Device path: \\.\pipe\foo → dosdevices/pipe/foo
            let rest = path[4..].replace('\\', "/");
            return Ok(format!("{}dosdevices/{}", base, rest));
        } else if path.starts_with("\\\\") {
            // UNC: \\server\share → dosdevices/server/share
            let rest = path[2..].replace('\\', "/");
            return Ok(format!("{}dosdevices/{}", base, rest));
        } else {
            path
        };

        // Sürücü harfi var mı? ör. "C:\Windows" → drive_c/Windows
        let bytes = stripped.as_bytes();
        if bytes.len() >= 2 && bytes[1] == b':' {
            let drive = bytes[0].to_ascii_lowercase() as char;
            let rest   = if bytes.len() > 2 { &stripped[2..] } else { "" };
            let rest_clean = rest.replace('\\', "/").trim_start_matches('/').to_string();
            Ok(format!("{}drive_{}/{}", base, drive, rest_clean))
        } else {
            // Mutlak NT yolu
            Ok(format!("{}dosdevices/{}", base, stripped.replace('\\', "/")))
        }
    }

    /// DOS yolunu Wine prefix'ine çevir (NT dönüştürücüsünün takma adı).
    #[must_use]
    pub fn dos_to_unix(dos: &str, wine_prefix: &str) -> Result<String, WineError> {
        Self::nt_to_unix(dos, wine_prefix)
    }

    /// Prefix için örnek dönüşümleri konsola göster.
    pub fn dump_prefix(prefix: &str) {
        let hdr = format!("{}: {}", &Lang::get(MsgId::CompatWinePrefix), prefix);
        console_writeln(&hdr);
        let examples = &[
            "C:\\Windows\\System32\\kernel32.dll",
            "D:\\Games\\test.exe",
            "\\??\\C:\\temp\\file.txt",
            "\\\\.\\pipe\\wineserver",
            "\\\\server\\share\\file.txt",
        ];
        for ex in examples {
            match Self::nt_to_unix(ex, prefix) {
                Ok(unix)  => { let msg = format!("  {} → {}", ex, unix); console_writeln(&msg); }
                Err(e)    => { let msg = format!("  {} → ERR:{}", ex, e.as_str()); console_writeln(&msg); }
            }
        }
    }
}

// ─── Wine DLL Yöneticisi ─────────────────────────────────────────

/// Yüklenmiş tek bir Wine DLL kaydı.
#[derive(Debug, Clone)]
pub struct WineDll {
    pub name:        String,
    pub base:        u64,
    pub size:        u32,
    pub ref_count:   u32,
    pub is_builtin:  bool,
}

/// Wine DLL yükleme/boşaltma tablosu.
pub struct WineDllManager {
    dlls: BTreeMap<String, WineDll>,
}

impl WineDllManager {
    /// Temel Wine yerleşik DLL'lerle önceden dolu tablo.
    #[must_use]
    pub fn new() -> Self {
        let mut mgr = Self { dlls: BTreeMap::new() };
        // Yerleşik (builtin) DLL'ler
        let builtins = [
            ("ntdll",     0xBFFF_0000u64, 0x0010_0000u32),
            ("kernel32",  0x7C80_0000,    0x0008_0000),
            ("user32",    0x7E41_0000,    0x0009_0000),
            ("gdi32",     0x7C9D_0000,    0x0005_0000),
            ("advapi32",  0x7C95_0000,    0x0008_0000),
            ("msvcrt",    0x7C34_0000,    0x0009_0000),
            ("ole32",     0x7741_0000,    0x0010_0000),
            ("shell32",   0x7C9C_0000,    0x0018_0000),
            ("ws2_32",    0x71AB_0000,    0x0003_0000),
            ("winmm",     0x76B4_0000,    0x0003_0000),
            ("d3d9",      0x69A0_0000,    0x0014_0000),
            ("d3d11",     0x6A40_0000,    0x001C_0000),
            ("opengl32",  0x6950_0000,    0x0008_0000),
            ("vulkan-1",  0x6A00_0000,    0x0010_0000),
        ];
        for (name, base, size) in builtins {
            mgr.dlls.insert(name.to_string(), WineDll {
                name: name.to_string(),
                base, size,
                ref_count: 0,
                is_builtin: true,
            });
        }
        mgr
    }

    /// DLL yükle. Zaten yüklü yerleşik DLL'lerde ref_count artar.
    /// Yeni DLL için `base` ve `size` verilmeli.
    #[must_use]
    pub fn load(&mut self, name: &str, base: u64, size: u32) -> Result<u64, WineError> {
        if name.is_empty() { return Err(WineError::InvalidDllName); }
        let key = name.trim_end_matches(".dll").to_lowercase();
        if let Some(dll) = self.dlls.get_mut(&key) {
            dll.ref_count += 1;
            let msg = format!("{}: {} (ref={}, base=0x{:08X})",
                &Lang::get(MsgId::CompatWineDllLoad), key, dll.ref_count, dll.base);
            console_writeln(&msg);
            return Ok(dll.base);
        }
        // Yeni DLL kaydı
        let msg = format!("{}: {} (yeni, base=0x{:08X}, size={}K)",
            &Lang::get(MsgId::CompatWineDllLoad), key, base, size / 1024);
        console_writeln(&msg);
        self.dlls.insert(key.clone(), WineDll {
            name: key, base, size, ref_count: 1, is_builtin: false,
        });
        Ok(base)
    }

    /// DLL boşalt (ref_count azalt; sıfırsa kaldır).
    #[must_use]
    pub fn unload(&mut self, name: &str) -> Result<(), WineError> {
        let key = name.trim_end_matches(".dll").to_lowercase();
        let dll = self.dlls.get_mut(&key).ok_or(WineError::InvalidDllName)?;
        if dll.ref_count > 0 { dll.ref_count -= 1; }
        if dll.ref_count == 0 && !dll.is_builtin {
            self.dlls.remove(&key);
        }
        Ok(())
    }

    /// DLL tabanı adres al.
    #[must_use]
    pub fn base_of(&self, name: &str) -> Option<u64> {
        let key = name.trim_end_matches(".dll").to_lowercase();
        self.dlls.get(&key).map(|d| d.base)
    }

    /// Yüklü DLL sayısı (ref_count > 0 olanlar).
    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.dlls.values().filter(|d| d.ref_count > 0).count()
    }

    /// Wine prosedür çağrısı (öykünücü: stub).
    #[must_use]
    pub fn call_proc(
        &self,
        dll:  &str,
        proc: &str,
        _args: &[u64],
    ) -> Result<i64, WineError> {
        if dll.is_empty() || proc.is_empty() {
            return Err(WineError::InvalidDllName);
        }
        let msg = format!("{}: {}!{}", &Lang::get(MsgId::CompatWineProc), dll, proc);
        console_writeln(&msg);
        Ok(0)
    }
}

impl Default for WineDllManager {
    fn default() -> Self { Self::new() }
}

// ─── Wine Kayıt Defteri ──────────────────────────────────────────

/// Kayıt defteri değer türleri (REG_*).
#[path = "wine_server_registry.rs"] pub mod wine_server_registry;
pub use wine_server_registry::*;