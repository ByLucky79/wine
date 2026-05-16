// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Window Timer Manager
// File Path            : apps/system/compat/win_layer/timer_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32,
//                        RISC-V 64, MIPS 32, MIPS 64, PowerPC 32,
//                        PowerPC 64, m68k, SPARC, LoongArch64
// *******************************************************************

use alloc::vec::Vec;
use crate::dos_emulator::console_writeln;
use alloc::format;

#[derive(Debug, Clone, Copy)]
pub struct WinTimer {
    pub hwnd: u64, pub id: u64, pub interval: u32, pub elapsed: u32,
}

pub struct TimerManager {
    pub timers: Vec<WinTimer>,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerManager {
    pub fn new() -> Self { Self { timers: Vec::new() } }
    pub fn set_timer(&mut self, hwnd: u64, id: u64, interval: u32) -> u64 {
        self.timers.push(WinTimer { hwnd, id, interval, elapsed: 0 });
        let msg = format!("{} SetTimer(hwnd=0x{:X}, id={}, interval={}ms)", "[WIN32]", hwnd, id, interval);
        console_writeln(&msg); id
    }
    pub fn kill_timer(&mut self, hwnd: u64, id: u64) -> bool {
        let before = self.timers.len();
        self.timers.retain(|t| !(t.hwnd == hwnd && t.id == id));
        let after = self.timers.len();
        let msg = format!("{} KillTimer(hwnd=0x{:X}, id={}) -> {}", "[WIN32]", hwnd, id, before != after);
        console_writeln(&msg); before != after
    }
    pub fn tick(&mut self, delta_ms: u32) {
        for t in &mut self.timers { t.elapsed += delta_ms; }
        let expired: Vec<u64> = self.timers.iter().filter(|t| t.elapsed >= t.interval).map(|t| t.id).collect();
        for id in expired {
            let msg = format!("{} WM_TIMER(id={})", "[WIN32]", id);
            console_writeln(&msg);
            for t in &mut self.timers { if t.id == id { t.elapsed = 0; } }
        }
    }
    pub fn dump(&self) {
        let msg = format!("{} Active timers:", "[WIN32]");
        console_writeln(&msg);
        for t in &self.timers { let msg = format!("  hwnd=0x{:X} id={} interval={}ms", t.hwnd, t.id, t.interval); console_writeln(&msg); }
    }
}
