// *******************************************************************
//                    OZKAN-OS Operating System
//
// File Task Definition : Compatibility Layer (Compat) — Main Crate
// File Path            : apps/system/compat/src/ozkan_compat.rs
// Author               : Ozkan Yildirim
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
//
// Description:
//   Entry point that bundles various OS compatibility layers
//   (DOS, Android, Darwin, Win32, Linux ELF, Java, WASM, Wine)
//   under a single crate. The language system works via kernel_ui.
//
// Dependencies:
//   1-) kernel/graphics/ui/src/lib.rs  (kernel_ui — Lang/MsgId)
//   2-) apps/system/compat/src/core_services.rs
//   3-) apps/system/compat/src/wine_server.rs
//   4-) apps/system/compat/src/compat_manager.rs
//
//              File Modifications
// 2026-04-17      Crate created, all modules added
// 2026-04-18      kernel_ui dependency added, version 2026.04
// *******************************************************************

#![no_std]
extern crate alloc;

pub use kernel_ui::{Lang, MsgId};

pub mod dos_emulator;
pub mod android;
pub mod win32;
pub mod darwin;
pub mod core_services;
pub mod wine_server;
pub mod linux_elf;
pub mod java_bytecode;
pub mod compat_manager;
pub mod wasm_runtime;
pub mod win_layer;
