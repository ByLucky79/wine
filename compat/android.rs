// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : OzkAndroid Player (APK Sandbox)
// Dosya Yolu         : apps/system/compat/src/android.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32, RISC-V 64, MIPS 32, MIPS 64, PowerPC 32, PowerPC 64, m68k, SPARC, LoongArch64
//
// Açıklama:
//   OzkAndroid katmanı. ZIP/APK ayrıştırıcısı, AndroidManifest.xml
//   binary XML parser (ResChunkHeader/ResStringPool), DEX header
//   ayrıştırıcısı (DexHeader/DexStringId/DexTypeId/DexProtoId/
//   DexFieldId/DexMethodId/DexClassDef), APK Manager (kurulum/
//   kaldırma/listeleme), JNI köprüsü stub'ı ve sandbox başlatma
//   altyapısı sağlar. GUI modülü OzKan WM FFI'ya bağlanır.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/src/dos_emulator.rs  (console_writeln)
//   2-) kernel/graphics/ui/src/lang/lang_flags.rs (MsgId)
//   3-) kernel/graphics/ui/src/lang/lang_manager.rs (Lang::get)
//
//              Dosyaya Müdahaleler
// 2026-04-17      C → Rust çevirisi (no_std)
// 2026-04-18      Lang sistemi, #[must_use], birim testler
// 2026-04-18      Android UI MsgId'leri (549-558), gui_main() tam Lang sistemi
// *******************************************************************

use crate::dos_emulator::console_writeln;
use kernel_ui::{Lang, MsgId};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

// ─── GUI FFI ───────────────────────────────────────────────────

extern "C" {
    fn ozkan_wm_create_window(x: i32, y: i32, w: u32, h: u32, title: *const u8) -> i32;
    fn ozkan_wm_draw_all();
    fn ozkan_gfx_fill_rect(x: i32, y: i32, w: i32, h: i32, color: u32);
    fn ozkan_gfx_draw_text(x: i32, y: i32, text: *const u8, len: u32, color: u32);
    fn ozkan_gfx_swap_buffers();
}

// ─── Tasarım Sabitleri ─────────────────────────────────────────

const WIN_W: i32 = 620;
const WIN_H: i32 = 480;
const CLR_ANDROID_BG: u32 = 0xFF3DDC84;
const CLR_DARK_NAV: u32 = 0xFF073042;
const CLR_CONTENT: u32 = 0xFFFFFFFF;
const CLR_TEXT: u32 = 0xFF202124;
const CLR_ACCENT: u32 = 0xFF107C10;
const CLR_JIT_BAR: u32 = 0xFF00BFFF;

// ─── Veri Yapıları ─────────────────────────────────────────────

/// Android uygulama bilgisi (dinamik, mutable).
#[derive(Debug, Clone)]
pub struct ApkInfo {
    pub name: String,
    pub package: String,
    pub version_code: u32,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub jit_percent: i32,
    pub status: String,
}

impl ApkInfo {
    pub fn new(name: &str, package: &str, version_code: u32, min_sdk: u32, target_sdk: u32, jit_percent: i32, status: &str) -> Self {
        Self {
            name: String::from(name),
            package: String::from(package),
            version_code,
            min_sdk,
            target_sdk,
            jit_percent,
            status: String::from(status),
        }
    }
}

/// Intent sistemi stub'ı.
#[derive(Debug, Clone)]
pub struct Intent {
    pub action: String,
    pub package: String,
    pub extras: BTreeMap<String, String>,
}

impl Intent {
    pub fn new(action: &str, package: &str) -> Self {
        Self {
            action: String::from(action),
            package: String::from(package),
            extras: BTreeMap::new(),
        }
    }

    pub fn put_extra(&mut self, key: &str, value: &str) {
        self.extras.insert(String::from(key), String::from(value));
    }
}

/// JNI köprüsü stub'ı.
pub struct JniBridge;

impl JniBridge {
    #[must_use]
    pub fn call_static_method(_class: &str, _method: &str, _sig: &str) -> i32 {
        console_writeln(&Lang::get(MsgId::CompatApkLaunch));
        0
    }

    #[must_use]
    pub fn call_method(_obj: u64, _method: &str, _sig: &str) -> i32 {
        console_writeln(&Lang::get(MsgId::CompatApkLaunch));
        0
    }
}

/// APK yöneticisi: kurulum, kaldırma, listeleme.
pub struct ApkManager {
    pub apps: alloc::vec::Vec<ApkInfo>,
}

