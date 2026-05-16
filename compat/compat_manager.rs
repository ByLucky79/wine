// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Uyumluluk Yöneticisi (Orchestrator)
// Dosya Yolu         : apps/system/compat/src/compat_manager.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32, RISC-V 64, MIPS 32, MIPS 64, PowerPC 32, PowerPC 64, m68k, SPARC, LoongArch64
//
// Açıklama:
//   Tüm uyumluluk alt sistemlerini (DOS, Win32, Darwin, Android,
//   Linux ELF, Java/JVM, WebAssembly, Wine) tek bir noktadan yöneten
//   orchestrator. Magic byte analizi ile otomatik platform tespiti,
//   cross-platform API köprüsü ve sandbox başlatma sağlar.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs
//   2-) apps/system/compat/src/win32.rs
//   3-) apps/system/compat/src/darwin.rs
//   4-) apps/system/compat/src/android.rs
//   5-) apps/system/compat/src/wine_server.rs
//   6-) apps/system/compat/src/linux_elf.rs
//   7-) apps/system/compat/src/java_bytecode.rs
//   8-) apps/system/compat/src/wasm_runtime.rs
//   9-) kernel/graphics/ui/src/lang/lang_flags.rs (MsgId)
//
//              Dosyaya Müdahaleler
// 2026-04-17      Crate oluşturuldu, tüm modüller bağlandı
// 2026-04-18      Lang sistemi, hata tipleri, güven skoru,
//                 gelişmiş binary detector, cross-platform API köprüsü
// *******************************************************************

#![allow(dead_code)]

use crate::dos_emulator::console_writeln;
use crate::dos_emulator::{DosEmulator, DosExeLoader};
use crate::android::{ApkManager, ZipParser};
use crate::win32::{Win32Manager, PeParser, PeLoader};
use crate::win_layer::api_emulator::Win32ApiEmulator;
use crate::darwin::{DarwinManager, OzMachOLoader};
use crate::core_services::{LaunchServicesDb, UtiRegistry};
use crate::wine_server::{WineObjectManager, WineRegistryHive, WineDllManager};
use crate::linux_elf::{ElfLoader, LinuxProcess};
use crate::java_bytecode::{ClassFileParser, JavaRuntime};
use crate::wasm_runtime::{WasmModuleParser, WasmRuntime};
use kernel_ui::{Lang, MsgId};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// ─── Hata Tipi ──────────────────────────────────────────────────

/// Uyumluluk katmanı hata türleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatError {
    /// İkili veri çok kısa, platform tespit edilemedi.
    DataTooShort,
    /// Bilinmeyen / desteklenmeyen format.
    UnknownFormat,
    /// Yürütme dosyası bozuk.
    CorruptBinary,
    /// Yeterli bellek yok.
    OutOfMemory,
    /// Platform için gerekli alt sistem başlatılmamış.
    SubsystemNotReady,
    /// İzin reddedildi (sandbox kısıtlaması).
    PermissionDenied,
}

impl CompatError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DataTooShort       => "data too short to detect platform",
            Self::UnknownFormat      => "unknown or unsupported binary format",
            Self::CorruptBinary      => "binary is corrupt or truncated",
            Self::OutOfMemory        => "insufficient memory",
            Self::SubsystemNotReady  => "required subsystem not initialized",
            Self::PermissionDenied   => "permission denied by sandbox policy",
        }
    }
}

// ─── Platform Enum ──────────────────────────────────────────────

/// Tespit edilen yürütme platformu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Dos,
    Win32,
    Darwin,
    Android,
    Linux,
    Java,
    Wasm,
    Wine,
    Unknown,
}

impl Platform {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Dos     => "DOS",
            Self::Win32   => "Win32/PE",
            Self::Darwin  => "Darwin/Mach-O",
            Self::Android => "Android/APK",
            Self::Linux   => "Linux/ELF",
            Self::Java    => "Java/JVM",
            Self::Wasm    => "WebAssembly",
            Self::Wine    => "Wine",
            Self::Unknown => "Bilinmiyor",
        }
    }

    /// Platform, sandbox içinde çalıştırılmalı mı?
    #[must_use]
    pub fn requires_sandbox(self) -> bool {
        matches!(self, Self::Win32 | Self::Android | Self::Wine | Self::Dos)
    }
}

