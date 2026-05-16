// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Wine Server — Windows Registry Emülasyonu
// Dosya Yolu         : apps/system/compat/wine_server_registry.rs
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
//   Windows Registry emülasyonu: WineRegValue, WineRegKey,
//   WineRegistryHive — okuma/yazma/arama işlemleri.
//
// Bağımlı Dosyalar:
//   apps/system/compat/wine_server.rs
//
//              Dosyaya Müdahaleler
// 2026-05-14      Dosya oluşturuldu (wine_server.rs bölündü)
// *******************************************************************

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;
use crate::dos_emulator::console_writeln;
#[derive(Debug, Clone)]
pub enum WineRegValue {
    /// REG_DWORD
    Dword(u32),
    /// REG_QWORD
    Qword(u64),
    /// REG_SZ / REG_EXPAND_SZ
    Sz(String),
    /// REG_BINARY
    Binary(Vec<u8>),
    /// REG_MULTI_SZ
    MultiSz(Vec<String>),
    /// REG_NONE
    None,
}

impl WineRegValue {
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Dword(_)   => "REG_DWORD",
            Self::Qword(_)   => "REG_QWORD",
            Self::Sz(_)      => "REG_SZ",
            Self::Binary(_)  => "REG_BINARY",
            Self::MultiSz(_) => "REG_MULTI_SZ",
            Self::None       => "REG_NONE",
        }
    }
}

/// Kayıt defteri anahtarı.
#[derive(Debug, Clone)]
pub struct WineRegKey {
    pub name:    String,
    pub values:  BTreeMap<String, WineRegValue>,
    pub subkeys: BTreeMap<String, WineRegKey>,
}

impl WineRegKey {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), values: BTreeMap::new(), subkeys: BTreeMap::new() }
    }

    pub fn set_value(&mut self, name: &str, value: WineRegValue) {
        self.values.insert(name.to_string(), value);
    }

    #[must_use]
    pub fn get_value(&self, name: &str) -> Option<&WineRegValue> {
        self.values.get(name)
    }

    /// Alt anahtar al ya da oluştur.
    pub fn create_subkey(&mut self, name: &str) -> &mut WineRegKey {
        self.subkeys.entry(name.to_string())
            .or_insert_with(|| WineRegKey::new(name))
    }

    #[must_use]
    pub fn get_subkey(&self, name: &str) -> Option<&WineRegKey> {
        self.subkeys.get(name)
    }

    #[must_use]
    pub fn get_subkey_mut(&mut self, name: &str) -> Option<&mut WineRegKey> {
        self.subkeys.get_mut(name)
    }
}

/// Windows kayıt defteri hive öykünücüsü (HKLM / HKCU / HKCR).
pub struct WineRegistryHive {
    pub root: WineRegKey,
}

impl WineRegistryHive {
    /// Temel Windows anahtarlarıyla başlatılmış kayıt defteri.
    #[must_use]
    pub fn new() -> Self {
        let mut hive = Self { root: WineRegKey::new("") };
        Self::populate(&mut hive.root);
        hive
    }