impl Default for ApkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApkManager {
    pub fn new() -> Self {
        let mut mgr = Self { apps: alloc::vec::Vec::new() };
        // Varsayılan uygulamalar
        mgr.apps.push(ApkInfo::new("WhatsApp",  "com.whatsapp",  24000, 21, 33, 85, "Hazir"));
        mgr.apps.push(ApkInfo::new("Instagram", "com.instagram", 30000, 23, 34, 60, "Derleniyor"));
        mgr.apps.push(ApkInfo::new("Ozkan-Map", "os.ozkan.maps", 100,   21, 33, 100, "Hazir"));
        mgr.apps.push(ApkInfo::new("Spotify",   "com.spotify",   8000,  21, 33, 20, "Beklemede"));
        mgr
    }

    #[must_use]
    pub fn install(&mut self, name: &str, package: &str, version_code: u32) -> Option<usize> {
        if self.apps.iter().any(|a| a.package == package) {
            let msg = format!("{}: {} zaten kurulu", &Lang::get(MsgId::CompatApkInstall), package);
            console_writeln(&msg);
            return None;
        }
        self.apps.push(ApkInfo::new(name, package, version_code, 21, 33, 0, "Yukleniyor"));
        let msg = format!("{}: {} v{}", &Lang::get(MsgId::CompatApkInstall), package, version_code);
        console_writeln(&msg);
        Some(self.apps.len() - 1)
    }

    #[must_use]
    pub fn uninstall(&mut self, package: &str) -> bool {
        if let Some(idx) = self.apps.iter().position(|a| a.package == package) {
            self.apps.remove(idx);
            let msg = format!("{}: {} kaldırıldı", &Lang::get(MsgId::CompatApkInstall), package);
            console_writeln(&msg);
            true
        } else {
            let msg = format!("{}: {} bulunamadı", &Lang::get(MsgId::Error), package);
            console_writeln(&msg);
            false
        }
    }

    #[must_use]
    pub fn find(&self, package: &str) -> Option<&ApkInfo> {
        self.apps.iter().find(|a| a.package == package)
    }

    #[must_use]
    pub fn update_status(&mut self, package: &str, status: &str, jit: i32) -> bool {
        if let Some(app) = self.apps.iter_mut().find(|a| a.package == package) {
            app.status = String::from(status);
            app.jit_percent = jit;
            true
        } else {
            false
        }
    }

    pub fn launch_intent(&self, intent: &Intent) {
        let msg = format!("{}: {} → {} ({} extra)",
            &Lang::get(MsgId::CompatApkLaunch),
            intent.action, intent.package, intent.extras.len());
        console_writeln(&msg);
    }

    pub fn parse_manifest_stub(&self, apk_path: &str) {
        let msg = format!("{}: {}", &Lang::get(MsgId::CompatApkLoaded), apk_path);
        console_writeln(&msg);
    }

    /// APK imza ve manifest kontrolü — gerçek içerik üzerinde çalışır.
    ///
    /// * ZIP (APK) local file header imzası: `PK\x03\x04`
    /// * AndroidManifest.xml binary AXML imzası: `0x00080003` (little-endian)
    ///
    /// Dönüş: (zip_ok, axml_ok, apk_size).
    pub fn parse_manifest(&self, apk_bytes: &[u8]) -> (bool, bool, usize) {
        let zip_ok = apk_bytes.len() >= 4
            && apk_bytes[0] == b'P' && apk_bytes[1] == b'K'
            && apk_bytes[2] == 0x03 && apk_bytes[3] == 0x04;
        // AXML sihri APK içinde AndroidManifest.xml entry'sinde — burada
        // yalnızca ilk 64 KiB içinde tarama yapıyoruz (merkezi dizin tam
        // ayrıştırılmadan bile kolayca görünür).
        let scan_end = core::cmp::min(apk_bytes.len(), 65_536);
        let mut axml_ok = false;
        let mut i = 0usize;
        while i + 4 <= scan_end {
            if apk_bytes[i] == 0x03 && apk_bytes[i + 1] == 0x00
                && apk_bytes[i + 2] == 0x08 && apk_bytes[i + 3] == 0x00
            {
                axml_ok = true;
                break;
            }
            i += 1;
        }
        (zip_ok, axml_ok, apk_bytes.len())
    }
}

// ─── Yardımcı Fonksiyonlar ─────────────────────────────────────

fn draw_str(x: i32, y: i32, s: &str, color: u32) {
    if s.is_empty() {
        return;
    }
    unsafe {
        ozkan_gfx_draw_text(x, y, s.as_ptr(), s.len() as u32, color);
    }
}