// ─── İkili Format Dedektörü ──────────────────────────────────────

/// İkili veri analizine dayalı platform tespiti.
///
/// Güven skoru:
/// - 100 → kesin (benzersiz magic bytes)
/// - 80  → yüksek (magic + alan doğrulaması)
/// - 50  → orta  (magic mevcut ama kırpık veri)
/// - 0   → bilinmiyor
pub struct BinaryDetector;

impl BinaryDetector {
    const MIN_DATA: usize = 4;

    /// Platform tespiti + güven skoru. En az 4 byte gereklidir.
    #[must_use]
    pub fn detect_with_confidence(data: &[u8]) -> (Platform, u8) {
        if data.len() < Self::MIN_DATA {
            return (Platform::Unknown, 0);
        }

        let m = &data[..4.min(data.len())];

        // ELF — Linux (7F 45 4C 46)
        if m == [0x7F, b'E', b'L', b'F'] {
            let conf = if data.len() >= 64 { 100 } else { 50 };
            return (Platform::Linux, conf);
        }

        // Java Class — CAFE BABE
        if m == [0xCA, 0xFE, 0xBA, 0xBE] {
            // Sihirli sözcük JVM class ve FAT Mach-O her ikisinde de var.
            // JVM class'ı: byte[7] sürüm numarası (> 30 modern Java)
            // FAT Mach-O : byte[4..8] arch sayısı (genelde <= 4)
            if data.len() >= 8 {
                let major = u16::from_be_bytes([data[6], data[7]]);
                if major >= 45 { // Java 1.1 = 45
                    return (Platform::Java, 100);
                }
            }
            return (Platform::Java, 80);
        }

        // Mach-O — Darwin (little-endian 64-bit: CE FA ED FE)
        if m == [0xCE, 0xFA, 0xED, 0xFE] {
            return (Platform::Darwin, 100);
        }
        // Mach-O — Darwin (big-endian 64-bit: FE ED FA CE)
        if m == [0xFE, 0xED, 0xFA, 0xCE] {
            return (Platform::Darwin, 100);
        }
        // Mach-O — Darwin (little-endian 32-bit: CE FA ED FE, farklı)
        if m == [0xFE, 0xED, 0xFA, 0xCF] || m == [0xCF, 0xFA, 0xED, 0xFE] {
            return (Platform::Darwin, 100);
        }
        // FAT Mach-O (CA FE BA BE, arch_count <= 4)
        if m == [0xCA, 0xFE, 0xBA, 0xBE] && data.len() >= 8 {
            let count = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
            if count <= 4 {
                return (Platform::Darwin, 90);
            }
        }

        // WASM (00 61 73 6D = \0asm)
        if m == [0x00, 0x61, 0x73, 0x6D] {
            if data.len() >= 8 {
                let ver = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                if ver == 1 { return (Platform::Wasm, 100); }
            }
            return (Platform::Wasm, 80);
        }

        // MZ — DOS veya Win32/PE
        if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A {
            // PE imzası kontrol et
            if data.len() >= 0x40 {
                let pe_off = u32::from_le_bytes([
                    data[0x3C], data[0x3D], data[0x3E], data[0x3F],
                ]) as usize;
                if pe_off + 4 <= data.len()
                    && data[pe_off]     == b'P'
                    && data[pe_off + 1] == b'E'
                    && data[pe_off + 2] == 0
                    && data[pe_off + 3] == 0
                {
                    return (Platform::Win32, 100);
                }
            }
            // Sadece MZ — DOS COM/EXE
            return (Platform::Dos, 80);
        }

        // ZIP / APK — PK\x03\x04
        if m == [0x50, 0x4B, 0x03, 0x04] {
            return (Platform::Android, 90);
        }

        (Platform::Unknown, 0)
    }

    /// Yalnızca platform döndür.
    #[must_use]
    pub fn detect(data: &[u8]) -> Platform {
        Self::detect_with_confidence(data).0
    }

    /// Tespit bilgisini konsola yaz.
    pub fn dump_info(data: &[u8]) {
        if data.len() < 8 {
            console_writeln(&Lang::get(MsgId::CompatDetectShort));
            return;
        }
        let (platform, confidence) = Self::detect_with_confidence(data);
        let msg = format!("[Tespit] Platform: {}  güven={}%  magic={:02X}{:02X}{:02X}{:02X}",
            platform.name(), confidence,
            data[0], data[1], data[2], data[3]);
        console_writeln(&msg);
    }
}

