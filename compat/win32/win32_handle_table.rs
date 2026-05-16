// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 handle tablosu ve nesne kayıt yöneticisi.
// Dosya Yolu         : compat/win32/win32_handle_table.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Bu dosya process, thread, file, event, mutex ve benzeri Win32
//   nesneleri için ortak handle ayırma, arama ve kapatma tablosunu sağlar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/kernel_objects.rs
//   3-) compat/win32/process_mgr.rs
//
//              Dosyaya Müdahaleler
// 2026-05-16      Ortak Win32 handle tablosu dosyası oluşturuldu
// *******************************************************************

#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::win32::win32_base_types::{WinDword, WinHandle, WinHandleKind, WIN_ERROR_INVALID_HANDLE};

#[derive(Debug, Clone)]
pub struct WinHandleEntry {
    pub handle: WinHandle,
    pub kind: WinHandleKind,
    pub access_mask: WinDword,
    pub object_value: u64,
    pub name: String,
    pub inheritable: bool,
    pub reference_count: u32,
}

impl WinHandleEntry {
    pub fn new(
        handle: WinHandle,
        kind: WinHandleKind,
        access_mask: WinDword,
        object_value: u64,
        name: &str,
        inheritable: bool,
    ) -> Self {
        Self {
            handle,
            kind,
            access_mask,
            object_value,
            name: String::from(name),
            inheritable,
            reference_count: 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct WinHandleTable {
    next_handle: WinHandle,
    entries: Vec<WinHandleEntry>,
}

impl WinHandleTable {
    pub fn new() -> Self {
        Self {
            next_handle: 0x1000,
            entries: Vec::new(),
        }
    }

    pub fn allocate(
        &mut self,
        kind: WinHandleKind,
        access_mask: WinDword,
        object_value: u64,
        name: &str,
        inheritable: bool,
    ) -> WinHandle {
        let handle = self.next_handle;
        self.next_handle = self.next_handle.saturating_add(4);
        self.entries.push(WinHandleEntry::new(
            handle,
            kind,
            access_mask,
            object_value,
            name,
            inheritable,
        ));
        handle
    }

    pub fn close(&mut self, handle: WinHandle) -> bool {
        if let Some(index) = self.entries.iter().position(|entry| entry.handle == handle) {
            let refs = self.entries[index].reference_count;
            if refs > 1 {
                self.entries[index].reference_count -= 1;
            } else {
                self.entries.remove(index);
            }
            true
        } else {
            false
        }
    }

    pub fn duplicate(&mut self, handle: WinHandle, inheritable: bool) -> Result<WinHandle, WinDword> {
        let source = self
            .entries
            .iter_mut()
            .find(|entry| entry.handle == handle)
            .ok_or(WIN_ERROR_INVALID_HANDLE)?;
        source.reference_count = source.reference_count.saturating_add(1);
        let duplicated_handle = self.next_handle;
        self.next_handle = self.next_handle.saturating_add(4);
        self.entries.push(WinHandleEntry {
            handle: duplicated_handle,
            kind: source.kind,
            access_mask: source.access_mask,
            object_value: source.object_value,
            name: source.name.clone(),
            inheritable,
            reference_count: source.reference_count,
        });
        Ok(duplicated_handle)
    }

    pub fn get(&self, handle: WinHandle) -> Option<&WinHandleEntry> {
        self.entries.iter().find(|entry| entry.handle == handle)
    }

    pub fn get_mut(&mut self, handle: WinHandle) -> Option<&mut WinHandleEntry> {
        self.entries.iter_mut().find(|entry| entry.handle == handle)
    }

    pub fn object_value(&self, handle: WinHandle, expected_kind: WinHandleKind) -> Option<u64> {
        self.entries
            .iter()
            .find(|entry| entry.handle == handle && entry.kind == expected_kind)
            .map(|entry| entry.object_value)
    }

    pub fn count_by_kind(&self, kind: WinHandleKind) -> usize {
        self.entries.iter().filter(|entry| entry.kind == kind).count()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_handle = 0x1000;
    }
}