fn draw_apk_card(x: i32, y: i32, apps: &[ApkInfo], idx: usize) {
    if idx >= apps.len() {
        return;
    }
    let app = &apps[idx];

    // Kart arka planı
    unsafe { ozkan_gfx_fill_rect(x, y, WIN_W - 200, 80, 0xFFF1F3F4); }

    // Uygulama ikonu (yeşil)
    unsafe { ozkan_gfx_fill_rect(x + 15, y + 15, 50, 50, CLR_ANDROID_BG); }
    draw_str(x + 28, y + 30, "A", CLR_DARK_NAV);

    // İsim ve paket
    draw_str(x + 80, y + 15, &app.name, CLR_TEXT);
    draw_str(x + 80, y + 35, &app.package, 0xFF5F6368);

    // JIT Progress Bar
    draw_str(x + 80, y + 55, &Lang::get(MsgId::CompatApkUiJit), 0xFF5F6368);
    unsafe { ozkan_gfx_fill_rect(x + 120, y + 58, 150, 10, 0xFFDADCE0); }
    let bar_w = (150 * app.jit_percent) / 100;
    unsafe { ozkan_gfx_fill_rect(x + 120, y + 58, bar_w, 10, CLR_JIT_BAR); }

    // Başlat butonu
    unsafe { ozkan_gfx_fill_rect(x + WIN_W - 300, y + 25, 80, 30, CLR_ANDROID_BG); }
    draw_str(x + WIN_W - 285, y + 32, &Lang::get(MsgId::CompatApkUiStart), CLR_DARK_NAV);
}

// ─── Ana Fonksiyonlar ──────────────────────────────────────────

/// Android Player GUI'sini çiz.
pub fn gui_main(manager: &ApkManager) {
    let win_title = &Lang::get(MsgId::CompatApkUiWinTitle);
    let mut title_buf = [0u8; 64];
    let tb = win_title.as_bytes();
    let tlen = tb.len().min(63);
    title_buf[..tlen].copy_from_slice(&tb[..tlen]);

    unsafe {
        ozkan_wm_create_window(100, 50, WIN_W as u32, WIN_H as u32, title_buf.as_ptr());
        ozkan_gfx_fill_rect(100, 50, WIN_W, WIN_H, CLR_CONTENT);
        ozkan_gfx_fill_rect(100, 50, WIN_W, 60, CLR_DARK_NAV);
    }
    draw_str(120, 68, &Lang::get(MsgId::CompatApkUiTitle), 0xFFFFFFFF);
    draw_str(120, 85, &Lang::get(MsgId::CompatApkUiSubtitle), CLR_ANDROID_BG);

    unsafe {
        ozkan_gfx_fill_rect(100, 110, 180, WIN_H - 60, 0xFFF8F9FA);
    }
    draw_str(115, 130, &Lang::get(MsgId::CompatApkUiApps), CLR_ACCENT);
    draw_str(115, 160, &Lang::get(MsgId::CompatApkUiLoad), CLR_TEXT);
    draw_str(115, 190, &Lang::get(MsgId::CompatApkUiBinderLog), CLR_TEXT);
    draw_str(115, 220, &Lang::get(MsgId::CompatApkUiJitSettings), CLR_TEXT);

    // APK listesi
    let mut cur_y = 120;
    for i in 0..manager.apps.len() {
        draw_apk_card(290, cur_y, &manager.apps, i);
        cur_y += 90;
    }

    // Alt bilgi barı
    unsafe {
        ozkan_gfx_fill_rect(290, 435, WIN_W - 310, 35, 0xFFE8F0FE);
    }
    draw_str(300, 445, &Lang::get(MsgId::CompatApkUiStatus), 0xFF1967D2);

    unsafe {
        ozkan_wm_draw_all();
        ozkan_gfx_swap_buffers();
    }
}

/// APK çalıştırma komutu.
pub fn cmd_runapk(manager: &ApkManager, args: &[&str]) {
    if args.len() < 2 {
        console_writeln(&Lang::get(MsgId::CompatApkLaunch));
        return;
    }
    let msg = format!("{}: {}...", &Lang::get(MsgId::CompatApkSandbox), args[1]);
    console_writeln(&msg);
    gui_main(manager);
}

/// APK kurulum komutu.
pub fn cmd_installapk(manager: &mut ApkManager, args: &[&str]) {
    if args.len() < 4 {
        console_writeln(&Lang::get(MsgId::CompatApkInstall));
        return;
    }
    let version = args[3].parse::<u32>().unwrap_or(1);
    let _ = manager.install(args[1], args[2], version);
}

/// APK kaldırma komutu.
pub fn cmd_uninstallapk(manager: &mut ApkManager, args: &[&str]) {
    if args.len() < 2 {
        console_writeln(&Lang::get(MsgId::CompatApkInstall));
        return;
    }
    let _ = manager.uninstall(args[1]);
}

