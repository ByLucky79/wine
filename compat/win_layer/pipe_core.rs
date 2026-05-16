// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Pipe / NamedPipe Emulator
// File Path            : apps/system/compat/win_layer/pipe_core.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Description:
//   Anonymous and named pipe emulation for the Win32 compatibility
//   layer. Supports create, connect, disconnect, read, write, peek,
//   and close operations. No_std compatible.
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

#[derive(Debug, Clone)]
pub enum PipeType { Anonymous, Named }

#[derive(Debug, Clone)]
pub struct PipeObject {
    pub handle: u64, pub pipe_type: PipeType, pub name: String,
    pub buffer: Vec<u8>, pub connected: bool, pub read_end: bool, pub write_end: bool,
}

pub struct PipeManager {
    pub pipes: Vec<PipeObject>, pub next_handle: u64,
}

impl Default for PipeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PipeManager {
    pub fn new() -> Self { Self { pipes: Vec::new(), next_handle: 0xC000 } }
    fn alloc_handle(&mut self) -> u64 { let h = self.next_handle; self.next_handle += 1; h }
    pub fn create_pipe(&mut self, read_size: u32, write_size: u32) -> (u64, u64) {
        let read_h = self.alloc_handle(); let write_h = self.alloc_handle();
        self.pipes.push(PipeObject { handle: read_h, pipe_type: PipeType::Anonymous, name: String::new(), buffer: Vec::new(), connected: true, read_end: true, write_end: false });
        self.pipes.push(PipeObject { handle: write_h, pipe_type: PipeType::Anonymous, name: String::new(), buffer: Vec::new(), connected: true, read_end: false, write_end: true });
        let msg = format!("[PIPE] CreatePipe() -> read=0x{:08X} write=0x{:08X} (buf={}/{})", read_h, write_h, read_size, write_size);
        console_writeln(&msg); (read_h, write_h)
    }
    pub fn create_named_pipe(&mut self, name: &str, _open_mode: u32, _pipe_mode: u32, _max_instances: u32, _out_buf: u32, _in_buf: u32, _timeout: u32) -> u64 {
        let h = self.alloc_handle();
        self.pipes.push(PipeObject { handle: h, pipe_type: PipeType::Named, name: String::from(name), buffer: Vec::new(), connected: false, read_end: true, write_end: true });
        let msg = format!("[PIPE] CreateNamedPipe(\"{}\") -> 0x{:08X}", name, h);
        console_writeln(&msg); h
    }
    pub fn connect_named_pipe(&mut self, handle: u64) -> bool {
        if let Some(p) = self.pipes.iter_mut().find(|p| p.handle == handle) { p.connected = true; }
        let msg = format!("[PIPE] ConnectNamedPipe(0x{:08X}) -> true", handle);
        console_writeln(&msg); true
    }
    pub fn disconnect_named_pipe(&mut self, handle: u64) -> bool {
        if let Some(p) = self.pipes.iter_mut().find(|p| p.handle == handle) { p.connected = false; }
        let msg = format!("[PIPE] DisconnectNamedPipe(0x{:08X})", handle);
        console_writeln(&msg); true
    }
    pub fn wait_named_pipe(_name: &str, _timeout: u32) -> bool { console_writeln("[PIPE] WaitNamedPipe() -> true"); true }
    pub fn peek_named_pipe(&self, handle: u64, _buf: u64, _buf_size: u32, _read: u64, avail: &mut u32, _left: u64) -> bool {
        *avail = self.pipes.iter().find(|p| p.handle == handle).map_or(0, |p| p.buffer.len() as u32);
        let msg = format!("[PIPE] PeekNamedPipe(0x{:08X}) -> avail={}", handle, *avail);
        console_writeln(&msg); true
    }
    pub fn write_pipe(&mut self, handle: u64, data: &[u8]) -> u32 {
        if let Some(p) = self.pipes.iter_mut().find(|p| p.handle == handle) { p.buffer.extend_from_slice(data); }
        let msg = format!("[PIPE] WritePipe(0x{:08X}, {} bytes)", handle, data.len());
        console_writeln(&msg); data.len() as u32
    }
    pub fn read_pipe(&mut self, handle: u64, buf: &mut [u8]) -> u32 {
        let count = if let Some(p) = self.pipes.iter_mut().find(|p| p.handle == handle) {
            let n = buf.len().min(p.buffer.len());
            buf[..n].copy_from_slice(&p.buffer[..n]);
            p.buffer.drain(..n);
            n as u32
        } else { 0 };
        let msg = format!("[PIPE] ReadPipe(0x{:08X}, {} bytes)", handle, count);
        console_writeln(&msg); count
    }
    pub fn close_pipe(&mut self, handle: u64) -> bool {
        let before = self.pipes.len();
        self.pipes.retain(|p| p.handle != handle);
        let after = self.pipes.len();
        let msg = format!("[PIPE] ClosePipe(0x{:08X}) -> {}", handle, before != after);
        console_writeln(&msg); before != after
    }
}