// ─── Cross-Platform API Köprüsü ──────────────────────────────────

/// Platform bağımsız API çağrısı temsili.
#[derive(Debug, Clone)]
pub enum CrossApiCall {
    FileOpen    { path: String, flags: u32, mode: u32 },
    FileRead    { fd: u64, size: usize },
    FileWrite   { fd: u64, data: Vec<u8> },
    FileClose   { fd: u64 },
    MemAlloc    { size: usize, prot: u32 },
    MemFree     { addr: u64, size: usize },
    MemProtect  { addr: u64, size: usize, prot: u32 },
    ProcCreate  { path: String, args: Vec<String>, env: Vec<String> },
    ProcExit    { code: i32 },
    ThreadCreate{ entry: u64, stack: u64, arg: u64 },
    ThreadSleep { ns: u64 },
    SockCreate  { domain: i32, type_: i32, protocol: i32 },
    SockConnect { fd: u64, addr: String, port: u16 },
    TimeGet,
    Print       { text: String },
    Unknown     { id: u64 },
}

impl CrossApiCall {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::FileOpen    { .. } => "FileOpen",
            Self::FileRead    { .. } => "FileRead",
            Self::FileWrite   { .. } => "FileWrite",
            Self::FileClose   { .. } => "FileClose",
            Self::MemAlloc    { .. } => "MemAlloc",
            Self::MemFree     { .. } => "MemFree",
            Self::MemProtect  { .. } => "MemProtect",
            Self::ProcCreate  { .. } => "ProcCreate",
            Self::ProcExit    { .. } => "ProcExit",
            Self::ThreadCreate{ .. } => "ThreadCreate",
            Self::ThreadSleep { .. } => "ThreadSleep",
            Self::SockCreate  { .. } => "SockCreate",
            Self::SockConnect { .. } => "SockConnect",
            Self::TimeGet            => "TimeGet",
            Self::Print       { .. } => "Print",
            Self::Unknown     { .. } => "Unknown",
        }
    }
}

pub struct ApiBridge;

impl ApiBridge {
    /// API çağrısını dağıt ve sonucu döndür.
    #[must_use]
    pub fn dispatch(call: &CrossApiCall) -> i64 {
        let prefix = &Lang::get(MsgId::CompatApiDispatch);
        match call {
            CrossApiCall::FileOpen { path, flags, mode } => {
                let msg = format!("{} FileOpen(\"{}\", flags={:#x}, mode={:#o}) → fd=3",
                    prefix, path, flags, mode);
                console_writeln(&msg); 3
            }
            CrossApiCall::FileRead { fd, size } => {
                let msg = format!("{} FileRead(fd={}, {} bytes)", prefix, fd, size);
                console_writeln(&msg); *size as i64
            }
            CrossApiCall::FileWrite { fd, data } => {
                let msg = format!("{} FileWrite(fd={}, {} bytes)", prefix, fd, data.len());
                console_writeln(&msg); data.len() as i64
            }
            CrossApiCall::FileClose { fd } => {
                let msg = format!("{} FileClose(fd={})", prefix, fd);
                console_writeln(&msg); 0
            }
            CrossApiCall::MemAlloc { size, prot } => {
                let msg = format!("{} MemAlloc(size={}, prot={:#x}) → 0x1000_0000",
                    prefix, size, prot);
                console_writeln(&msg); 0x1000_0000
            }
            CrossApiCall::MemFree { addr, size } => {
                let msg = format!("{} MemFree(addr={:#x}, size={})", prefix, addr, size);
                console_writeln(&msg); 0
            }
            CrossApiCall::MemProtect { addr, size, prot } => {
                let msg = format!("{} MemProtect(addr={:#x}, size={}, prot={:#x})",
                    prefix, addr, size, prot);
                console_writeln(&msg); 0
            }
            CrossApiCall::ProcCreate { path, args, env } => {
                let msg = format!("{} ProcCreate(\"{}\", {} args, {} env) → pid=1",
                    prefix, path, args.len(), env.len());
                console_writeln(&msg); 1
            }
            CrossApiCall::ProcExit { code } => {
                let msg = format!("{} ProcExit({})", prefix, code);
                console_writeln(&msg); *code as i64
            }
            CrossApiCall::ThreadCreate { entry, stack, arg } => {
                let msg = format!("{} ThreadCreate(entry={:#x}, stack={:#x}, arg={:#x}) → tid=2",
                    prefix, entry, stack, arg);
                console_writeln(&msg); 2
            }
            CrossApiCall::ThreadSleep { ns } => {
                let msg = format!("{} ThreadSleep({} ns)", prefix, ns);
                console_writeln(&msg); 0
            }
            CrossApiCall::SockCreate { domain, type_, protocol } => {
                let msg = format!("{} SockCreate(dom={}, type={}, proto={}) → fd=4",
                    prefix, domain, type_, protocol);
                console_writeln(&msg); 4
            }
            CrossApiCall::SockConnect { fd, addr, port } => {
                let msg = format!("{} SockConnect(fd={}, {}:{}) → 0", prefix, fd, addr, port);
                console_writeln(&msg); 0
            }
            CrossApiCall::TimeGet => {
                let msg = format!("{} TimeGet() → 0x0000_0000_0000_0001", prefix);
                console_writeln(&msg); 1
            }
            CrossApiCall::Print { text } => {
                let msg = format!("{} Print: {}", prefix, text);
                console_writeln(&msg); text.len() as i64
            }
            CrossApiCall::Unknown { id } => {
                let msg = format!("{} Unknown(id={:#x}) → -1", prefix, id);
                console_writeln(&msg); -1
            }
        }
    }