// ─── REAL ZIP Parser ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ZipLocalFileHeader {
    pub signature: u32, pub version: u16, pub flags: u16, pub compression: u16,
    pub mod_time: u16, pub mod_date: u16, pub crc32: u32,
    pub compressed_size: u32, pub uncompressed_size: u32,
    pub name_len: u16, pub extra_len: u16,
    pub name: String,
}

impl ZipLocalFileHeader {
    pub const SIGNATURE: u32 = 0x04034B50;
    pub fn parse(data: &[u8]) -> Option<(Self, usize)> {
        if data.len() < 30 { return None; }
        let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if sig != Self::SIGNATURE { return None; }
        let name_len = u16::from_le_bytes([data[26], data[27]]) as usize;
        let extra_len = u16::from_le_bytes([data[28], data[29]]) as usize;
        if data.len() < 30 + name_len + extra_len { return None; }
        let name = core::str::from_utf8(&data[30..30 + name_len]).unwrap_or("").to_string();
        Some((Self {
            signature: sig, version: u16::from_le_bytes([data[4], data[5]]),
            flags: u16::from_le_bytes([data[6], data[7]]),
            compression: u16::from_le_bytes([data[8], data[9]]),
            mod_time: u16::from_le_bytes([data[10], data[11]]),
            mod_date: u16::from_le_bytes([data[12], data[13]]),
            crc32: u32::from_le_bytes([data[14], data[15], data[16], data[17]]),
            compressed_size: u32::from_le_bytes([data[18], data[19], data[20], data[21]]),
            uncompressed_size: u32::from_le_bytes([data[22], data[23], data[24], data[25]]),
            name_len: name_len as u16, extra_len: extra_len as u16, name,
        }, 30 + name_len + extra_len))
    }
}

#[derive(Debug, Clone)]
pub struct ZipCentralDirHeader {
    pub signature: u32, pub version_made_by: u16, pub version_needed: u16,
    pub flags: u16, pub compression: u16, pub mod_time: u16, pub mod_date: u16,
    pub crc32: u32, pub comp_size: u32, pub uncomp_size: u32,
    pub name_len: u16, pub extra_len: u16, pub comment_len: u16,
    pub disk_start: u16, pub int_attr: u16, pub ext_attr: u32, pub local_header_offset: u32,
    pub name: String,
}

impl ZipCentralDirHeader {
    pub const SIGNATURE: u32 = 0x02014B50;
    pub fn parse(data: &[u8]) -> Option<(Self, usize)> {
        if data.len() < 46 { return None; }
        let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if sig != Self::SIGNATURE { return None; }
        let name_len = u16::from_le_bytes([data[28], data[29]]) as usize;
        let extra_len = u16::from_le_bytes([data[30], data[31]]) as usize;
        let comment_len = u16::from_le_bytes([data[32], data[33]]) as usize;
        if data.len() < 46 + name_len + extra_len + comment_len { return None; }
        let name = core::str::from_utf8(&data[46..46 + name_len]).unwrap_or("").to_string();
        Some((Self {
            signature: sig, version_made_by: u16::from_le_bytes([data[4], data[5]]),
            version_needed: u16::from_le_bytes([data[6], data[7]]),
            flags: u16::from_le_bytes([data[8], data[9]]),
            compression: u16::from_le_bytes([data[10], data[11]]),
            mod_time: u16::from_le_bytes([data[12], data[13]]),
            mod_date: u16::from_le_bytes([data[14], data[15]]),
            crc32: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            comp_size: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            uncomp_size: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            name_len: name_len as u16, extra_len: extra_len as u16, comment_len: comment_len as u16,
            disk_start: u16::from_le_bytes([data[34], data[35]]),
            int_attr: u16::from_le_bytes([data[36], data[37]]),
            ext_attr: u32::from_le_bytes([data[38], data[39], data[40], data[41]]),
            local_header_offset: u32::from_le_bytes([data[42], data[43], data[44], data[45]]),
            name,
        }, 46 + name_len + extra_len + comment_len))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ZipEndOfCentralDir {
    pub signature: u32, pub disk_num: u16, pub disk_start: u16,
    pub disk_entries: u16, pub total_entries: u16,
    pub size: u32, pub offset: u32, pub comment_len: u16,
}

impl ZipEndOfCentralDir {
    pub const SIGNATURE: u32 = 0x06054B50;
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 22 { return None; }
        let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if sig != Self::SIGNATURE { return None; }
        Some(Self {
            signature: sig, disk_num: u16::from_le_bytes([data[4], data[5]]),
            disk_start: u16::from_le_bytes([data[6], data[7]]),
            disk_entries: u16::from_le_bytes([data[8], data[9]]),
            total_entries: u16::from_le_bytes([data[10], data[11]]),
            size: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            offset: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            comment_len: u16::from_le_bytes([data[20], data[21]]),
        })
    }
}

pub struct ZipParser;

impl ZipParser {
    pub fn find_eocd(data: &[u8]) -> Option<(ZipEndOfCentralDir, usize)> {
        if data.len() < 22 { return None; }
        for i in (0..=data.len() - 22).rev() {
            if let Some(eocd) = ZipEndOfCentralDir::parse(&data[i..]) {
                return Some((eocd, i));
            }
        }
        None
    }
    pub fn parse_central_directory(data: &[u8]) -> Vec<ZipCentralDirHeader> {
        let mut entries = Vec::new();
        let Some((eocd, _)) = Self::find_eocd(data) else { return entries; };
        let mut off = eocd.offset as usize;
        let end = off + eocd.size as usize;
        while off < end && off < data.len() {
            if let Some((entry, size)) = ZipCentralDirHeader::parse(&data[off..]) {
                off += size; entries.push(entry);
            } else { break; }
        }
        entries
    }
    pub fn dump_entries(entries: &[ZipCentralDirHeader]) {
        console_writeln(&Lang::get(MsgId::CompatApkLoaded));
        for e in entries {
            let msg = format!("  {} comp={}/{} method={} crc=0x{:08X}",
                e.name, e.comp_size, e.uncomp_size, e.compression, e.crc32);
            console_writeln(&msg);
        }
    }
}

// ─── REAL AndroidManifest Binary XML Parser ────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ResChunkHeader {
    pub type_: u16, pub header_size: u16, pub size: u32,
}

