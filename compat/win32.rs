// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 Uyumluluk Katmanı Kökü — alt modülleri re-export eder
// Dosya Yolu         : apps/system/compat/win32.rs
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
//   Windows uyumluluk katmanı. PE ayrıştırıcı, registry hive ayrıştırıcı,
//   proses/thread yöneticisi, pencere mesaj kuyruğu, syscall emülatörü.
//   win32/ alt dizinindeki modüllere bölünmüştür.
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/win32/mod.rs
//
//              Dosyaya Müdahaleler
// 2026-04-17      C → Rust çevirisi (no_std)
// 2026-05-13      mod.rs yapısına dönüştürüldü (10 alt dosyaya bölündü)
// *******************************************************************

#[path = "win32/win32_mod.rs"]
pub mod win32;
pub use win32::*;