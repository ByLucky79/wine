// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Uyumluluk Katmanı Kök Modülü
// Dosya Yolu         : compat/win32/win32_mod.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Win32 uyumluluk katmanı alt modüllerini bir araya getiren kök modül.
//   Temel tipler, handle tablosu, sanal bellek, süreç tipleri, PE yükleyici,
//   registry, proses yöneticisi, GDI ve syscall emülatörü bu modül üzerinden erişilir.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/win32_handle_table.rs
//   3-) compat/win32/win32_virtual_memory.rs
//   4-) compat/win32/win32_process_types.rs
//   5-) compat/win32/pe_loader.rs
//   6-) compat/win32/pe_resources.rs
//   7-) compat/win32/registry.rs
//   8-) compat/win32/process_mgr.rs
//   9-) compat/win32/gdi.rs
//   10-) compat/win32/kernel_objects.rs
//   11-) compat/win32/app_manager.rs
//   12-) compat/win32/syscall_emu.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// 2026-05-16      Ortak temel Win32 modülleri kök modüle bağlandı
// *******************************************************************

pub mod win32_base_types;
pub mod win32_handle_table;
pub mod win32_virtual_memory;
pub mod win32_process_types;
pub mod pe_loader;
pub mod pe_resources;
pub mod registry;
pub mod process_mgr;
pub mod gdi;
pub mod kernel_objects;
pub mod app_manager;
pub mod syscall_emu;

pub use win32_base_types::*;
pub use win32_handle_table::*;
pub use win32_virtual_memory::*;
pub use win32_process_types::*;
pub use pe_loader::*;
pub use pe_resources::*;
pub use registry::*;
pub use process_mgr::*;
pub use kernel_objects::*;
pub use app_manager::*;
pub use syscall_emu::*;