impl ResChunkHeader {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        Some(Self { type_: u16::from_le_bytes([data[0], data[1]]), header_size: u16::from_le_bytes([data[2], data[3]]), size: u32::from_le_bytes([data[4], data[5], data[6], data[7]]) })
    }
}

#[derive(Debug, Clone)]
pub struct ResStringPool {
    pub string_count: u32, pub style_count: u32, pub flags: u32,
    pub strings_start: u32, pub styles_start: u32,
    pub string_offsets: Vec<u32>, pub strings: Vec<String>,
}

impl ResStringPool {
    pub const UTF8_FLAG: u32 = 0x100;
    pub fn parse(data: &[u8]) -> Option<Self> {
        let hdr = ResChunkHeader::parse(data)?;
        if hdr.type_ != 0x0001 || data.len() < hdr.size as usize { return None; }
        let string_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let style_count = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let flags = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let strings_start = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let styles_start = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let mut offsets = Vec::new();
        let mut off = 28usize;
        for _ in 0..string_count {
            if data.len() < off + 4 { break; }
            offsets.push(u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]));
            off += 4;
        }
        let base = strings_start as usize;
        let mut strings = Vec::new();
        for o in &offsets {
            let pos = base + *o as usize;
            if data.len() < pos + 2 { strings.push(String::new()); continue; }
            if (flags & Self::UTF8_FLAG) != 0 {
                let len = data[pos] as usize;
                if data.len() < pos + 1 + len { strings.push(String::new()); continue; }
                strings.push(core::str::from_utf8(&data[pos + 1..pos + 1 + len]).unwrap_or("").to_string());
            } else {
                let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                let mut s = String::new();
                for i in 0..len {
                    let cpos = pos + 2 + i * 2;
                    if data.len() < cpos + 2 { break; }
                    let c = u16::from_le_bytes([data[cpos], data[cpos + 1]]);
                    if c < 0x80 { s.push(c as u8 as char); } else { s.push('?'); }
                }
                strings.push(s);
            }
        }
        Some(Self { string_count, style_count, flags, strings_start, styles_start, string_offsets: offsets, strings })
    }
}

#[derive(Debug, Clone)]
pub struct XmlStartElement {
    pub line: u32, pub comment: u32, pub ns: u32, pub name: u32,
    pub attr_start: u16, pub attr_size: u16, pub attr_count: u16,
    pub id_index: u16, pub class_index: u16, pub style_index: u16,
}

impl XmlStartElement {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 20 { return None; }
        Some(Self {
            line: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            comment: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            ns: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            name: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            attr_start: u16::from_le_bytes([data[16], data[17]]),
            attr_size: u16::from_le_bytes([data[18], data[19]]),
            attr_count: u16::from_le_bytes([data[20], data[21]]),
            id_index: u16::from_le_bytes([data[22], data[23]]),
            class_index: u16::from_le_bytes([data[24], data[25]]),
            style_index: u16::from_le_bytes([data[26], data[27]]),
        })
    }
}

pub struct AndroidManifestParser;