    /// Linux syscall numarasını CrossApiCall'a çevir.
    #[must_use]
    pub fn from_linux_syscall(nr: u64, a1: u64, a2: u64, a3: u64) -> CrossApiCall {
        match nr {
            0  => CrossApiCall::FileRead  { fd: a1, size: a3 as usize },
            1  => CrossApiCall::FileWrite { fd: a1, data: Vec::new() },
            2  => CrossApiCall::FileOpen  { path: String::new(), flags: a2 as u32, mode: a3 as u32 },
            3  => CrossApiCall::FileClose { fd: a1 },
            9  => CrossApiCall::MemAlloc  { size: a2 as usize, prot: 3 },
            11 => CrossApiCall::MemFree   { addr: a1, size: a2 as usize },
            39 => CrossApiCall::ProcCreate{ path: String::new(), args: Vec::new(), env: Vec::new() },
            41 => CrossApiCall::SockCreate{ domain: a1 as i32, type_: a2 as i32, protocol: a3 as i32 },
            42 => CrossApiCall::SockConnect{ fd: a1, addr: String::new(), port: 0 },
            60 | 231 => CrossApiCall::ProcExit { code: a1 as i32 },
            _  => CrossApiCall::Unknown   { id: nr },
        }
    }

    /// Win32 API numarasını CrossApiCall'a çevir.
    #[must_use]
    pub fn from_win32_api(api: u32, args: &[u64]) -> CrossApiCall {
        let a = |i: usize| args.get(i).copied().unwrap_or(0);
        match api {
            0x0100 => CrossApiCall::FileOpen  { path: String::new(), flags: a(1) as u32, mode: a(2) as u32 },
            0x0101 => CrossApiCall::FileRead  { fd: a(0), size: a(1) as usize },
            0x0102 => CrossApiCall::FileWrite { fd: a(0), data: Vec::new() },
            0x0103 => CrossApiCall::FileClose { fd: a(0) },
            0x0200 => CrossApiCall::MemAlloc  { size: a(0) as usize, prot: a(1) as u32 },
            0x0201 => CrossApiCall::MemFree   { addr: a(0), size: a(1) as usize },
            0x0202 => CrossApiCall::MemProtect{ addr: a(0), size: a(1) as usize, prot: a(2) as u32 },
            0x0300 => CrossApiCall::ProcCreate{ path: String::new(), args: Vec::new(), env: Vec::new() },
            0x0301 => CrossApiCall::ProcExit  { code: a(0) as i32 },
            0x0400 => CrossApiCall::ThreadCreate{ entry: a(0), stack: a(1), arg: a(2) },
            0x0401 => CrossApiCall::ThreadSleep { ns: a(0) * 1_000_000 },
            0x0500 => CrossApiCall::SockCreate  { domain: a(0) as i32, type_: a(1) as i32, protocol: a(2) as i32 },
            0x0501 => CrossApiCall::SockConnect { fd: a(0), addr: String::new(), port: a(1) as u16 },
            0x0600 => CrossApiCall::Print { text: String::new() },
            _      => CrossApiCall::Unknown { id: api as u64 },
        }
    }
}

