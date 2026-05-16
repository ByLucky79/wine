// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 ortak temel tipleri ve durum kodları.
// Dosya Yolu         : compat/win32/win32_base_types.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Bu dosya Win32 uyumluluk katmanının ortak tip, handle, pointer,
//   hata kodu ve bellek koruma sabitlerini tek yerde toplar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_handle_table.rs
//   2-) compat/win32/win32_virtual_memory.rs
//   3-) compat/win32/win32_process_types.rs
//
//              Dosyaya Müdahaleler
// 2026-05-16      Ortak Win32 temel tipler dosyası oluşturuldu
// *******************************************************************

#![allow(dead_code)]

pub type WinBool = i32;
pub type WinByte = u8;
pub type WinWord = u16;
pub type WinDword = u32;
pub type WinQword = u64;
pub type WinLong = i32;
pub type WinUlong = u32;
pub type WinUint = u32;
pub type WinSizeT = usize;
pub type WinHandle = u64;
pub type WinHinstance = WinHandle;
pub type WinHmodule = WinHandle;
pub type WinHwnd = WinHandle;
pub type WinHkey = WinHandle;
pub type WinPtr = usize;
pub type WinLparam = isize;
pub type WinWparam = usize;
pub type WinLresult = isize;

pub const WIN_FALSE: WinBool = 0;
pub const WIN_TRUE: WinBool = 1;
pub const WIN_INVALID_HANDLE_VALUE: WinHandle = !0u64;
pub const WIN_NULL_HANDLE: WinHandle = 0;

pub const WIN_ERROR_SUCCESS: WinDword = 0;
pub const WIN_ERROR_INVALID_FUNCTION: WinDword = 1;
pub const WIN_ERROR_FILE_NOT_FOUND: WinDword = 2;
pub const WIN_ERROR_PATH_NOT_FOUND: WinDword = 3;
pub const WIN_ERROR_ACCESS_DENIED: WinDword = 5;
pub const WIN_ERROR_INVALID_HANDLE: WinDword = 6;
pub const WIN_ERROR_NOT_ENOUGH_MEMORY: WinDword = 8;
pub const WIN_ERROR_INVALID_DATA: WinDword = 13;
pub const WIN_ERROR_OUTOFMEMORY: WinDword = 14;
pub const WIN_ERROR_INVALID_DRIVE: WinDword = 15;
pub const WIN_ERROR_NO_MORE_FILES: WinDword = 18;
pub const WIN_ERROR_WRITE_PROTECT: WinDword = 19;
pub const WIN_ERROR_SHARING_VIOLATION: WinDword = 32;
pub const WIN_ERROR_HANDLE_EOF: WinDword = 38;
pub const WIN_ERROR_NOT_SUPPORTED: WinDword = 50;
pub const WIN_ERROR_FILE_EXISTS: WinDword = 80;
pub const WIN_ERROR_INVALID_PARAMETER: WinDword = 87;
pub const WIN_ERROR_INSUFFICIENT_BUFFER: WinDword = 122;
pub const WIN_ERROR_ALREADY_EXISTS: WinDword = 183;
pub const WIN_ERROR_NOACCESS: WinDword = 998;
pub const WIN_ERROR_DLL_NOT_FOUND: WinDword = 1157;
pub const WIN_ERROR_PROC_NOT_FOUND: WinDword = 127;

pub const WIN_PAGE_NOACCESS: WinDword = 0x01;
pub const WIN_PAGE_READONLY: WinDword = 0x02;
pub const WIN_PAGE_READWRITE: WinDword = 0x04;
pub const WIN_PAGE_WRITECOPY: WinDword = 0x08;
pub const WIN_PAGE_EXECUTE: WinDword = 0x10;
pub const WIN_PAGE_EXECUTE_READ: WinDword = 0x20;
pub const WIN_PAGE_EXECUTE_READWRITE: WinDword = 0x40;

pub const WIN_MEM_COMMIT: WinDword = 0x0000_1000;
pub const WIN_MEM_RESERVE: WinDword = 0x0000_2000;
pub const WIN_MEM_DECOMMIT: WinDword = 0x0000_4000;
pub const WIN_MEM_RELEASE: WinDword = 0x0000_8000;
pub const WIN_MEM_TOP_DOWN: WinDword = 0x0010_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinHandleKind {
    Unknown,
    Process,
    Thread,
    File,
    Event,
    Mutex,
    Semaphore,
    RegistryKey,
    Module,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinProtection {
    pub access: WinDword,
}

impl WinProtection {
    pub const fn new(access: WinDword) -> Self {
        Self { access }
    }

    pub const fn can_read(self) -> bool {
        matches!(
            self.access,
            WIN_PAGE_READONLY | WIN_PAGE_READWRITE | WIN_PAGE_EXECUTE_READ | WIN_PAGE_EXECUTE_READWRITE
        )
    }

    pub const fn can_write(self) -> bool {
        matches!(self.access, WIN_PAGE_READWRITE | WIN_PAGE_EXECUTE_READWRITE | WIN_PAGE_WRITECOPY)
    }

    pub const fn can_execute(self) -> bool {
        matches!(self.access, WIN_PAGE_EXECUTE | WIN_PAGE_EXECUTE_READ | WIN_PAGE_EXECUTE_READWRITE)
    }
}
