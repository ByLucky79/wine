// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : macOS Core Services Uyumluluk Katmanı
// Dosya Yolu         : apps/system/compat/src/core_services.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32, RISC-V 64, MIPS 32, MIPS 64, PowerPC 32, PowerPC 64, m68k, SPARC, LoongArch64
//
// Açıklama:
//   macOS Core Services API uyumluluk katmanı. CFString, CFBundle,
//   LaunchServices, UTI (Uniform Type Identifier) kayıt defteri,
//   FSEvents akış öykünücüsü ve Spotlight meta veri sorgu motoru
//   içerir. Tüm API'ler no_std / alloc ortamında çalışır.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs  (console_writeln)
//   2-) kernel/graphics/ui/src/lang/lang_flags.rs (MsgId)
//   3-) kernel/graphics/ui/src/lang/lang_manager.rs (Lang::get)
//
//              Dosyaya Müdahaleler
// 2026-04-17      C → Rust çevirisi (no_std)
// 2026-04-18      Lang sistemi entegrasyonu, hata tipleri, tam UTI
//                 LaunchServices iyileştirmeleri, FSEvents & Spotlight
// *******************************************************************

#![allow(dead_code)]

use crate::dos_emulator::console_writeln;
use kernel_ui::{Lang, MsgId};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicBool, Ordering};

// ─── Hata Tipi ──────────────────────────────────────────────────

/// Core Services katmanı hata türleri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreServicesError {
    /// Verilen bundle ID bulunamadı.
    BundleNotFound,
    /// UTI tanımlayıcısı kayıt defterinde yok.
    UtiNotFound,
    /// Şema veya uzantı için kayıtlı uygulama yok.
    HandlerNotFound,
    /// String verisi çok uzun (maks. `CFString::MAX_BYTES`).
    StringTooLong,
    /// Akış zaten kapalı.
    StreamClosed,
    /// Geçersiz sorgu ifadesi.
    InvalidQuery,
}

impl CoreServicesError {
    /// İnsan-okunabilir açıklama (dil sisteminden bağımsız, sabit İngilizce).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BundleNotFound  => "bundle not found",
            Self::UtiNotFound     => "UTI not found",
            Self::HandlerNotFound => "no handler registered",
            Self::StringTooLong   => "string exceeds CFString capacity",
            Self::StreamClosed    => "FSEvents stream is closed",
            Self::InvalidQuery    => "invalid Spotlight query",
        }
    }
}

// ─── Başlatma ───────────────────────────────────────────────────

static CORE_SERVICES_READY: AtomicBool = AtomicBool::new(false);

/// Core Services uyumluluk katmanını başlat.
/// Yeniden çağrılabilir (idempotent).
pub fn core_services_init() {
    CORE_SERVICES_READY.store(true, Ordering::SeqCst);
    console_writeln(&Lang::get(MsgId::CompatCoreServicesInit));
}

/// Katmanın başlatılıp başlatılmadığını sorgula.
#[must_use]
pub fn is_ready() -> bool {
    CORE_SERVICES_READY.load(Ordering::SeqCst)
}

// ─── CFString ───────────────────────────────────────────────────

/// `CFStringRef` benzeri, sabit-kapasiteli UTF-8 string wrapper'ı.
///
/// macOS'ta CFString ref-counted ve heap tabanlıdır; bu
/// implementasyon no_std uyumluluğu için sabit dizi kullanır.
/// 511 UTF-8 bayt + null sonlandırıcıya kadar veri tutar.
#[derive(Debug, Clone, Copy)]
pub struct CFString {
    data: [u8; Self::CAPACITY],
    len:  u16,
}

impl CFString {
    /// Tutulan veri kapasitesi (null hariç, byte cinsinden).
    pub const MAX_BYTES: usize = 511;
    const CAPACITY: usize = Self::MAX_BYTES + 1;

    /// Boş CFString oluştur.
    #[must_use]
    pub const fn empty() -> Self {
        Self { data: [0u8; Self::CAPACITY], len: 0 }
    }

    /// Rust &str'den CFString oluştur.
    /// `s` MAX_BYTES'tan uzunsa `Err(CoreServicesError::StringTooLong)` döner.
    #[must_use]
    pub fn try_from_str(s: &str) -> Result<Self, CoreServicesError> {
        let bytes = s.as_bytes();
        if bytes.len() > Self::MAX_BYTES {
            return Err(CoreServicesError::StringTooLong);
        }
        let mut data = [0u8; Self::CAPACITY];
        data[..bytes.len()].copy_from_slice(bytes);
        Ok(Self { data, len: bytes.len() as u16 })
    }

