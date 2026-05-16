// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Win32 Compatibility Layer — Module Root
// File Path            : apps/system/compat/win_layer.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Module root for the Windows compatibility type system.
//   Re-exports submodules so callers may use win_layer::native_base
//   or win_layer::shared_defs directly.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

pub mod native_base;
pub mod shared_defs;
pub mod kernel_object;
pub mod timer_core;
pub mod system_env;
pub mod heap_core;
pub mod sync_core;
pub mod atom_core;

pub mod registry_engine;
pub mod console_core;
pub mod window_core;
pub mod gdi_core;
pub mod scm_core;
pub mod pipe_core;
pub mod seh_core;
pub mod mailslot_core;
pub mod api_emulator;
pub mod subclass_core;
pub mod syscall_core;
pub mod pe_engine;
