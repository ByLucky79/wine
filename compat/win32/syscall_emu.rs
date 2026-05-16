// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Syscall Emülatörü Kök Modülü — 4 alt modülü re-export eder
// Dosya Yolu         : apps/system/compat/win32/syscall_emu.rs
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
//   Win32 syscall emülatörü 4 alt modüle bölünmüştür:
//   - pe_helpers       : PE Delay Import, Bound Import, CLI Header
//   - win_objects      : Timer, Metrics, Env, Heap, Sync, Atom, Clipboard, Hook
//   - process_mgr_extra: PEB, TEB, FileSearch, SCM, ThreadScheduler
//   - api_emulator     : Win32ApiEmulator impl ve testler
//
// Bağımlı Dosyalar:
//   1-) apps/system/compat/win32/pe_helpers.rs
//   2-) apps/system/compat/win32/win_objects.rs
//   3-) apps/system/compat/win32/process_mgr_extra.rs
//   4-) apps/system/compat/win32/api_emulator.rs
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu (win32.rs bölündü)
// 2026-05-13      2060 satır → 4 alt modüle bölündü
// *******************************************************************

pub mod pe_helpers;
pub mod win_objects;
pub mod process_mgr_extra;
pub mod api_emulator;

pub use pe_helpers::*;
pub use win_objects::*;
pub use process_mgr_extra::*;
pub use api_emulator::*;
