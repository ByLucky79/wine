// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Service Control Manager (SCM) Core
// File Path            : apps/system/compat/win_layer/scm_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Windows Service Control Manager emulation. Full service
//   lifecycle state machine: Stopped → StartPending → Running,
//   Running → StopPending → Stopped, Running → PausePending → Paused,
//   Paused → ContinuePending → Running. No_std compatible.
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

#![allow(dead_code)]

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;
use crate::win_layer::shared_defs::ServiceStatus;

#[derive(Debug, Clone)]
pub struct ServiceEntry {
    pub handle: u64, pub name: String, pub display_name: String, pub status: ServiceStatus, pub start_type: u32,
    pub path: String, pub dependencies: Vec<String>,
}

pub struct ServiceControlManager {
    pub services: Vec<ServiceEntry>, pub db_handle: u64, pub next_handle: u64,
}

impl Default for ServiceControlManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceControlManager {
    pub fn new() -> Self { Self { services: Vec::new(), db_handle: 0xD000, next_handle: 0xD100 } }
    pub fn open_sc_manager(_machine: &str, _db: &str, _access: u32) -> u64 {
        let msg = format!("[SCM] OpenSCManager() -> 0x{:08X}", 0xD000);
        console_writeln(&msg); 0xD000
    }
    fn find_service(&self, handle: u64) -> Option<&ServiceEntry> {
        self.services.iter().find(|s| s.handle == handle)
    }
    fn find_service_mut(&mut self, handle: u64) -> Option<&mut ServiceEntry> {
        self.services.iter_mut().find(|s| s.handle == handle)
    }
    pub fn open_service(&mut self, _scm: u64, name: &str, _access: u32) -> u64 {
        if let Some(s) = self.services.iter().find(|s| s.name.eq_ignore_ascii_case(name)) {
            let msg = format!("[SCM] OpenService(\"{}\") -> 0x{:08X}", name, s.handle);
            console_writeln(&msg); return s.handle;
        }
        let msg = format!("[SCM] OpenService(\"{}\") -> 0 (not found)", name);
        console_writeln(&msg); 0
    }
    pub fn create_service(&mut self, _scm: u64, name: &str, display: &str, _access: u32, _svc_type: u32, start_type: u32, _error_control: u32, path: &str, _group: &str) -> u64 {
        if self.services.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
            let msg = format!("[SCM] CreateService(\"{}\") -> 0 (already exists)", name);
            console_writeln(&msg); return 0;
        }
        let h = self.next_handle; self.next_handle += 1;
        self.services.push(ServiceEntry { handle: h, name: String::from(name), display_name: String::from(display), status: ServiceStatus::Stopped, start_type, path: String::from(path), dependencies: Vec::new() });
        let msg = format!("[SCM] CreateService(\"{}\", \"{}\", start={}) -> 0x{:08X}", name, display, start_type, h);
        console_writeln(&msg); h
    }
    pub fn delete_service(&mut self, handle: u64) -> bool {
        if let Some(idx) = self.services.iter().position(|s| s.handle == handle) {
            let name = self.services[idx].name.clone();
            self.services.remove(idx);
            let msg = format!("[SCM] DeleteService(\"{}\") -> true", name);
            console_writeln(&msg); true
        } else {
            let msg = format!("[SCM] DeleteService(0x{:08X}) -> false (not found)", handle);
            console_writeln(&msg); false
        }
    }
    pub fn start_service(&mut self, handle: u64, _args: &[String]) -> bool {
        if let Some(s) = self.find_service_mut(handle) {
            if !matches!(s.status, ServiceStatus::Stopped) {
                let msg = format!("[SCM] StartService(\"{}\") -> false (not stopped)", s.name);
                console_writeln(&msg); return false;
            }
            s.status = ServiceStatus::StartPending;
            let msg = format!("[SCM] StartService(\"{}\") -> StartPending", s.name);
            console_writeln(&msg);
            s.status = ServiceStatus::Running;
            let msg = format!("[SCM] StartService(\"{}\") -> Running", s.name);
            console_writeln(&msg); true
        } else {
            let msg = format!("[SCM] StartService(0x{:08X}) -> false (not found)", handle);
            console_writeln(&msg); false
        }
    }
    pub fn stop_service(&mut self, handle: u64) -> bool {
        if let Some(s) = self.find_service_mut(handle) {
            if !matches!(s.status, ServiceStatus::Running) {
                let msg = format!("[SCM] StopService(\"{}\") -> false (not running)", s.name);
                console_writeln(&msg); return false;
            }
            s.status = ServiceStatus::StopPending;
            let msg = format!("[SCM] StopService(\"{}\") -> StopPending", s.name);
            console_writeln(&msg);
            s.status = ServiceStatus::Stopped;
            let msg = format!("[SCM] StopService(\"{}\") -> Stopped", s.name);
            console_writeln(&msg); true
        } else {
            let msg = format!("[SCM] StopService(0x{:08X}) -> false (not found)", handle);
            console_writeln(&msg); false
        }
    }
    pub fn pause_service(&mut self, handle: u64) -> bool {
        if let Some(s) = self.find_service_mut(handle) {
            if !matches!(s.status, ServiceStatus::Running) {
                let msg = format!("[SCM] PauseService(\"{}\") -> false (not running)", s.name);
                console_writeln(&msg); return false;
            }
            s.status = ServiceStatus::PausePending;
            let msg = format!("[SCM] PauseService(\"{}\") -> PausePending", s.name);
            console_writeln(&msg);
            s.status = ServiceStatus::Paused;
            let msg = format!("[SCM] PauseService(\"{}\") -> Paused", s.name);
            console_writeln(&msg); true
        } else {
            let msg = format!("[SCM] PauseService(0x{:08X}) -> false (not found)", handle);
            console_writeln(&msg); false
        }
    }
    pub fn continue_service(&mut self, handle: u64) -> bool {
        if let Some(s) = self.find_service_mut(handle) {
            if !matches!(s.status, ServiceStatus::Paused) {
                let msg = format!("[SCM] ContinueService(\"{}\") -> false (not paused)", s.name);
                console_writeln(&msg); return false;
            }
            s.status = ServiceStatus::ContinuePending;
            let msg = format!("[SCM] ContinueService(\"{}\") -> ContinuePending", s.name);
            console_writeln(&msg);
            s.status = ServiceStatus::Running;
            let msg = format!("[SCM] ContinueService(\"{}\") -> Running", s.name);
            console_writeln(&msg); true
        } else {
            let msg = format!("[SCM] ContinueService(0x{:08X}) -> false (not found)", handle);
            console_writeln(&msg); false
        }
    }
    pub fn control_service(&mut self, handle: u64, control: u32) -> bool {
        match control {
            1 => self.stop_service(handle),     // SERVICE_CONTROL_STOP
            2 => self.pause_service(handle),    // SERVICE_CONTROL_PAUSE
            3 => self.continue_service(handle), // SERVICE_CONTROL_CONTINUE
            _ => {
                let msg = format!("[SCM] ControlService(0x{:08X}, {}) -> unknown control", handle, control);
                console_writeln(&msg); false
            }
        }
    }
    pub fn query_service_status(&self, handle: u64) -> ServiceStatus {
        if let Some(s) = self.find_service(handle) {
            let msg = format!("[SCM] QueryServiceStatus(\"{}\") -> {:?}", s.name, s.status);
            console_writeln(&msg); s.status
        } else {
            let msg = format!("[SCM] QueryServiceStatus(0x{:08X}) -> Stopped (not found)", handle);
            console_writeln(&msg); ServiceStatus::Stopped
        }
    }
    pub fn close_service_handle(&self, _handle: u64) -> bool { true }
    pub fn enum_services(&self) {
        let msg = format!("[SCM] {} services registered:", self.services.len());
        console_writeln(&msg);
        for s in &self.services { let msg = format!("  {} (\"{}\") status={:?} path={}", s.name, s.display_name, s.status, s.path); console_writeln(&msg); }
    }
}
