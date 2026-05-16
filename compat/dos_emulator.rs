// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : DOS Emülasyon Katmanı Kök Modülü — 5 alt modülü re-export eder
// Dosya Yolu         : apps/system/compat/dos_emulator.rs
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
//   OZ-DOS 2026 uyumluluk katmanı 5 alt modüle bölünmüştür:
//   - dos_console    : konsol çıktısı, PSP, file handle, memory block
//   - dos_core       : DosEmulator ana struct ve impl
//   - dos_loaders    : MZ header, EXE/COM yükleyici, FAT parser, FCB
//   - dos_int21      : DosProcess, Registers, INT 21h emülatörü
//   - dos_shell      : SpinLock, sürücü tablosu, DosShell
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/dos_console.rs
//   2-) apps/system/compat/dos_core.rs
//   3-) apps/system/compat/dos_loaders.rs
//   4-) apps/system/compat/dos_int21.rs
//   5-) apps/system/compat/dos_shell.rs
//
//              Dosyaya Müdahaleler
// 2026-04-17      C → Rust çevirisi
// 2026-05-13      2619 satır → 5 alt modüle bölündü
// *******************************************************************

pub mod dos_console;
pub mod dos_core;
pub mod dos_loaders;
pub mod dos_int21;
pub mod dos_shell;

pub use dos_console::*;
pub use dos_core::*;
pub use dos_loaders::*;
pub use dos_int21::*;
pub use dos_shell::*;