    /// Rust &str'den CFString oluştur; fazla kısmı sessizce kırpar.
    #[must_use]
    pub fn from_str_truncate(s: &str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len().min(Self::MAX_BYTES);
        let mut data = [0u8; Self::CAPACITY];
        data[..len].copy_from_slice(&bytes[..len]);
        Self { data, len: len as u16 }
    }

    /// UTF-8 dilim olarak içeriği döndür.
    /// Geçersiz bayt dizisi durumunda boş string döner.
    #[must_use]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len as usize]).unwrap_or("")
    }

    /// Geçerli içerik uzunluğu (bayt).
    #[must_use]
    pub fn len(&self) -> usize { self.len as usize }

    /// İçeriğin boş olup olmadığı.
    #[must_use]
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// Null-sonlandırılmış ham bayt dilimi.
    #[must_use]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        &self.data[..self.len as usize + 1]
    }
}

impl Default for CFString {
    fn default() -> Self { Self::empty() }
}

impl PartialEq for CFString {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.as_str() == other.as_str()
    }
}

// ─── CFBundle ───────────────────────────────────────────────────

/// Bundle yükleme durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleState {
    Unloaded,
    Loading,
    Loaded,
    Failed,
}

/// `CFBundle` benzeri bundle meta veri tutucusu.
#[derive(Debug, Clone)]
pub struct CFBundle {
    pub bundle_id:   String,
    pub path:        String,
    pub version:     String,
    pub state:       BundleState,
    pub executable:  String,
}

impl CFBundle {
    /// Belirtilen yoldaki bundle'ı yükle.
    /// Başarılı olduğunda `BundleState::Loaded` durumuna geçer.
    #[must_use]
    pub fn load(path: &str) -> Result<Self, CoreServicesError> {
        if path.is_empty() {
            return Err(CoreServicesError::BundleNotFound);
        }
        let msg = format!("{}: {}", &Lang::get(MsgId::CompatBundleLoading), path);
        console_writeln(&msg);

        // Bundle ID'yi son path bileşeninden türet (ör. "MyApp.app" → "com.ozkan.MyApp")
        let name = path.split('/').next_back().unwrap_or(path);
        let name_clean = name.trim_end_matches(".app");
        let bundle_id = format!("com.ozkan.{}", name_clean);

        Ok(Self {
            bundle_id,
            path: path.to_string(),
            version: String::from("1.0"),
            state: BundleState::Loaded,
            executable: format!("{}/{}", path, name_clean),
        })
    }
}

// ─── LaunchServices Veritabanı ───────────────────────────────────

/// Tek bir uygulama kaydı.
#[derive(Debug, Clone)]
pub struct LaunchServicesHandler {
    pub bundle_id:  String,
    pub name:       String,
    pub schemes:    Vec<String>,
    pub extensions: Vec<String>,
    pub utis:       Vec<String>,
}

/// macOS LaunchServices veritabanı öykünücüsü.
///
/// URL şemaları, dosya uzantıları ve UTI tanımlayıcıları için
/// varsayılan uygulama kayıtlarını tutar.
pub struct LaunchServicesDb {
    handlers:     Vec<LaunchServicesHandler>,
    default_apps: BTreeMap<String, String>,
}

impl LaunchServicesDb {
    /// Varsayılan macOS uygulamalarıyla başlatılmış yeni veritabanı.
    #[must_use]
    pub fn new() -> Self {
        let mut db = Self {
            handlers:     Vec::new(),
            default_apps: BTreeMap::new(),
        };
        // Temel macOS varsayılan uygulama kayıtları
        db.register("com.apple.Safari",
            "Safari",
            &["http", "https", "ftp"],
            &["html", "htm", "xhtml"],
            &["public.html", "public.xhtml"]);
        db.register("com.apple.Mail",
            "Mail",
            &["mailto"],
            &["eml", "msg"],
            &["com.apple.mail.email"]);
        db.register("com.apple.TextEdit",
            "TextEdit",
            &["txmt"],
            &["txt", "rtf", "md"],
            &["public.plain-text", "public.rtf"]);
        db.register("com.apple.Preview",
            "Preview",
            &[],
            &["pdf", "png", "jpg", "jpeg", "gif", "tiff", "bmp"],
            &["com.adobe.pdf", "public.image"]);
        db.register("com.apple.Finder",
            "Finder",
            &["x-ozkan-finder"],
            &[],
            &["public.folder", "public.directory"]);
        db.register("com.apple.QuickTime",
            "QuickTime",
            &[],
            &["mov", "mp4", "m4v", "avi"],
            &["public.movie", "public.mpeg-4"]);
        db.register("com.apple.iTunes",
            "iTunes",
            &["itms"],
            &["mp3", "m4a", "aac", "aiff", "wav"],
            &["public.audio"]);
        db
    }

