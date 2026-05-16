// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Win32 Syscall Emulator
// File Path            : apps/system/compat/win_layer/syscall_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   NT syscall emulation layer (NtCreateFile, NtAllocateVirtualMemory,
//   NtCreateThread, etc.) for the Win32 compatibility layer.
//   No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use crate::dos_emulator::console_writeln;
use crate::win_layer::shared_defs::{NtStatus, RegistryValue, Win32Section};
use crate::win32::{Win32Manager, WinProcess, ProcessMemoryManager};
use crate::win_layer::api_emulator::Win32ApiEmulator;

// ─── Win32 Syscall Emulator ────────────────────────────────────

pub struct SyscallEmulator;

impl SyscallEmulator {
    pub fn nt_create_file(mgr: &mut Win32Manager, name: &str, desired_access: u32, create_disposition: u32) -> (NtStatus, u64) {
        let handle = Win32ApiEmulator::create_file(mgr, name, desired_access, 0, create_disposition, 0);
        if handle != 0xFFFFFFFF {
            let msg = format!("{} NtCreateFile({}) -> handle=0x{:08X}", "[WIN32]", name, handle);
            console_writeln(&msg); (NtStatus::Success, handle)
        } else {
            let msg = format!("{} NtCreateFile({}) -> {} err={}", "[WIN32]", name, "[WIN32]", mgr.last_error);
            console_writeln(&msg); (NtStatus::InvalidHandle, 0)
        }
    }
    pub fn nt_read_file(mgr: &mut Win32Manager, file_handle: u64, buffer: &mut [u8]) -> NtStatus {
        let (ok, read) = Win32ApiEmulator::read_file(mgr, file_handle, buffer);
        let msg = format!("{} NtReadFile handle=0x{:08X}, read={}", "[WIN32]", file_handle, read);
        console_writeln(&msg); if ok { NtStatus::Success } else { NtStatus::InvalidHandle }
    }
    pub fn nt_write_file(mgr: &mut Win32Manager, file_handle: u64, buffer: &[u8]) -> NtStatus {
        let (ok, written) = Win32ApiEmulator::write_file(mgr, file_handle, buffer);
        let msg = format!("{} NtWriteFile handle=0x{:08X}, written={}", "[WIN32]", file_handle, written);
        console_writeln(&msg); if ok { NtStatus::Success } else { NtStatus::InvalidHandle }
    }
    pub fn nt_allocate_virtual_memory(pmm: &mut ProcessMemoryManager, base_address: &mut u64, _zero_bits: usize, region_size: &mut u64, _allocation_type: u32, protect: u32) -> NtStatus {
        let addr = if *base_address == 0 { pmm.allocate(*region_size, protect) } else { *base_address };
        *base_address = addr;
        let msg = format!("{} NtAllocateVirtualMemory addr=0x{:016X}, size=0x{:016X}, prot=0x{:08X}", "[WIN32]", addr, *region_size, protect);
        console_writeln(&msg); NtStatus::Success
    }
    pub fn nt_free_virtual_memory(pmm: &mut ProcessMemoryManager, base_address: &mut u64, _region_size: &mut u64, _free_type: u32) -> NtStatus {
        let ok = pmm.free(*base_address);
        let msg = format!("{} NtFreeVirtualMemory addr=0x{:016X} -> {}", "[WIN32]", *base_address, ok);
        console_writeln(&msg); if ok { NtStatus::Success } else { NtStatus::InvalidParameter }
    }
    pub fn nt_create_thread(process: &mut WinProcess, _desired_access: u32, _object_attributes: u64, _process_handle: u64, _client_id: u64, _context: u64, _initial_teb: u64, create_suspended: bool) -> (NtStatus, u32) {
        let tid = process.create_thread(0x401000);
        if create_suspended { if let Some(t) = process.threads.iter_mut().find(|t| t.tid == tid) { t.suspend(); } }
        let msg = format!("{} NtCreateThread -> TID={}, suspended={}", "[WIN32]", tid, create_suspended);
        console_writeln(&msg); (NtStatus::Success, tid)
    }
    pub fn nt_terminate_thread(process: &mut WinProcess, tid: u32, exit_status: u32) -> NtStatus {
        let ok = Win32ApiEmulator::exit_thread(process, tid, exit_status);
        let msg = format!("{} NtTerminateThread TID={}, status={}", "[WIN32]", tid, exit_status);
        console_writeln(&msg); if ok { NtStatus::Success } else { NtStatus::InvalidHandle }
    }
    pub fn nt_query_information_process(process: &WinProcess, _process_handle: u64, info_class: u32, _info: u64, _info_length: u32, _return_length: u64) -> NtStatus {
        let msg = format!("{} NtQueryInformationProcess PID={}, class={}", "[WIN32]", process.pid, info_class);
        console_writeln(&msg); NtStatus::Success
    }
    pub fn nt_open_key(mgr: &mut Win32Manager, path: &str) -> (NtStatus, u64) {
        let handle = Win32ApiEmulator::reg_open_key_ex(mgr, path);
        if handle != 0 {
            let msg = format!("{}: {} -> handle=0x{:08X}", "[WIN32]", path, handle);
            console_writeln(&msg); (NtStatus::Success, handle)
        } else { (NtStatus::InvalidHandle, 0) }
    }
    pub fn nt_query_value_key(mgr: &Win32Manager, path: &str, value_name: &str) -> NtStatus {
        if let Some(_val) = Win32ApiEmulator::reg_query_value_ex(mgr, path, value_name) {
            let msg = format!("{}: {}\\{} -> ok", "[WIN32]", path, value_name);
            console_writeln(&msg); NtStatus::Success
        } else {
            let msg = format!("{}: {}\\{} -> {}", "[WIN32]", path, value_name, "[WIN32]");
            console_writeln(&msg); NtStatus::InvalidHandle
        }
    }
    pub fn nt_set_value_key(mgr: &mut Win32Manager, path: &str, value_name: &str, value: RegistryValue) -> NtStatus {
        if let Some(key) = mgr.registry.get_mut(path) {
            key.set_value(value_name, value);
            let msg = format!("{}: {}\\{}", "[WIN32]", path, value_name);
            console_writeln(&msg); NtStatus::Success
        } else { NtStatus::InvalidHandle }
    }
    pub fn nt_close(mgr: &mut Win32Manager, handle: u64) -> NtStatus {
        if Win32ApiEmulator::close_handle(mgr, handle) { NtStatus::Success } else { NtStatus::InvalidHandle }
    }
    pub fn nt_delay_execution(_alertable: bool, delay_interval: i64) -> NtStatus {
        let msg = format!("{} NtDelayExecution interval={}ms", "[WIN32]", delay_interval / -10000);
        console_writeln(&msg); NtStatus::Success
    }
    pub fn nt_query_system_time(time: &mut u64) -> NtStatus { *time = 0x01D9000000000000; NtStatus::Success }
    pub fn nt_open_file(mgr: &mut Win32Manager, name: &str, desired_access: u32) -> (NtStatus, u64) {
        let handle = Win32ApiEmulator::create_file(mgr, name, desired_access, 0, 3, 0);
        if handle != 0xFFFFFFFF {
            let msg = format!("{} NtOpenFile({}) -> handle=0x{:08X}", "[WIN32]", name, handle);
            console_writeln(&msg); (NtStatus::Success, handle)
        } else {
            let msg = format!("{} NtOpenFile({}) -> failed err={}", "[WIN32]", name, mgr.last_error);
            console_writeln(&msg); (NtStatus::InvalidHandle, 0)
        }
    }
    pub fn nt_query_information_file(mgr: &Win32Manager, file_handle: u64, size: &mut u64, position: &mut u64) -> NtStatus {
        if let Some(fh) = mgr.files.get(&file_handle) {
            *size = fh.data.len() as u64;
            *position = fh.pos as u64;
            let msg = format!("{} NtQueryInformationFile handle=0x{:08X}, size={}, pos={}", "[WIN32]", file_handle, size, position);
            console_writeln(&msg); NtStatus::Success
        } else { NtStatus::InvalidHandle }
    }
    pub fn nt_set_information_file(mgr: &mut Win32Manager, file_handle: u64, position: u64) -> NtStatus {
        if let Some(fh) = mgr.files.get_mut(&file_handle) {
            fh.pos = position as usize;
            let msg = format!("{} NtSetInformationFile handle=0x{:08X}, pos={}", "[WIN32]", file_handle, position);
            console_writeln(&msg); NtStatus::Success
        } else { NtStatus::InvalidHandle }
    }
    pub fn nt_protect_virtual_memory(pmm: &mut ProcessMemoryManager, address: u64, _size: u64, new_protect: u32) -> NtStatus {
        let ok = pmm.protect(address, new_protect);
        let msg = format!("{} NtProtectVirtualMemory addr=0x{:016X}, prot=0x{:08X} -> {}", "[WIN32]", address, new_protect, ok);
        console_writeln(&msg); if ok { NtStatus::Success } else { NtStatus::InvalidParameter }
    }
    pub fn nt_query_virtual_memory(pmm: &ProcessMemoryManager, address: u64, base: &mut u64, size: &mut u64, protect: &mut u32) -> NtStatus {
        if let Some(region) = pmm.query(address) {
            *base = region.base; *size = region.size; *protect = region.prot;
            let msg = format!("{} NtQueryVirtualMemory addr=0x{:016X} -> base=0x{:016X}, size=0x{:016X}, prot=0x{:08X}", "[WIN32]", address, region.base, region.size, region.prot);
            console_writeln(&msg); NtStatus::Success
        } else {
            let msg = format!("{} NtQueryVirtualMemory addr=0x{:016X} -> not found", "[WIN32]", address);
            console_writeln(&msg); NtStatus::InvalidParameter
        }
    }
    pub fn nt_create_section(mgr: &mut Win32Manager, size: u64) -> (NtStatus, u64) {
        let handle = mgr.next_handle; mgr.next_handle += 1;
        mgr.sections.insert(handle, Win32Section { handle, size, base: 0 });
        let msg = format!("{} NtCreateSection size=0x{:016X} -> handle=0x{:08X}", "[WIN32]", size, handle);
        console_writeln(&msg); (NtStatus::Success, handle)
    }
    pub fn nt_map_view_of_section(mgr: &mut Win32Manager, section_handle: u64, pmm: &mut ProcessMemoryManager, base: &mut u64) -> NtStatus {
        if let Some(sec) = mgr.sections.get_mut(&section_handle) {
            let addr = if *base == 0 { pmm.allocate(sec.size, 0x04) } else { *base };
            sec.base = addr; *base = addr;
            let msg = format!("{} NtMapViewOfSection handle=0x{:08X} -> base=0x{:016X}", "[WIN32]", section_handle, addr);
            console_writeln(&msg); NtStatus::Success
        } else { NtStatus::InvalidHandle }
    }
    pub fn nt_unmap_view_of_section(mgr: &mut Win32Manager, section_handle: u64, pmm: &mut ProcessMemoryManager) -> NtStatus {
        if let Some(sec) = mgr.sections.get(&section_handle) {
            let ok = pmm.free(sec.base);
            let msg = format!("{} NtUnmapViewOfSection handle=0x{:08X} -> {}", "[WIN32]", section_handle, ok);
            console_writeln(&msg); if ok { NtStatus::Success } else { NtStatus::InvalidParameter }
        } else { NtStatus::InvalidHandle }
    }
}
