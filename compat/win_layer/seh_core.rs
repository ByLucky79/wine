// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Structured Exception Handling (SEH) Core
// File Path            : apps/system/compat/win_layer/seh_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   SEH manager with exception handler chain and unhandled filter
//   support for the Win32 compatibility layer. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::vec::Vec;
use crate::win_layer::shared_defs::ExceptionDisposition;

#[derive(Debug, Clone)]
pub struct ExceptionRecord {
    pub exception_code: u32,
    pub exception_flags: u32,
    pub exception_address: u64,
    pub number_parameters: u32,
    pub exception_information: [u64; 15],
}

impl ExceptionRecord {
    pub fn new(code: u32, address: u64) -> Self {
        Self { exception_code: code, exception_flags: 0, exception_address: address, number_parameters: 0, exception_information: [0; 15] }
    }
}

#[derive(Debug, Clone)]
pub struct ContextRecord {
    pub ip: u64, pub sp: u64, pub bp: u64,
    pub ax: u64, pub bx: u64, pub cx: u64, pub dx: u64,
    pub si: u64, pub di: u64,
}

impl ContextRecord {
    pub fn new() -> Self {
        Self { ip: 0, sp: 0, bp: 0, ax: 0, bx: 0, cx: 0, dx: 0, si: 0, di: 0 }
    }
}

pub type ExceptionHandlerFn = fn(&ExceptionRecord, &mut ContextRecord) -> ExceptionDisposition;

#[derive(Debug, Clone)]
pub struct SehManager {
    pub handlers: Vec<(u64, ExceptionHandlerFn)>,
    pub unhandled_filter: Option<ExceptionHandlerFn>,
}

impl Default for SehManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SehManager {
    pub fn new() -> Self {
        Self { handlers: Vec::new(), unhandled_filter: None }
    }
    pub fn add_handler(&mut self, id: u64, handler: ExceptionHandlerFn) {
        self.handlers.push((id, handler));
    }
    pub fn remove_handler(&mut self, id: u64) -> bool {
        if let Some(idx) = self.handlers.iter().position(|(i, _)| *i == id) {
            self.handlers.remove(idx); true
        } else { false }
    }
    pub fn raise_exception(&mut self, record: &ExceptionRecord, context: &mut ContextRecord) -> ExceptionDisposition {
        for (_id, handler) in &self.handlers {
            let result = handler(record, context);
            if matches!(result, ExceptionDisposition::ExceptionContinueExecution) {
                return ExceptionDisposition::ExceptionContinueExecution;
            }
        }
        if let Some(filter) = self.unhandled_filter {
            return filter(record, context);
        }
        ExceptionDisposition::ExceptionContinueSearch
    }
    pub fn set_unhandled_filter(&mut self, filter: ExceptionHandlerFn) {
        self.unhandled_filter = Some(filter);
    }
}