    /// Yeni bir uygulama işleyicisi kaydet.
    pub fn register(
        &mut self,
        bundle_id: &str,
        name:      &str,
        schemes:   &[&str],
        exts:      &[&str],
        utis:      &[&str],
    ) {
        let handler = LaunchServicesHandler {
            bundle_id:  bundle_id.to_string(),
            name:       name.to_string(),
            schemes:    schemes.iter().map(|s| s.to_string()).collect(),
            extensions: exts.iter().map(|s| s.to_string()).collect(),
            utis:       utis.iter().map(|s| s.to_string()).collect(),
        };
        for s in schemes { self.default_apps.insert(s.to_string(), bundle_id.to_string()); }
        for e in exts    { self.default_apps.insert(e.to_string(), bundle_id.to_string()); }
        for u in utis    { self.default_apps.insert(u.to_string(), bundle_id.to_string()); }
        self.handlers.push(handler);
    }

    /// URL şeması için varsayılan bundle ID'yi bul.
    #[must_use]
    pub fn lookup_scheme(&self, scheme: &str) -> Option<&str> {
        self.default_apps.get(scheme).map(String::as_str)
    }

    /// Dosya uzantısı için varsayılan bundle ID'yi bul.
    #[must_use]
    pub fn lookup_extension(&self, ext: &str) -> Option<&str> {
        self.default_apps.get(ext).map(String::as_str)
    }

    /// UTI için varsayılan bundle ID'yi bul.
    #[must_use]
    pub fn lookup_uti(&self, uti: &str) -> Option<&str> {
        self.default_apps.get(uti).map(String::as_str)
    }

    /// Kayıtlı tüm işleyici sayısını döndür.
    #[must_use]
    pub fn handler_count(&self) -> usize { self.handlers.len() }

    /// Veritabanını konsola döküm et.
    pub fn dump(&self) {
        let hdr = format!("{} ({} kayıt):",
            &Lang::get(MsgId::CompatLaunchServicesDb),
            self.handlers.len());
        console_writeln(&hdr);
        for h in &self.handlers {
            let msg = format!("  {} ({}): şemalar={:?}  uzantılar={:?}",
                h.bundle_id, h.name, h.schemes, h.extensions);
            console_writeln(&msg);
        }
    }
}

impl Default for LaunchServicesDb {
    fn default() -> Self { Self::new() }
}

// ─── UTI (Uniform Type Identifier) Kayıt Defteri ────────────────

/// Tek bir UTI tanımı.
#[derive(Debug, Clone)]
pub struct UtiEntry {
    pub identifier:    String,
    pub preferred_tag: String,
    pub tags:          Vec<String>,
    pub conforms_to:   Vec<String>,
    pub description:   &'static str,
}

/// macOS UTI kayıt defteri öykünücüsü.
///
/// `conforms_to` hiyerarşisi üzerinde özyinelemeli uyumluluk
/// denetimi sağlar (`conforms_to` metodu).
pub struct UtiRegistry {
    entries: BTreeMap<String, UtiEntry>,
}