impl AndroidManifestParser {
    pub fn parse(data: &[u8]) -> Option<(ResStringPool, Vec<XmlStartElement>)> {
        let hdr = ResChunkHeader::parse(data)?;
        if hdr.type_ != 0x0003 { return None; } // XML type
        let mut off = hdr.header_size as usize;
        let mut string_pool: Option<ResStringPool> = None;
        let mut elements = Vec::new();
        while off < data.len() {
            let chunk = ResChunkHeader::parse(&data[off..])?;
            match chunk.type_ {
                0x0001 => { // String pool
                    string_pool = ResStringPool::parse(&data[off..]);
                }
                0x0100 => { // Start namespace
                }
                0x0102 => { // Start element
                    if let Some(el) = XmlStartElement::parse(&data[off + chunk.header_size as usize..]) {
                        elements.push(el);
                    }
                }
                _ => {}
            }
            off += chunk.size as usize;
        }
        Some((string_pool?, elements))
    }
}

// ─── REAL DEX Header Parser ────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct DexHeader {
    pub magic: [u8; 8], pub checksum: u32, pub signature: [u8; 20],
    pub file_size: u32, pub header_size: u32, pub endian_tag: u32,
    pub link_size: u32, pub link_off: u32,
    pub map_off: u32,
    pub string_ids_size: u32, pub string_ids_off: u32,
    pub type_ids_size: u32, pub type_ids_off: u32,
    pub proto_ids_size: u32, pub proto_ids_off: u32,
    pub field_ids_size: u32, pub field_ids_off: u32,
    pub method_ids_size: u32, pub method_ids_off: u32,
    pub class_defs_size: u32, pub class_defs_off: u32,
    pub data_size: u32, pub data_off: u32,
}