    fn populate(root: &mut WineRegKey) {
        // HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion
        {
            let hklm = root.create_subkey("HKEY_LOCAL_MACHINE");
            let sw   = hklm.create_subkey("SOFTWARE");
            let ms   = sw.create_subkey("Microsoft");
            let wnt  = ms.create_subkey("Windows NT");
            let cv   = wnt.create_subkey("CurrentVersion");
            cv.set_value("ProductName",     WineRegValue::Sz("OzkanOS Wine Layer".to_string()));
            cv.set_value("CurrentVersion",  WineRegValue::Sz("10.0".to_string()));
            cv.set_value("CurrentBuild",    WineRegValue::Sz("19045".to_string()));
            cv.set_value("CurrentBuildNumber", WineRegValue::Sz("19045".to_string()));
            cv.set_value("SystemRoot",      WineRegValue::Sz("C:\\Windows".to_string()));
            cv.set_value("ProgramFilesDir", WineRegValue::Sz("C:\\Program Files".to_string()));
            cv.set_value("RegisteredOwner", WineRegValue::Sz("Ozkan Yildirim".to_string()));

            // HKLM\SYSTEM\CurrentControlSet\Services
            let sys = hklm.create_subkey("SYSTEM");
            let ccs = sys.create_subkey("CurrentControlSet");
            let _svc = ccs.create_subkey("Services");
        }

        // HKEY_CURRENT_USER
        {
            let hkcu = root.create_subkey("HKEY_CURRENT_USER");
            let env  = hkcu.create_subkey("Environment");
            env.set_value("TEMP", WineRegValue::Sz("C:\\Users\\User\\AppData\\Local\\Temp".to_string()));
            env.set_value("TMP",  WineRegValue::Sz("C:\\Users\\User\\AppData\\Local\\Temp".to_string()));
            env.set_value("PATH", WineRegValue::Sz(
                "C:\\Windows;C:\\Windows\\System32;C:\\Windows\\SysWOW64".to_string()
            ));
            let _desktop = hkcu.create_subkey("Desktop");
            let _wallpaper = hkcu.create_subkey("Control Panel\\Desktop");
        }

        // HKEY_CLASSES_ROOT
        {
            let hkcr = root.create_subkey("HKEY_CLASSES_ROOT");
            // Yaygın uzantılar
            let pairs = [
                (".txt",  "txtfile"),
                (".exe",  "exefile"),
                (".dll",  "dllfile"),
                (".bat",  "batfile"),
                (".lnk",  "lnkfile"),
                (".htm",  "htmlfile"),
                (".html", "htmlfile"),
                (".pdf",  "AcroExch.Document"),
                (".mp3",  "mp3file"),
                (".png",  "pngfile"),
                (".jpg",  "jpegfile"),
                (".jpeg", "jpegfile"),
            ];
            for (ext, prog_id) in pairs {
                let key = hkcr.create_subkey(ext);
                key.set_value("", WineRegValue::Sz(prog_id.to_string()));
            }
        }
    }

    /// Yol ayrıştırarak (\ ile) anahtara mutably erişim sağla.
    #[must_use]
    pub fn open_key_mut(&mut self, path: &str) -> Option<&mut WineRegKey> {
        if path.is_empty() { return Some(&mut self.root); }
        let mut current = &mut self.root;
        for part in path.split('\\') {
            if part.is_empty() { continue; }
            current = current.create_subkey(part);
        }
        Some(current)
    }

    /// Yol ayrıştırarak anahtara salt okunur erişim.
    #[must_use]
    pub fn open_key(&self, path: &str) -> Option<&WineRegKey> {
        let mut current = &self.root;
        for part in path.split('\\') {
            if part.is_empty() { continue; }
            current = current.get_subkey(part)?;
        }
        Some(current)
    }

    /// Değer oku.
    #[must_use]
    pub fn read_value(&self, key_path: &str, value_name: &str) -> Option<&WineRegValue> {
        self.open_key(key_path)?.get_value(value_name)
    }

    /// Değer yaz.
    pub fn write_value(&mut self, key_path: &str, value_name: &str, value: WineRegValue) {
        if let Some(key) = self.open_key_mut(key_path) {
            key.set_value(value_name, value);
        }
    }

    /// Tüm kayıt defterini konsola döküm et.
    pub fn dump(&self) {
        Self::dump_key(&self.root, "");
    }

    fn dump_key(key: &WineRegKey, prefix: &str) {
        let key_path = if prefix.is_empty() {
            String::new()
        } else {
            format!("{}\\{}", prefix, key.name)
        };
        for (k, v) in &key.values {
            let msg = format!("{}\\{}  [{}]", key_path, k, v.type_name());
            console_writeln(&msg);
        }
        for sk in key.subkeys.values() {
            Self::dump_key(sk, &key_path);
        }
    }
}

impl Default for WineRegistryHive {
    fn default() -> Self { Self::new() }
}