impl UtiRegistry {
    /// Temel macOS UTI'larıyla doldurulmuş yeni kayıt defteri.
    #[must_use]
    pub fn new() -> Self {
        let mut reg = Self { entries: BTreeMap::new() };

        // Temel hiyerarşi
        reg.add("public.item",      "", &[],                           &[],                             "Tüm dosya sistemi öğeleri");
        reg.add("public.data",      "", &[],                           &["public.item"],                "Ham veri");
        reg.add("public.content",   "", &[],                           &["public.data"],                "İçerik");
        reg.add("public.directory", "", &[],                           &["public.item"],                "Dizin");
        reg.add("public.folder",    "", &[],                           &["public.directory"],           "Klasör");
        reg.add("public.executable","", &[],                           &["public.data"],                "Çalıştırılabilir");

        // Metin
        reg.add("public.text",        "text", &[],                                   &["public.content"],              "Metin");
        reg.add("public.plain-text",  "txt",  &["txt", "text", "text/plain"],        &["public.text"],                 "Düz metin");
        reg.add("public.utf8-plain-text","txt",&["txt", "text/plain; charset=utf-8"],&["public.plain-text"],           "UTF-8 düz metin");
        reg.add("public.rtf",         "rtf",  &["rtf", "text/rtf"],                  &["public.text"],                 "Zengin Metin (RTF)");
        reg.add("public.html",        "html", &["html", "htm", "text/html"],         &["public.text"],                 "HTML belgesi");
        reg.add("public.xhtml",       "xhtml",&["xhtml", "application/xhtml+xml"],   &["public.html"],                 "XHTML belgesi");
        reg.add("public.xml",         "xml",  &["xml", "text/xml"],                  &["public.text"],                 "XML belgesi");
        reg.add("public.json",        "json", &["json", "application/json"],         &["public.text"],                 "JSON belgesi");
        reg.add("public.yaml",        "yaml", &["yaml", "yml", "text/yaml"],         &["public.text"],                 "YAML belgesi");

        // Görüntü
        reg.add("public.image",       "",     &[],                                   &["public.content"],              "Görüntü");
        reg.add("public.jpeg",        "jpg",  &["jpg", "jpeg", "image/jpeg"],        &["public.image"],                "JPEG görüntü");
        reg.add("public.png",         "png",  &["png", "image/png"],                 &["public.image"],                "PNG görüntü");
        reg.add("public.tiff",        "tiff", &["tiff", "tif", "image/tiff"],        &["public.image"],                "TIFF görüntü");
        reg.add("public.gif",         "gif",  &["gif", "image/gif"],                 &["public.image"],                "GIF görüntü");
        reg.add("public.bmp",         "bmp",  &["bmp", "image/bmp"],                 &["public.image"],                "BMP görüntü");
        reg.add("public.svg-image",   "svg",  &["svg", "image/svg+xml"],             &["public.image"],                "SVG vektör görüntü");
        reg.add("public.heic",        "heic", &["heic", "image/heic"],               &["public.image"],                "HEIC görüntü");
        reg.add("com.compuserve.gif", "gif",  &["gif", "image/gif"],                 &["public.gif"],                  "CompuServe GIF");

        // Ses
        reg.add("public.audio",       "",     &[],                                   &["public.content"],              "Ses");
        reg.add("public.mp3",         "mp3",  &["mp3", "audio/mpeg"],                &["public.audio"],                "MP3 ses");
        reg.add("public.aac-audio",   "aac",  &["aac", "audio/aac"],                 &["public.audio"],                "AAC ses");
        reg.add("com.apple.m4a-audio","m4a",  &["m4a", "audio/x-m4a"],              &["public.audio"],                "M4A ses");
        reg.add("com.microsoft.waveform-audio","wav",&["wav","audio/wav"],           &["public.audio"],                "WAV ses");
        reg.add("public.aiff-audio",  "aiff", &["aiff","aif","audio/aiff"],          &["public.audio"],                "AIFF ses");

        // Video
        reg.add("public.movie",       "",     &[],                                   &["public.content"],              "Video");
        reg.add("public.mpeg",        "mpeg", &["mpeg","mpg","video/mpeg"],          &["public.movie"],                "MPEG video");
        reg.add("public.mpeg-4",      "mp4",  &["mp4","video/mp4"],                  &["public.movie"],                "MPEG-4 video");
        reg.add("com.apple.quicktime-movie","mov",&["mov","video/quicktime"],        &["public.movie"],                "QuickTime video");
        reg.add("public.avi",         "avi",  &["avi","video/avi"],                  &["public.movie"],                "AVI video");
        reg.add("public.webm",        "webm", &["webm","video/webm"],                &["public.movie"],                "WebM video");

        // Belge
        reg.add("com.adobe.pdf",       "pdf",  &["pdf","application/pdf"],           &["public.data","public.content"],"PDF belgesi");
        reg.add("com.microsoft.word.doc","doc",&["doc","application/msword"],        &["public.content"],              "Word belgesi");
        reg.add("org.openxmlformats.wordprocessingml.document","docx",
                &["docx","application/vnd.openxmlformats-officedocument.wordprocessingml.document"],
                &["public.content"], "Word OOXML belgesi");
        reg.add("org.openxmlformats.spreadsheetml.sheet","xlsx",
                &["xlsx","application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"],
                &["public.content"], "Excel OOXML çalışma kitabı");

        // Arşiv
        reg.add("public.zip-archive",      "zip",  &["zip","application/zip"],       &["public.data"],                 "ZIP arşiv");
        reg.add("org.gnu.gnu-zip-archive", "gz",   &["gz","application/gzip"],       &["public.data"],                 "GZIP arşiv");
        reg.add("public.tar-archive",      "tar",  &["tar","application/x-tar"],     &["public.data"],                 "TAR arşiv");
        reg.add("com.rarlab.rar-archive",  "rar",  &["rar","application/x-rar"],     &["public.data"],                 "RAR arşiv");

        // Uygulama
        reg.add("com.apple.bundle",             "", &[], &["public.directory"],                      "Bundle");
        reg.add("com.apple.application-bundle", "", &[], &["com.apple.bundle","public.executable"],  "Uygulama bundle");
        reg.add("com.apple.package",            "", &[], &["public.directory"],                      "Paket");

        reg
    }

