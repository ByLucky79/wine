// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Registry hive ayrıştırıcı ve yüksek seviye registry API.
// Dosya Yolu         : compat/win32/registry.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Bu dosya registry anahtarları, değerler ve hive düzeyi temel
//   registry işlemlerini süreç/thread tiplerinden ayrılmış biçimde sağlar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/win32_handle_table.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// 2026-05-16      Süreç/thread tipleri registry katmanından ayrıldı
// *******************************************************************

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryValue {
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    Dword(u32),
    Qword(u64),
    MultiString(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct RegistryKey {
    pub name: String,
    pub values: BTreeMap<String, RegistryValue>,
    pub subkeys: BTreeMap<String, RegistryKey>,
}

impl RegistryKey {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            values: BTreeMap::new(),
            subkeys: BTreeMap::new(),
        }
    }

    pub fn set_value(&mut self, value_name: &str, value: RegistryValue) {
        self.values.insert(String::from(value_name), value);
    }

    pub fn get_value(&self, value_name: &str) -> Option<&RegistryValue> {
        self.values.get(value_name)
    }

    pub fn ensure_subkey(&mut self, key_name: &str) -> &mut RegistryKey {
        self.subkeys
            .entry(String::from(key_name))
            .or_insert_with(|| RegistryKey::new(key_name))
    }

    pub fn get_subkey(&self, key_name: &str) -> Option<&RegistryKey> {
        self.subkeys.get(key_name)
    }
}

#[derive(Debug, Clone)]
pub struct RegistryHive {
    pub hive_name: String,
    pub root: RegistryKey,
}

impl RegistryHive {
    pub fn new(hive_name: &str) -> Self {
        Self {
            hive_name: String::from(hive_name),
            root: RegistryKey::new(hive_name),
        }
    }

    pub fn create_path(&mut self, path: &[&str]) -> &mut RegistryKey {
        let mut current = &mut self.root;
        for segment in path {
            current = current.ensure_subkey(segment);
        }
        current
    }

    pub fn open_path(&self, path: &[&str]) -> Option<&RegistryKey> {
        let mut current = &self.root;
        for segment in path {
            current = current.get_subkey(segment)?;
        }
        Some(current)
    }

    pub fn set_path_value(&mut self, path: &[&str], value_name: &str, value: RegistryValue) {
        let key = self.create_path(path);
        key.set_value(value_name, value);
    }
}