// ─── Uyumluluk Yöneticisi ────────────────────────────────────────

/// Tüm uyumluluk alt sistemlerini kapsayan ana yapı.
pub struct CompatManager {
    pub dos:             DosEmulator,
    pub win32:           Win32Manager,
    pub darwin:          DarwinManager,
    pub android:         ApkManager,
    pub wine_objects:    WineObjectManager,
    pub wine_registry:   WineRegistryHive,
    pub wine_dlls:       WineDllManager,
    pub linux_process:   LinuxProcess,
    pub java_runtime:    JavaRuntime,
    pub wasm_runtime:    WasmRuntime,
    pub launch_services: LaunchServicesDb,
    pub uti_registry:    UtiRegistry,
}

impl CompatManager {
    /// Tüm alt sistemleri başlat.
    #[must_use]
    pub fn new() -> Self {
        console_writeln(&Lang::get(MsgId::CompatMgrInit));
        Self {
            dos:             DosEmulator::new(),
            win32:           Win32Manager::new(),
            darwin:          DarwinManager::new(),
            android:         ApkManager::new(),
            wine_objects:    WineObjectManager::new(),
            wine_registry:   WineRegistryHive::new(),
            wine_dlls:       WineDllManager::new(),
            linux_process:   LinuxProcess::new(1),
            java_runtime:    JavaRuntime::new(),
            wasm_runtime:    WasmRuntime::new(),
            launch_services: LaunchServicesDb::new(),
            uti_registry:    UtiRegistry::new(),
        }
    }

    /// İkili veriyi analiz et ve uygun alt sisteme yükle.
    /// Tespit edilen platformu döndürür.
    #[must_use]
    pub fn detect_and_load(&mut self, data: &[u8]) -> Result<Platform, CompatError> {
        if data.len() < BinaryDetector::MIN_DATA {
            return Err(CompatError::DataTooShort);
        }

        let (platform, confidence) = BinaryDetector::detect_with_confidence(data);
        let msg = format!("{}: {}  güven={}%",
            &Lang::get(MsgId::CompatMgrDetect), platform.name(), confidence);
        console_writeln(&msg);

        if platform == Platform::Unknown {
            console_writeln(&Lang::get(MsgId::CompatMgrUnknown));
            return Err(CompatError::UnknownFormat);
        }

        match platform {
            Platform::Dos => {
                if data.len() >= 2 && data[0] == 0x4D && data[1] == 0x5A
                    && data.len() > 28
                {
                    if let Some((mz, relocs, _)) = DosExeLoader::load(data) {
                        let msg = format!("[DOS] EXE: {} paragraf başlık, {} yer değiştirme",
                            mz.e_cparhdr, relocs.len());
                        console_writeln(&msg);
                        let _ = self.dos.load_exe(data);
                    }
                } else {
                    let _ = self.dos.load_com(data);
                }
            }

            Platform::Win32 => {
                let pe = PeParser::parse(data);
                if !pe.is_valid { return Err(CompatError::CorruptBinary); }
                let msg = format!("[PE] {} bölüm, ImageBase=0x{:016X}, Ep=0x{:08X}",
                    pe.sections.len(),
                    pe.optional_header.image_base,
                    pe.optional_header.address_of_entry_point);
                console_writeln(&msg);
                let mut mapped = PeLoader::map_image(data, &pe, pe.optional_header.image_base);
                PeLoader::apply_relocations(&mut mapped, &pe, pe.optional_header.image_base);
                PeLoader::resolve_imports(&mut mapped, &pe);
                let msg2 = format!("[PE] Yüklendi: {} byte @ 0x{:016X}, {} reloc, {} import",
                    mapped.memory.len(),
                    mapped.base_address,
                    mapped.relocs_applied,
                    mapped.imports_resolved.len());
                console_writeln(&msg2);
            }

            Platform::Darwin => {
                let base = 0x0000_1000_0000_0000u64;
                let mut cache = crate::darwin::DyldCache::new();
                let img = OzMachOLoader::load_and_link("", data, &mut cache, base);
                let msg = format!("[Mach-O] 0x{:016X}, {} segment, {} export",
                    base, img.segments.len(), img.exports.len());
                console_writeln(&msg);
            }

            Platform::Android => {
                let entries = ZipParser::parse_central_directory(data);
                let msg = format!("[APK] {} giriş", entries.len());
                console_writeln(&msg);
            }

            Platform::Linux => {
                let base = 0x0000_4000_0000u64;
                if let Some(img) = ElfLoader::load_and_map(data, base) {
                    let msg = format!("[ELF64] {} byte @ 0x{:016X}, {} phdr, ep=0x{:016X}",
                        img.memory.len(), base, img.phdrs.len(), img.header.e_entry);
                    console_writeln(&msg);
                    self.linux_process.image = Some(img);
                } else {
                    return Err(CompatError::CorruptBinary);
                }
            }

            Platform::Java => {
                if let Some(cf) = ClassFileParser::parse(data) {
                    let msg = format!("[JVM] {} metot, {} alan, sürüm {}.{}",
                        cf.methods.len(), cf.fields.len(),
                        cf.header.major, cf.header.minor);
                    console_writeln(&msg);
                    let _ = self.java_runtime.load_class(data);
                } else {
                    return Err(CompatError::CorruptBinary);
                }
            }

            Platform::Wasm => {
                if let Some(module) = WasmModuleParser::parse(data) {
                    let msg = format!("[WASM] {} tür, {} export, {} işlev gövdesi",
                        module.types.len(), module.exports.len(), module.func_bodies.len());
                    console_writeln(&msg);
                    let _ = self.wasm_runtime.load(data);
                } else {
                    return Err(CompatError::CorruptBinary);
                }
            }

            Platform::Wine | Platform::Unknown => {
                console_writeln(&Lang::get(MsgId::CompatMgrUnknown));
                return Err(CompatError::UnknownFormat);
            }
        }

        Ok(platform)
    }