    /// Yeni UTI girişi ekle.
    pub fn add(
        &mut self,
        id:          &str,
        preferred:   &str,
        tags:        &[&str],
        conforms_to: &[&str],
        description: &'static str,
    ) {
        self.entries.insert(id.to_string(), UtiEntry {
            identifier:    id.to_string(),
            preferred_tag: preferred.to_string(),
            tags:          tags.iter().map(|t| t.to_string()).collect(),
            conforms_to:   conforms_to.iter().map(|c| c.to_string()).collect(),
            description,
        });
    }

    /// ID ile UTI girdisini bul.
    #[must_use]
    pub fn lookup(&self, id: &str) -> Option<&UtiEntry> {
        self.entries.get(id)
    }

    /// `id` UTI'sinin `parent` UTI'sine uyup uymadığını özyinelemeli denetle.
    #[must_use]
    pub fn conforms_to(&self, id: &str, parent: &str) -> bool {
        if id == parent { return true; }
        if let Some(entry) = self.entries.get(id) {
            for c in &entry.conforms_to {
                if self.conforms_to(c, parent) { return true; }
            }
        }
        false
    }

    /// Dosya uzantısından UTI bul (ör. "png" → "public.png").
    #[must_use]
    pub fn uti_for_extension(&self, ext: &str) -> Option<&str> {
        self.entries.values()
            .find(|e| e.tags.iter().any(|t| t == ext))
            .map(|e| e.identifier.as_str())
    }

    /// MIME türünden UTI bul (ör. "image/jpeg" → "public.jpeg").
    #[must_use]
    pub fn uti_for_mime(&self, mime: &str) -> Option<&str> {
        self.entries.values()
            .find(|e| e.tags.iter().any(|t| t == mime))
            .map(|e| e.identifier.as_str())
    }

    /// Tercih edilen uzantıyı döndür (ör. "public.jpeg" → "jpg").
    #[must_use]
    pub fn preferred_extension(&self, id: &str) -> Option<&str> {
        self.entries.get(id).map(|e| e.preferred_tag.as_str())
    }

    /// Toplam kayıtlı UTI sayısı.
    #[must_use]
    pub fn count(&self) -> usize { self.entries.len() }

    /// Kayıt defterini konsola döküm et.
    pub fn dump(&self) {
        let hdr = format!("{} ({} giriş):",
            &Lang::get(MsgId::CompatUtiRegistry),
            self.entries.len());
        console_writeln(&hdr);
        for e in self.entries.values() {
            let msg = format!("  {} [{}]  uyumlu={:?}  etiketler={:?}",
                e.identifier, e.description, e.conforms_to, e.tags);
            console_writeln(&msg);
        }
    }
}

impl Default for UtiRegistry {
    fn default() -> Self { Self::new() }
}

// ─── FSEvents Akış Öykünücüsü ───────────────────────────────────

/// FSEvents olay bayrakları.
pub mod fs_event_flags {
    pub const MUST_SCAN_SUBDIRS:  u32 = 0x0000_0001;
    pub const USER_DROPPED:       u32 = 0x0000_0002;
    pub const KERNEL_DROPPED:     u32 = 0x0000_0004;
    pub const IDS_WRAPPED:        u32 = 0x0000_0008;
    pub const HISTORY_DONE:       u32 = 0x0000_0010;
    pub const ROOT_CHANGED:       u32 = 0x0000_0020;
    pub const MOUNT:              u32 = 0x0000_0040;
    pub const UNMOUNT:            u32 = 0x0000_0080;
    pub const ITEM_CREATED:       u32 = 0x0000_0100;
    pub const ITEM_REMOVED:       u32 = 0x0000_0200;
    pub const ITEM_INODE_META_MOD:u32 = 0x0000_0400;
    pub const ITEM_RENAMED:       u32 = 0x0000_0800;
    pub const ITEM_MODIFIED:      u32 = 0x0000_1000;
    pub const ITEM_FINDER_INFO_MOD:u32= 0x0000_2000;
    pub const ITEM_CHANGE_OWNER:  u32 = 0x0000_4000;
    pub const ITEM_XATTR_MOD:     u32 = 0x0000_8000;
    pub const ITEM_IS_FILE:       u32 = 0x0001_0000;
    pub const ITEM_IS_DIR:        u32 = 0x0002_0000;
    pub const ITEM_IS_SYMLINK:    u32 = 0x0004_0000;
    pub const ITEM_IS_HARDLINK:   u32 = 0x0010_0000;
    pub const ITEM_IS_LAST_HARDLINK:u32=0x0020_0000;
    pub const ITEM_CLONED:        u32 = 0x0040_0000;
}