impl DexHeader {
    pub const DEX_MAGIC: &[u8] = b"dex\n035\0";
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 112 { return None; }
        let mut magic = [0u8; 8]; magic.copy_from_slice(&data[0..8]);
        if &magic[0..4] != b"dex\n" { return None; }
        Some(Self {
            magic, checksum: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            signature: { let mut s = [0u8; 20]; s.copy_from_slice(&data[12..32]); s },
            file_size: u32::from_le_bytes([data[32], data[33], data[34], data[35]]),
            header_size: u32::from_le_bytes([data[36], data[37], data[38], data[39]]),
            endian_tag: u32::from_le_bytes([data[40], data[41], data[42], data[43]]),
            link_size: u32::from_le_bytes([data[44], data[45], data[46], data[47]]),
            link_off: u32::from_le_bytes([data[48], data[49], data[50], data[51]]),
            map_off: u32::from_le_bytes([data[52], data[53], data[54], data[55]]),
            string_ids_size: u32::from_le_bytes([data[56], data[57], data[58], data[59]]),
            string_ids_off: u32::from_le_bytes([data[60], data[61], data[62], data[63]]),
            type_ids_size: u32::from_le_bytes([data[64], data[65], data[66], data[67]]),
            type_ids_off: u32::from_le_bytes([data[68], data[69], data[70], data[71]]),
            proto_ids_size: u32::from_le_bytes([data[72], data[73], data[74], data[75]]),
            proto_ids_off: u32::from_le_bytes([data[76], data[77], data[78], data[79]]),
            field_ids_size: u32::from_le_bytes([data[80], data[81], data[82], data[83]]),
            field_ids_off: u32::from_le_bytes([data[84], data[85], data[86], data[87]]),
            method_ids_size: u32::from_le_bytes([data[88], data[89], data[90], data[91]]),
            method_ids_off: u32::from_le_bytes([data[92], data[93], data[94], data[95]]),
            class_defs_size: u32::from_le_bytes([data[96], data[97], data[98], data[99]]),
            class_defs_off: u32::from_le_bytes([data[100], data[101], data[102], data[103]]),
            data_size: u32::from_le_bytes([data[104], data[105], data[106], data[107]]),
            data_off: u32::from_le_bytes([data[108], data[109], data[110], data[111]]),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DexStringId { pub string_data_off: u32 }
impl DexStringId {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 { return None; }
        Some(Self { string_data_off: u32::from_le_bytes([data[0], data[1], data[2], data[3]]) })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DexTypeId { pub descriptor_idx: u32 }
impl DexTypeId {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 { return None; }
        Some(Self { descriptor_idx: u32::from_le_bytes([data[0], data[1], data[2], data[3]]) })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DexProtoId {
    pub shorty_idx: u32, pub return_type_idx: u32, pub parameters_off: u32,
}
impl DexProtoId {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 12 { return None; }
        Some(Self {
            shorty_idx: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            return_type_idx: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            parameters_off: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DexFieldId {
    pub class_idx: u16, pub type_idx: u16, pub name_idx: u32,
}
impl DexFieldId {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        Some(Self {
            class_idx: u16::from_le_bytes([data[0], data[1]]),
            type_idx: u16::from_le_bytes([data[2], data[3]]),
            name_idx: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DexMethodId {
    pub class_idx: u16, pub proto_idx: u16, pub name_idx: u32,
}
impl DexMethodId {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        Some(Self {
            class_idx: u16::from_le_bytes([data[0], data[1]]),
            proto_idx: u16::from_le_bytes([data[2], data[3]]),
            name_idx: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DexClassDef {
    pub class_idx: u32, pub access_flags: u32, pub superclass_idx: u32,
    pub interfaces_off: u32, pub source_file_idx: u32, pub annotations_off: u32,
    pub class_data_off: u32, pub static_values_off: u32,
}
impl DexClassDef {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 32 { return None; }
        Some(Self {
            class_idx: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            access_flags: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            superclass_idx: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            interfaces_off: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            source_file_idx: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            annotations_off: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            class_data_off: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            static_values_off: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
        })
    }
}

pub struct DexParser;

impl DexParser {
    pub fn parse(data: &[u8]) -> Option<(DexHeader, Vec<DexStringId>, Vec<DexTypeId>, Vec<DexProtoId>, Vec<DexFieldId>, Vec<DexMethodId>, Vec<DexClassDef>)> {
        let hdr = DexHeader::parse(data)?;
        let mut strings = Vec::new();
        let off = hdr.string_ids_off as usize;
        for i in 0..hdr.string_ids_size as usize {
            let pos = off + i * 4;
            if data.len() < pos + 4 { break; }
            if let Some(s) = DexStringId::parse(&data[pos..]) { strings.push(s); }
        }
        let mut types = Vec::new();
        let off = hdr.type_ids_off as usize;
        for i in 0..hdr.type_ids_size as usize {
            let pos = off + i * 4;
            if data.len() < pos + 4 { break; }
            if let Some(t) = DexTypeId::parse(&data[pos..]) { types.push(t); }
        }
        let mut protos = Vec::new();
        let off = hdr.proto_ids_off as usize;
        for i in 0..hdr.proto_ids_size as usize {
            let pos = off + i * 12;
            if data.len() < pos + 12 { break; }
            if let Some(p) = DexProtoId::parse(&data[pos..]) { protos.push(p); }
        }
        let mut fields = Vec::new();
        let off = hdr.field_ids_off as usize;
        for i in 0..hdr.field_ids_size as usize {
            let pos = off + i * 8;
            if data.len() < pos + 8 { break; }
            if let Some(f) = DexFieldId::parse(&data[pos..]) { fields.push(f); }
        }
        let mut methods = Vec::new();
        let off = hdr.method_ids_off as usize;
        for i in 0..hdr.method_ids_size as usize {
            let pos = off + i * 8;
            if data.len() < pos + 8 { break; }
            if let Some(m) = DexMethodId::parse(&data[pos..]) { methods.push(m); }
        }
        let mut classes = Vec::new();
        let off = hdr.class_defs_off as usize;
        for i in 0..hdr.class_defs_size as usize {
            let pos = off + i * 32;
            if data.len() < pos + 32 { break; }
            if let Some(c) = DexClassDef::parse(&data[pos..]) { classes.push(c); }
        }
        Some((hdr, strings, types, protos, fields, methods, classes))
    }
    pub fn dump(hdr: &DexHeader) {
        let msg = format!("{}: size={} endian=0x{:08X} str={} type={} proto={} field={} method={} class={}",
            &Lang::get(MsgId::CompatDalvikOp),
            hdr.file_size, hdr.endian_tag, hdr.string_ids_size, hdr.type_ids_size,
            hdr.proto_ids_size, hdr.field_ids_size, hdr.method_ids_size, hdr.class_defs_size);
        console_writeln(&msg);
    }
}

// ─── Birim Testler ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::format;

    // ─── ApkInfo ──────────────────────────────────────────────────

    #[test]
    fn test_apk_info_new() {
        let app = ApkInfo::new("TestApp", "com.test.app", 1, 21, 33, 50, "Hazir");
        assert_eq!(app.name, "TestApp");
        assert_eq!(app.package, "com.test.app");
        assert_eq!(app.jit_percent, 50);
    }

    // ─── Intent ───────────────────────────────────────────────────

    #[test]
    fn test_intent_extras() {
        let mut intent = Intent::new("android.intent.action.VIEW", "com.example");
        intent.put_extra("url", "https://ozkan.os");
        assert_eq!(intent.extras.get("url").map(|s| s.as_str()), Some("https://ozkan.os"));
    }

    // ─── ApkManager ───────────────────────────────────────────────

    #[test]
    fn test_apk_manager_defaults() {
        let mgr = ApkManager::new();
        assert!(mgr.apps.len() >= 3);
        assert!(mgr.find("com.whatsapp").is_some());
    }

    #[test]
    fn test_apk_install_and_find() {
        let mut mgr = ApkManager::new();
        let idx = mgr.install("MyApp", "com.myapp", 100);
        assert!(idx.is_some());
        let app = mgr.find("com.myapp");
        assert!(app.is_some());
        assert_eq!(app.unwrap().version_code, 100);
    }

    #[test]
    fn test_apk_install_duplicate() {
        let mut mgr = ApkManager::new();
        assert!(mgr.install("W", "com.whatsapp", 999).is_none());
    }

    #[test]
    fn test_apk_uninstall() {
        let mut mgr = ApkManager::new();
        assert!(mgr.uninstall("com.whatsapp"));
        assert!(mgr.find("com.whatsapp").is_none());
    }

    #[test]
    fn test_apk_uninstall_not_found() {
        let mut mgr = ApkManager::new();
        assert!(!mgr.uninstall("com.doesnotexist"));
    }

    #[test]
    fn test_apk_update_status() {
        let mut mgr = ApkManager::new();
        assert!(mgr.update_status("com.whatsapp", "Hazir", 100));
        assert_eq!(mgr.find("com.whatsapp").unwrap().jit_percent, 100);
    }

    #[test]
    fn test_apk_update_status_not_found() {
        let mut mgr = ApkManager::new();
        assert!(!mgr.update_status("com.nobody", "X", 0));
    }

    // ─── ZIP Parser ───────────────────────────────────────────────

    #[test]
    fn test_zip_local_header_invalid_sig() {
        let data = vec![0u8; 30];
        assert!(ZipLocalFileHeader::parse(&data).is_none());
    }

    #[test]
    fn test_zip_local_header_too_short() {
        let data = vec![0x50, 0x4B, 0x03, 0x04];
        assert!(ZipLocalFileHeader::parse(&data).is_none());
    }

    #[test]
    fn test_zip_central_dir_invalid_sig() {
        let data = vec![0u8; 46];
        assert!(ZipCentralDirHeader::parse(&data).is_none());
    }

    #[test]
    fn test_zip_eocd_valid() {
        let mut data = vec![0u8; 22];
        data[0] = 0x50; data[1] = 0x4B; data[2] = 0x05; data[3] = 0x06;
        data[10] = 2; data[11] = 0; // total_entries at bytes [10,11]
        let eocd = ZipEndOfCentralDir::parse(&data);
        assert!(eocd.is_some());
        assert_eq!(eocd.unwrap().total_entries, 2);
    }

    #[test]
    fn test_zip_eocd_invalid() {
        let data = vec![0u8; 22];
        assert!(ZipEndOfCentralDir::parse(&data).is_none());
    }

    // ─── DEX Parser ───────────────────────────────────────────────

    #[test]
    fn test_dex_header_too_short() {
        let data = vec![0u8; 64];
        assert!(DexHeader::parse(&data).is_none());
    }

    #[test]
    fn test_dex_header_bad_magic() {
        let data = vec![0u8; 112];
        assert!(DexHeader::parse(&data).is_none());
    }

    #[test]
    fn test_dex_header_valid_magic() {
        let mut data = vec![0u8; 112];
        data[0..4].copy_from_slice(b"dex\n");
        data[4..8].copy_from_slice(b"035\0");
        let hdr = DexHeader::parse(&data);
        assert!(hdr.is_some());
        assert_eq!(&hdr.unwrap().magic[0..4], b"dex\n");
    }

    // ─── ResChunkHeader ───────────────────────────────────────────

    #[test]
    fn test_res_chunk_header_parse() {
        let mut data = vec![0u8; 8];
        data[0] = 0x01; data[1] = 0x00; // type = 1
        data[4] = 0x08; // size
        let hdr = ResChunkHeader::parse(&data);
        assert!(hdr.is_some());
        assert_eq!(hdr.unwrap().type_, 1);
    }

    #[test]
    fn test_res_chunk_header_too_short() {
        let data = vec![0u8; 4];
        assert!(ResChunkHeader::parse(&data).is_none());
    }

    // ─── JniBridge ────────────────────────────────────────────────

    #[test]
    fn test_jni_bridge_returns_zero() {
        assert_eq!(JniBridge::call_static_method("Foo", "bar", "()V"), 0);
        assert_eq!(JniBridge::call_method(0xDEAD, "baz", "()I"), 0);
    }
}

