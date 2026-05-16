// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Uyumluluk Katmanı Kök Modülü
// Dosya Yolu         : apps/system/compat/win32/win32_mod.rs
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
//   Win32 uyumluluk katmanı alt modüllerini bir araya getiren kök modül.
//   PE yükleyici, registry, proses yöneticisi, GDI ve syscall emülatörü
//   bu modül üzerinden erişilebilir.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/win32/pe_loader.rs
//   2-) apps/system/compat/win32/pe_resources.rs
//   3-) apps/system/compat/win32/registry.rs
//   4-) apps/system/compat/win32/process_mgr.rs
//   5-) apps/system/compat/win32/gdi.rs
//   6-) apps/system/compat/win32/console.rs
//   7-) apps/system/compat/win32/kernel_objects.rs
//   8-) apps/system/compat/win32/app_manager.rs
//   9-) apps/system/compat/win32/syscall_emu.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// *******************************************************************

pub mod pe_loader;
pub mod pe_resources;
pub mod registry;
pub mod process_mgr;
pub mod gdi;
pub mod kernel_objects;
pub mod app_manager;
pub mod syscall_emu;

pub use pe_loader::*;
pub use pe_resources::*;
pub use registry::*;
pub use process_mgr::*;
pub use kernel_objects::*;
pub use app_manager::*;
pub use syscall_emu::*;