// ─── Birim Testler ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        for v in 0u32..=82 {
            let op = WineRequestOpcode::from_u32(v);
            assert_ne!(op, WineRequestOpcode::Unknown, "opcode {} Unknown olmamalı", v);
        }
    }

    #[test]
    fn test_opcode_name_nonempty() {
        for v in 0u32..=82 {
            let op = WineRequestOpcode::from_u32(v);
            assert!(!op.name().is_empty());
        }
    }

    #[test]
    fn test_request_header_parse_ok() {
        let data: [u8; 12] = [
            0x0F, 0x00, 0x00, 0x00, // req = 15 (LoadDll)
            0x03, 0x00, 0x00, 0x00, // request_pipe = 3
            0x05, 0x00, 0x00, 0x00, // reply_pipe   = 5
        ];
        let hdr = WineServerRequestHeader::parse(&data).unwrap();
        assert_eq!(hdr.req, 15);
        assert_eq!(hdr.opcode(), WineRequestOpcode::LoadDll);
        assert_eq!(hdr.request_pipe, 3);
        assert_eq!(hdr.reply_pipe, 5);
    }

    #[test]
    fn test_request_header_too_short() {
        let data: [u8; 8] = [0u8; 8];
        assert_eq!(
            WineServerRequestHeader::parse(&data).unwrap_err(),
            WineError::RequestTooShort
        );
    }

    #[test]
    fn test_object_manager_create_close() {
        let mut mgr = WineObjectManager::new();
        let h1 = mgr.create(WineObjectType::Mutex, "Global\\TestMutex", 0x1F0001);
        let h2 = mgr.create(WineObjectType::Event, "Global\\TestEvent", 0x1F0003);
        assert_eq!(mgr.count(), 2);
        mgr.close(h1).unwrap();
        assert_eq!(mgr.count(), 1);
        assert!(mgr.lookup(h2).is_some());
    }

    #[test]
    fn test_object_manager_invalid_handle() {
        let mut mgr = WineObjectManager::new();
        assert_eq!(
            mgr.close(0xDEAD_BEEF).unwrap_err(),
            WineError::InvalidHandle
        );
    }

    #[test]
    fn test_nt_path_converter_drive() {
        let result = WineNtPathConverter::nt_to_unix("C:\\Windows\\System32", "/home/user/.wine").unwrap();
        assert!(result.contains("drive_c"));
        assert!(result.contains("Windows/System32"));
    }

    #[test]
    fn test_nt_path_converter_device() {
        let result = WineNtPathConverter::nt_to_unix("\\\\.\\pipe\\wineserver", "/home/user/.wine").unwrap();
        assert!(result.contains("dosdevices"));
        assert!(result.contains("pipe"));
    }

    #[test]
    fn test_nt_path_converter_empty() {
        assert_eq!(
            WineNtPathConverter::nt_to_unix("", "/home").unwrap_err(),
            WineError::InvalidNtPath
        );
    }

    #[test]
    fn test_dll_manager_builtin_load() {
        let mut mgr = WineDllManager::new();
        let base = mgr.load("kernel32.dll", 0, 0).unwrap();
        assert_ne!(base, 0);
        // İkinci yükleme ref_count artırmalı
        mgr.load("kernel32", 0, 0).unwrap();
        assert_eq!(mgr.dlls["kernel32"].ref_count, 2);
    }

    #[test]
    fn test_dll_manager_unload() {
        let mut mgr = WineDllManager::new();
        mgr.load("kernel32", 0, 0).unwrap();
        mgr.unload("kernel32").unwrap();
        // Builtin: ref_count 0 olsa da silinmez (builtin kalır)
        assert!(mgr.dlls.contains_key("kernel32"));
    }

    #[test]
    fn test_registry_read_write() {
        let mut hive = WineRegistryHive::new();
        // Mevcut değeri oku
        let val = hive.read_value(
            "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "ProductName"
        );
        assert!(val.is_some());

        // Yeni değer yaz ve oku
        hive.write_value(
            "HKEY_CURRENT_USER\\Software\\OzkanOS",
            "TestKey",
            WineRegValue::Sz("TestValue".to_string()),
        );
        let written = hive.read_value(
            "HKEY_CURRENT_USER\\Software\\OzkanOS",
            "TestKey"
        );
        assert!(written.is_some());
    }

    #[test]
    fn test_registry_extension_lookup() {
        let hive = WineRegistryHive::new();
        let val = hive.read_value("HKEY_CLASSES_ROOT\\.txt", "");
        assert!(val.is_some());
    }
}