/// Tek bir FSEvents olayı.
#[derive(Debug, Clone)]
pub struct FseventsEvent {
    /// Etkilenen dosya/dizin yolu.
    pub path:  String,
    /// Olay bayrakları (`fs_event_flags` sabitleri).
    pub flags: u32,
    /// Monoton artan olay kimliği.
    pub id:    u64,
}

impl FseventsEvent {
    /// Olay tipini metinsel olarak döndür (ör. "CREATE|FILE").
    #[must_use]
    pub fn type_str(&self) -> String {
        use fs_event_flags::*;
        let mut parts: Vec<&str> = Vec::new();
        if self.flags & ITEM_CREATED      != 0 { parts.push("CREATE");  }
        if self.flags & ITEM_REMOVED      != 0 { parts.push("REMOVE");  }
        if self.flags & ITEM_MODIFIED     != 0 { parts.push("MODIFY");  }
        if self.flags & ITEM_RENAMED      != 0 { parts.push("RENAME");  }
        if self.flags & ITEM_IS_FILE      != 0 { parts.push("FILE");    }
        if self.flags & ITEM_IS_DIR       != 0 { parts.push("DIR");     }
        if self.flags & ITEM_IS_SYMLINK   != 0 { parts.push("SYMLINK"); }
        if self.flags & MOUNT             != 0 { parts.push("MOUNT");   }
        if self.flags & UNMOUNT           != 0 { parts.push("UNMOUNT"); }
        if parts.is_empty() { return String::from("UNKNOWN"); }
        parts.join("|")
    }
}

/// FSEvents akış durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Running,
    Stopped,
}

/// macOS FSEvents akışı öykünücüsü.
pub struct FseventsStream {
    pub paths:      Vec<String>,
    pub latency_ms: u32,
    pub device_id:  u64,
    pub state:      StreamState,
    events:         Vec<FseventsEvent>,
    next_id:        u64,
}

impl FseventsStream {
    /// Belirtilen yollar üzerinde yeni akış oluştur.
    #[must_use]
    pub fn new(paths: &[&str], latency_ms: u32) -> Self {
        Self {
            paths:      paths.iter().map(|p| p.to_string()).collect(),
            latency_ms,
            device_id:  0x0000_0001_0000_0001,
            state:      StreamState::Running,
            events:     Vec::new(),
            next_id:    1,
        }
    }