    /// Tüm alt sistemlerin durumunu konsola yaz.
    pub fn dump_status(&self) {
        console_writeln(&Lang::get(MsgId::CompatMgrStatus));
        let lines: &[(&str, String)] = &[
            ("DOS",     format!("{} dosya, {} bellek bloğu",
                self.dos.files.len(),
                self.dos.memory.iter().filter(|m| m.owner != 0).count())),
            ("Win32",   format!("{} uygulama", self.win32.apps.len())),
            ("Darwin",  format!("{} uygulama, {} kext", self.darwin.apps.len(), self.darwin.kexts.len())),
            ("Android", format!("{} APK", self.android.apps.len())),
            ("Wine",    format!("{} nesne, {} DLL",
                self.wine_objects.count(),
                self.wine_dlls.loaded_count())),
            ("Linux",   format!("pid={} brk=0x{:X}", self.linux_process.pid, self.linux_process.brk)),
            ("Java",    format!("{} sınıf", self.java_runtime.classes.len())),
            ("WASM",    format!("{} modül", self.wasm_runtime.modules.len())),
        ];
        for (name, info) in lines {
            let msg = format!("  {:8}: {}", name, info);
            console_writeln(&msg);
        }
        console_writeln(&Lang::get(MsgId::CompatMgrStatusEnd));
    }

    /// Win32 EXE'yi sandbox içinde çalıştır.
    pub fn run_win32(&mut self, path: &str) {
        if path.is_empty() { return; }
        let msg = format!("{}: {}...", &Lang::get(MsgId::CompatRunWin), path);
        console_writeln(&msg);

        let handle = Win32ApiEmulator::create_file(&mut self.win32, path, 0x4000_0000, 0, 2, 0);
        if handle != 0xFFFF_FFFF {
            let stub = b"MZ\x90\x00PE\x00\x00"; // Stub PE başlığı
            Win32ApiEmulator::write_file(&mut self.win32, handle, stub);
            Win32ApiEmulator::close_handle(&mut self.win32, handle);
        }

        let pid = self.win32.create_process(path);
        let msg2 = format!("[Win32] Süreç başlatıldı: PID={}", pid);
        console_writeln(&msg2);

        let ev = Win32ApiEmulator::create_event(&mut self.win32, false, false);
        Win32ApiEmulator::set_event(&mut self.win32, ev);
        let msg3 = format!("[Win32] Olay oluşturuldu: handle=0x{:08X}", ev);
        console_writeln(&msg3);
        Win32ApiEmulator::close_handle(&mut self.win32, ev);
    }
}