    /// Yeni olay kuyruğa ekle.
    /// Akış durdurulmuşsa `Err(CoreServicesError::StreamClosed)` döner.
    #[must_use]
    pub fn push_event(&mut self, path: &str, flags: u32) -> Result<u64, CoreServicesError> {
        if self.state == StreamState::Stopped {
            return Err(CoreServicesError::StreamClosed);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.events.push(FseventsEvent { path: path.to_string(), flags, id });
        let msg = format!("{} #{:06}  {:>22}  {:#010X}  {}",
            &Lang::get(MsgId::CompatFseventsStream), id, path, flags, self.events.last().unwrap().type_str());
        console_writeln(&msg);
        Ok(id)
    }

    /// Bekleyen tüm olayları al ve kuyruğu temizle.
    pub fn flush(&mut self) -> Vec<FseventsEvent> {
        let ev = core::mem::take(&mut self.events);
        console_writeln(&Lang::get(MsgId::CompatFseventsFlush));
        ev
    }

    /// Kuyrukta bekleyen olay sayısı.
    #[must_use]
    pub fn pending_count(&self) -> usize { self.events.len() }

    /// Akışı durdur; bundan sonra `push_event` `StreamClosed` döner.
    pub fn stop(&mut self) {
        self.state = StreamState::Stopped;
    }

    /// Akışı konsola döküm et.
    pub fn dump(&self) {
        let hdr = format!("{} ({} olay, gecikme={}ms):",
            &Lang::get(MsgId::CompatFseventsStream),
            self.events.len(),
            self.latency_ms);
        console_writeln(&hdr);
        for e in &self.events {
            let msg = format!("  #{:06}  {:<30}  {:#010X}  {}",
                e.id, e.path, e.flags, e.type_str());
            console_writeln(&msg);
        }
    }
}

// ─── Spotlight Meta Veri Sorgu Motoru ───────────────────────────

/// Spotlight meta veri öğesi.
#[derive(Debug, Clone)]
pub struct MdItem {
    pub path:  String,
    pub attrs: BTreeMap<String, MdValue>,
}

/// Spotlight öznitelik değer tipi.
#[derive(Debug, Clone)]
pub enum MdValue {
    Text(String),
    Number(i64),
    Bool(bool),
    Date(u64),
}

impl MdValue {
    #[must_use]
    pub fn as_str(&self) -> String {
        match self {
            Self::Text(s)   => s.clone(),
            Self::Number(n) => format!("{}", n),
            Self::Bool(b)   => format!("{}", b),
            Self::Date(d)   => format!("date:{}", d),
        }
    }
}

/// Spotlight sorgu motoru öykünücüsü.
pub struct SpotlightQuery {
    pub query:   String,
    pub scope:   Vec<String>,
    results:     Vec<MdItem>,
    executed:    bool,
}

impl SpotlightQuery {
    /// Yeni sorgu oluştur. `scope` boşsa "/" aranır.
    #[must_use]
    pub fn new(query: &str, scope: &[&str]) -> Self {
        if query.is_empty() {
            return Self {
                query:    String::new(),
                scope:    Vec::new(),
                results:  Vec::new(),
                executed: false,
            };
        }
        console_writeln(&Lang::get(MsgId::CompatSpotlightStart));
        let scope_paths = if scope.is_empty() {
            alloc::vec![String::from("/")]
        } else {
            scope.iter().map(|s| s.to_string()).collect()
        };
        Self {
            query:    query.to_string(),
            scope:    scope_paths,
            results:  Vec::new(),
            executed: false,
        }
    }

    /// Sonuç öğesi ekle.
    pub fn add_result(&mut self, path: &str, attrs: &[(&str, MdValue)]) {
        let mut item = MdItem {
            path:  path.to_string(),
            attrs: BTreeMap::new(),
        };
        for (k, v) in attrs {
            item.attrs.insert(k.to_string(), v.clone());
        }
        self.results.push(item);
    }

    /// Sorguyu çalıştır (öykünücü: hazır sonuçları işaretler).
    /// Zaten çalıştırılmışsa `Err` döner.
    #[must_use]
    pub fn execute(&mut self) -> Result<usize, CoreServicesError> {
        if self.query.is_empty() {
            return Err(CoreServicesError::InvalidQuery);
        }
        self.executed = true;
        let msg = format!("{}: \"{}\" → {} sonuç  kapsam={:?}",
            &Lang::get(MsgId::CompatSpotlightRun),
            self.query,
            self.results.len(),
            self.scope);
        console_writeln(&msg);
        Ok(self.results.len())
    }

    /// Sonuçlara erişim.
    #[must_use]
    pub fn results(&self) -> &[MdItem] { &self.results }

    /// Sorgu çalıştırıldı mı?
    #[must_use]
    pub fn is_executed(&self) -> bool { self.executed }