impl Default for CompatManager {
    fn default() -> Self { Self::new() }
}

// ─── Birim Testler ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── BinaryDetector ────────────────────────────────────────────

    #[test]
    fn test_detect_elf() {
        let data = [0x7F, b'E', b'L', b'F', 2, 1, 1, 0,
                    0,0,0,0,0,0,0,0,
                    2,0, 62,0, 1,0,0,0,  // ET_EXEC, EM_X86_64
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0];
        let (p, c) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Linux);
        assert!(c >= 80);
    }

    #[test]
    fn test_detect_wasm() {
        let data = [0x00, 0x61, 0x73, 0x6D, 1, 0, 0, 0];
        let (p, c) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Wasm);
        assert_eq!(c, 100);
    }

    #[test]
    fn test_detect_apk_zip() {
        let data = [0x50, 0x4B, 0x03, 0x04, 0, 0, 0, 0];
        let (p, _) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Android);
    }

    #[test]
    fn test_detect_java_class() {
        // CAFEBABE + sürüm 61 (Java 17)
        let data = [0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 61];
        let (p, c) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Java);
        assert!(c >= 80);
    }

    #[test]
    fn test_detect_darwin_macho() {
        let data = [0xCE, 0xFA, 0xED, 0xFE, 7, 0, 0, 1]; // Mach-O 64-bit LE
        let (p, c) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Darwin);
        assert_eq!(c, 100);
    }

    #[test]
    fn test_detect_mz_dos() {
        let data = [0x4D, 0x5A, 0x90, 0x00, 0,0,0,0,
                    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0];
        // PE offset geçersizse DOS
        let (p, _) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Dos);
    }

    #[test]
    fn test_detect_too_short() {
        let (p, c) = BinaryDetector::detect_with_confidence(&[0x7F, 0x45]);
        assert_eq!(p, Platform::Unknown);
        assert_eq!(c, 0);
    }

    #[test]
    fn test_detect_unknown() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let (p, c) = BinaryDetector::detect_with_confidence(&data);
        assert_eq!(p, Platform::Unknown);
        assert_eq!(c, 0);
    }

    // ── Platform ─────────────────────────────────────────────────

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::Linux.name(), "Linux/ELF");
        assert_eq!(Platform::Win32.name(), "Win32/PE");
        assert_eq!(Platform::Wasm.name(),  "WebAssembly");
    }

    #[test]
    fn test_platform_sandbox() {
        assert!(Platform::Win32.requires_sandbox());
        assert!(Platform::Android.requires_sandbox());
        assert!(!Platform::Linux.requires_sandbox());
        assert!(!Platform::Wasm.requires_sandbox());
    }

    // ── CrossApiCall ─────────────────────────────────────────────

    #[test]
    fn test_api_bridge_file_ops() {
        let read = CrossApiCall::FileRead { fd: 3, size: 4096 };
        assert_eq!(ApiBridge::dispatch(&read), 4096);

        let close = CrossApiCall::FileClose { fd: 3 };
        assert_eq!(ApiBridge::dispatch(&close), 0);
    }

    #[test]
    fn test_api_bridge_mem_alloc() {
        let alloc = CrossApiCall::MemAlloc { size: 65536, prot: 3 };
        assert_ne!(ApiBridge::dispatch(&alloc), 0);
    }

    #[test]
    fn test_api_bridge_proc_exit() {
        let exit = CrossApiCall::ProcExit { code: 42 };
        assert_eq!(ApiBridge::dispatch(&exit), 42);
    }

    #[test]
    fn test_from_linux_syscall() {
        let call = ApiBridge::from_linux_syscall(60, 0, 0, 0); // exit
        assert!(matches!(call, CrossApiCall::ProcExit { code: 0 }));

        let read = ApiBridge::from_linux_syscall(0, 3, 0, 512); // read
        assert!(matches!(read, CrossApiCall::FileRead { fd: 3, size: 512 }));
    }

    #[test]
    fn test_from_win32_api() {
        let args = [3u64, 512, 0, 0];
        let read = ApiBridge::from_win32_api(0x0101, &args);
        assert!(matches!(read, CrossApiCall::FileRead { fd: 3, size: 512 }));
    }
}