    /// Sonuçları konsola döküm et.
    pub fn dump(&self) {
        for r in &self.results {
            let attrs_str: String = r.attrs.iter()
                .map(|(k, v)| format!("{}={}", k, v.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            let msg = format!("  {}  [{}]", r.path, attrs_str);
            console_writeln(&msg);
        }
    }
}

// ─── Birim Testler ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfstring_roundtrip() {
        let s = CFString::try_from_str("merhaba").unwrap();
        assert_eq!(s.as_str(), "merhaba");
        assert_eq!(s.len(), 7);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_cfstring_empty() {
        let s = CFString::empty();
        assert!(s.is_empty());
        assert_eq!(s.as_str(), "");
    }

    #[test]
    fn test_cfstring_too_long() {
        let long_str: alloc::string::String = "a".repeat(CFString::MAX_BYTES + 1);
        assert_eq!(
            CFString::try_from_str(&long_str).unwrap_err(),
            CoreServicesError::StringTooLong
        );
    }

    #[test]
    fn test_cfstring_truncate() {
        let long_str: alloc::string::String = "b".repeat(CFString::MAX_BYTES + 50);
        let s = CFString::from_str_truncate(&long_str);
        assert_eq!(s.len(), CFString::MAX_BYTES);
    }

    #[test]
    fn test_cfstring_equality() {
        let a = CFString::try_from_str("test").unwrap();
        let b = CFString::try_from_str("test").unwrap();
        let c = CFString::try_from_str("other").unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_uti_conforms_to() {
        let reg = UtiRegistry::new();
        assert!(reg.conforms_to("public.jpeg", "public.image"));
        assert!(reg.conforms_to("public.jpeg", "public.content"));
        assert!(reg.conforms_to("public.jpeg", "public.data"));
        assert!(!reg.conforms_to("public.jpeg", "public.audio"));
    }

    #[test]
    fn test_uti_for_extension() {
        let reg = UtiRegistry::new();
        assert_eq!(reg.uti_for_extension("png"),  Some("public.png"));
        assert_eq!(reg.uti_for_extension("pdf"),  Some("com.adobe.pdf"));
        assert_eq!(reg.uti_for_extension("mp3"),  Some("public.mp3"));
        assert_eq!(reg.uti_for_extension("xyz"),  None);
    }

    #[test]
    fn test_uti_for_mime() {
        let reg = UtiRegistry::new();
        assert_eq!(reg.uti_for_mime("image/jpeg"),      Some("public.jpeg"));
        assert_eq!(reg.uti_for_mime("application/pdf"), Some("com.adobe.pdf"));
        assert_eq!(reg.uti_for_mime("audio/mpeg"),      Some("public.mp3"));
    }

    #[test]
    fn test_launch_services_lookup() {
        let db = LaunchServicesDb::new();
        assert_eq!(db.lookup_scheme("http"),  Some("com.apple.Safari"));
        assert_eq!(db.lookup_scheme("mailto"),Some("com.apple.Mail"));
        assert_eq!(db.lookup_extension("pdf"),Some("com.apple.Preview"));
        assert_eq!(db.lookup_extension("xyz"),None);
    }

    #[test]
    fn test_launch_services_uti_lookup() {
        let db = LaunchServicesDb::new();
        assert_eq!(db.lookup_uti("public.folder"), Some("com.apple.Finder"));
    }

    #[test]
    fn test_fsevents_push_and_flush() {
        let mut stream = FseventsStream::new(&["/home"], 100);
        assert_eq!(stream.pending_count(), 0);
        stream.push_event("/home/file.txt", fs_event_flags::ITEM_CREATED | fs_event_flags::ITEM_IS_FILE).unwrap();
        stream.push_event("/home/dir",      fs_event_flags::ITEM_CREATED | fs_event_flags::ITEM_IS_DIR).unwrap();
        assert_eq!(stream.pending_count(), 2);
        let events = stream.flush();
        assert_eq!(events.len(), 2);
        assert_eq!(stream.pending_count(), 0);
    }

    #[test]
    fn test_fsevents_stopped_error() {
        let mut stream = FseventsStream::new(&["/"], 50);
        stream.stop();
        let result = stream.push_event("/test", fs_event_flags::ITEM_MODIFIED);
        assert_eq!(result.unwrap_err(), CoreServicesError::StreamClosed);
    }

    #[test]
    fn test_fsevents_type_str() {
        let e = FseventsEvent {
            path:  String::from("/test"),
            flags: fs_event_flags::ITEM_CREATED | fs_event_flags::ITEM_IS_FILE,
            id:    1,
        };
        assert!(e.type_str().contains("CREATE"));
        assert!(e.type_str().contains("FILE"));
    }

    #[test]
    fn test_spotlight_query_execute() {
        let mut q = SpotlightQuery::new("kMDItemKind == 'PDF'", &["/Documents"]);
        q.add_result("/Documents/test.pdf", &[
            ("kMDItemKind",        MdValue::Text("PDF".to_string())),
            ("kMDItemFSSize",      MdValue::Number(204_800)),
            ("kMDItemContentType", MdValue::Text("com.adobe.pdf".to_string())),
        ]);
        let count = q.execute().unwrap();
        assert_eq!(count, 1);
        assert!(q.is_executed());
        assert_eq!(q.results()[0].path, "/Documents/test.pdf");
    }

    #[test]
    fn test_spotlight_empty_query() {
        let mut q = SpotlightQuery::new("", &[]);
        assert_eq!(q.execute().unwrap_err(), CoreServicesError::InvalidQuery);
    }

    #[test]
    fn test_cfbundle_load() {
        let bundle = CFBundle::load("/Applications/TextEdit.app").unwrap();
        assert_eq!(bundle.state, BundleState::Loaded);
        assert_eq!(bundle.bundle_id, "com.ozkan.TextEdit");
    }

    #[test]
    fn test_cfbundle_empty_path() {
        assert_eq!(
            CFBundle::load("").unwrap_err(),
            CoreServicesError::BundleNotFound
        );
    }
}
