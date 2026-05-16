// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : WebAssembly Runtime Uyumluluk Katmanı
// Dosya Yolu         : apps/system/compat/src/wasm_runtime.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Açıklama:
//   Gerçek WASM binary parser (LEB128, Section'lar, Type/Import/
//   Function/Export/Code) ve mini stack-based interpreter.
// *******************************************************************

#![allow(dead_code)]

use crate::dos_emulator::console_writeln;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use kernel_ui::{Lang, MsgId};

// ─── LEB128 Decoder ────────────────────────────────────────────

pub struct Leb128Decoder;

impl Leb128Decoder {
    pub fn decode_u32(data: &[u8], offset: usize) -> (u32, usize) {
        let mut result = 0u32; let mut shift = 0u32; let mut off = offset;
        loop {
            if off >= data.len() { break; }
            let byte = data[off]; off += 1;
            result |= ((byte & 0x7F) as u32) << shift;
            if (byte & 0x80) == 0 { break; }
            shift += 7;
            if shift >= 32 { break; }
        }
        (result, off)
    }
    pub fn decode_u64(data: &[u8], offset: usize) -> (u64, usize) {
        let mut result = 0u64; let mut shift = 0u32; let mut off = offset;
        loop {
            if off >= data.len() { break; }
            let byte = data[off]; off += 1;
            result |= ((byte & 0x7F) as u64) << shift;
            if (byte & 0x80) == 0 { break; }
            shift += 7;
            if shift >= 64 { break; }
        }
        (result, off)
    }
    pub fn decode_i32(data: &[u8], offset: usize) -> (i32, usize) {
        let mut result = 0i32; let mut shift = 0u32; let mut off = offset;
        loop {
            if off >= data.len() { break; }
            let byte = data[off]; off += 1;
            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;
            if (byte & 0x80) == 0 {
                if shift < 32 && (byte & 0x40) != 0 { result |= !0 << shift; }
                break;
            }
            if shift >= 32 { break; }
        }
        (result, off)
    }
    pub fn decode_i64(data: &[u8], offset: usize) -> (i64, usize) {
        let mut result = 0i64; let mut shift = 0u32; let mut off = offset;
        loop {
            if off >= data.len() { break; }
            let byte = data[off]; off += 1;
            result |= ((byte & 0x7F) as i64) << shift;
            shift += 7;
            if (byte & 0x80) == 0 {
                if shift < 64 && (byte & 0x40) != 0 { result |= !0 << shift; }
                break;
            }
            if shift >= 64 { break; }
        }
        (result, off)
    }
}

// ─── WASM Header Parser ────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct WasmHeader {
    pub magic: u32, pub version: u32,
}

impl WasmHeader {
    pub const WASM_MAGIC: u32 = 0x6D736100;
    pub const WASM_VERSION: u32 = 0x00000001;
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != Self::WASM_MAGIC { return None; }
        Some(Self { magic, version: u32::from_le_bytes([data[4], data[5], data[6], data[7]]) })
    }
}

// ─── WASM Section Parser ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WasmSection {
    pub id: u8, pub size: u32, pub data: Vec<u8>,
}

impl WasmSection {
    pub fn parse(data: &[u8], offset: usize) -> Option<(Self, usize)> {
        if offset >= data.len() { return None; }
        let id = data[offset];
        let (size, mut off) = Leb128Decoder::decode_u32(data, offset + 1);
        if data.len() < off + size as usize { return None; }
        let section_data = data[off..off + size as usize].to_vec();
        off += size as usize;
        Some((Self { id, size, data: section_data }, off))
    }
    pub fn parse_all(data: &[u8]) -> Vec<Self> {
        let mut sections = Vec::new();
        let mut off = 8usize; // skip header
        while let Some((sec, new_off)) = Self::parse(data, off) {
            sections.push(sec); off = new_off;
        }
        sections
    }
}

// ─── WASM Type Section Parser ──────────────────────────────────

#[derive(Debug, Clone)]
pub enum WasmValType {
    I32, I64, F32, F64, Void,
}

impl WasmValType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x7F => Some(Self::I32), 0x7E => Some(Self::I64),
            0x7D => Some(Self::F32), 0x7C => Some(Self::F64),
            _ => None,
        }
    }
    pub fn name(&self) -> &'static str {
        match self { Self::I32 => "i32", Self::I64 => "i64", Self::F32 => "f32", Self::F64 => "f64", Self::Void => "void" }
    }
}

#[derive(Debug, Clone)]
pub struct WasmFuncType {
    pub params: Vec<WasmValType>, pub results: Vec<WasmValType>,
}

pub struct WasmTypeParser;

impl WasmTypeParser {
    pub fn parse(data: &[u8]) -> Vec<WasmFuncType> {
        let mut types = Vec::new();
        let mut off = 0usize;
        let (count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
        for _ in 0..count {
            if off >= data.len() { break; }
            let form = data[off]; off += 1;
            if form != 0x60 { continue; }
            let (param_count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
            let mut params = Vec::new();
            for _ in 0..param_count {
                if off >= data.len() { break; }
                if let Some(t) = WasmValType::from_byte(data[off]) { params.push(t); }
                off += 1;
            }
            let (result_count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
            let mut results = Vec::new();
            for _ in 0..result_count {
                if off >= data.len() { break; }
                if let Some(t) = WasmValType::from_byte(data[off]) { results.push(t); }
                off += 1;
            }
            types.push(WasmFuncType { params, results });
        }
        types
    }
}

// ─── WASM Export Section Parser ────────────────────────────────

#[derive(Debug, Clone)]
pub struct WasmExport {
    pub name: String, pub kind: u8, pub index: u32,
}

pub struct WasmExportParser;

impl WasmExportParser {
    pub fn parse(data: &[u8]) -> Vec<WasmExport> {
        let mut exports = Vec::new();
        let mut off = 0usize;
        let (count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
        for _ in 0..count {
            if off >= data.len() { break; }
            let (name_len, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
            let name = if data.len() >= off + name_len as usize {
                core::str::from_utf8(&data[off..off + name_len as usize]).unwrap_or("").to_string()
            } else { String::new() };
            off += name_len as usize;
            if off >= data.len() { break; }
            let kind = data[off]; off += 1;
            let (index, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
            exports.push(WasmExport { name, kind, index });
        }
        exports
    }
}

// ─── WASM Function Body Parser ─────────────────────────────────

#[derive(Debug, Clone)]
pub struct WasmFuncBody {
    pub locals: Vec<(u32, WasmValType)>, pub code: Vec<u8>,
}

pub struct WasmFuncBodyParser;

impl WasmFuncBodyParser {
    pub fn parse(data: &[u8]) -> Option<WasmFuncBody> {
        let mut off = 0usize;
        let (local_count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
        let mut locals = Vec::new();
        for _ in 0..local_count {
            if off + 2 > data.len() { break; }
            let (count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
            if off >= data.len() { break; }
            let type_byte = data[off]; off += 1;
            if let Some(t) = WasmValType::from_byte(type_byte) { locals.push((count, t)); }
        }
        let code = data[off..].to_vec();
        Some(WasmFuncBody { locals, code })
    }
}

// ─── WASM Code Section Parser ──────────────────────────────────

#[derive(Debug, Clone)]
pub struct WasmCodeSection {
    pub bodies: Vec<WasmFuncBody>,
}

pub struct WasmCodeParser;

impl WasmCodeParser {
    pub fn parse(data: &[u8]) -> Vec<WasmFuncBody> {
        let mut bodies = Vec::new();
        let mut off = 0usize;
        let (count, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
        for _ in 0..count {
            if off >= data.len() { break; }
            let (body_size, new_off) = Leb128Decoder::decode_u32(data, off); off = new_off;
            if data.len() < off + body_size as usize { break; }
            if let Some(body) = WasmFuncBodyParser::parse(&data[off..off + body_size as usize]) {
                bodies.push(body);
            }
            off += body_size as usize;
        }
        bodies
    }
}

// ─── Full WASM Module Parser ───────────────────────────────────

#[derive(Debug, Clone)]
pub struct WasmModule {
    pub header: WasmHeader, pub sections: Vec<WasmSection>,
    pub types: Vec<WasmFuncType>, pub exports: Vec<WasmExport>,
    pub func_bodies: Vec<WasmFuncBody>,
}

pub struct WasmModuleParser;

impl WasmModuleParser {
    pub fn parse(data: &[u8]) -> Option<WasmModule> {
        let header = WasmHeader::parse(data)?;
        let sections = WasmSection::parse_all(data);
        let mut types = Vec::new();
        let mut exports = Vec::new();
        let mut func_bodies = Vec::new();
        for sec in &sections {
            match sec.id {
                1 => types = WasmTypeParser::parse(&sec.data),
                7 => exports = WasmExportParser::parse(&sec.data),
                10 => func_bodies = WasmCodeParser::parse(&sec.data),
                _ => {}
            }
        }
        let msg = format!("{}: {} types, {} exports, {} func bodies, version={}",
            &Lang::get(MsgId::CompatWasmInit),
            types.len(), exports.len(), func_bodies.len(), header.version);
        console_writeln(&msg);
        Some(WasmModule { header, sections, types, exports, func_bodies })
    }
}

// ─── WASM Mini Stack Interpreter ───────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum WasmValue {
    I32(i32), I64(i64), F32(u32), F64(u64),
}

#[derive(Debug, Clone)]
pub struct BlockFrame {
    pub return_pc: usize, pub arity: usize, pub is_loop: bool,
}

#[derive(Debug, Clone)]
pub struct WasmFrame {
    pub locals: Vec<WasmValue>, pub stack: Vec<WasmValue>, pub pc: usize, pub blocks: Vec<BlockFrame>,
}

impl WasmFrame {
    pub fn new(local_count: usize) -> Self {
        Self { locals: vec![WasmValue::I32(0); local_count], stack: Vec::new(), pc: 0, blocks: Vec::new() }
    }
    pub fn push_i32(&mut self, v: i32) { self.stack.push(WasmValue::I32(v)); }
    pub fn push_i64(&mut self, v: i64) { self.stack.push(WasmValue::I64(v)); }
    pub fn push_f32(&mut self, v: f32) { self.stack.push(WasmValue::F32(f32::to_bits(v))); }
    pub fn push_f64(&mut self, v: f64) { self.stack.push(WasmValue::F64(f64::to_bits(v))); }
    pub fn pop_i32(&mut self) -> i32 { match self.stack.pop() { Some(WasmValue::I32(v)) => v, _ => 0 } }
    pub fn pop_i64(&mut self) -> i64 { match self.stack.pop() { Some(WasmValue::I64(v)) => v, _ => 0 } }
    pub fn pop_f32(&mut self) -> f32 { match self.stack.pop() { Some(WasmValue::F32(v)) => f32::from_bits(v), _ => 0.0 } }
    pub fn pop_f64(&mut self) -> f64 { match self.stack.pop() { Some(WasmValue::F64(v)) => f64::from_bits(v), _ => 0.0 } }
}

#[derive(Debug, Clone)]
pub struct WasmMemory {
    pub data: Vec<u8>, pub pages: u32,
}

impl WasmMemory {
    pub fn new(initial_pages: u32) -> Self {
        Self { data: vec![0u8; (initial_pages * 65536) as usize], pages: initial_pages }
    }
    pub fn read_i32(&self, addr: usize) -> i32 {
        if addr + 4 <= self.data.len() { i32::from_le_bytes([self.data[addr], self.data[addr+1], self.data[addr+2], self.data[addr+3]]) } else { 0 }
    }
    pub fn read_i64(&self, addr: usize) -> i64 {
        if addr + 8 <= self.data.len() { i64::from_le_bytes([self.data[addr], self.data[addr+1], self.data[addr+2], self.data[addr+3], self.data[addr+4], self.data[addr+5], self.data[addr+6], self.data[addr+7]]) } else { 0 }
    }
    pub fn read_f32(&self, addr: usize) -> f32 {
        if addr + 4 <= self.data.len() { f32::from_le_bytes([self.data[addr], self.data[addr+1], self.data[addr+2], self.data[addr+3]]) } else { 0.0 }
    }
    pub fn read_f64(&self, addr: usize) -> f64 {
        if addr + 8 <= self.data.len() { f64::from_le_bytes([self.data[addr], self.data[addr+1], self.data[addr+2], self.data[addr+3], self.data[addr+4], self.data[addr+5], self.data[addr+6], self.data[addr+7]]) } else { 0.0 }
    }
    pub fn write_i32(&mut self, addr: usize, v: i32) {
        if addr + 4 <= self.data.len() { let b = v.to_le_bytes(); self.data[addr..addr+4].copy_from_slice(&b); }
    }
    pub fn write_i64(&mut self, addr: usize, v: i64) {
        if addr + 8 <= self.data.len() { let b = v.to_le_bytes(); self.data[addr..addr+8].copy_from_slice(&b); }
    }
    pub fn write_f32(&mut self, addr: usize, v: f32) {
        if addr + 4 <= self.data.len() { let b = v.to_le_bytes(); self.data[addr..addr+4].copy_from_slice(&b); }
    }
    pub fn write_f64(&mut self, addr: usize, v: f64) {
        if addr + 8 <= self.data.len() { let b = v.to_le_bytes(); self.data[addr..addr+8].copy_from_slice(&b); }
    }
}

fn f32_ceil(a: f32) -> f32 { let i = a as i32; if a > 0.0 && a != i as f32 { (i + 1) as f32 } else { i as f32 } }
fn f32_floor(a: f32) -> f32 { let i = a as i32; if a < 0.0 && a != i as f32 { (i - 1) as f32 } else { i as f32 } }
fn f32_trunc(a: f32) -> f32 { a as i32 as f32 }
fn f32_sqrt(a: f32) -> f32 { if a <= 0.0 { 0.0 } else { let mut x = a; for _ in 0..20 { x = (x + a / x) * 0.5; } x } }
fn f32_nearest(a: f32) -> f32 { let i = a as i32; let f = a - i as f32; if f >= 0.5 { (i + 1) as f32 } else if f <= -0.5 { (i - 1) as f32 } else { i as f32 } }
fn f64_ceil(a: f64) -> f64 { let i = a as i64; if a > 0.0 && a != i as f64 { (i + 1) as f64 } else { i as f64 } }
fn f64_floor(a: f64) -> f64 { let i = a as i64; if a < 0.0 && a != i as f64 { (i - 1) as f64 } else { i as f64 } }
fn f64_trunc(a: f64) -> f64 { a as i64 as f64 }
fn f64_sqrt(a: f64) -> f64 { if a <= 0.0 { 0.0 } else { let mut x = a; for _ in 0..20 { x = (x + a / x) * 0.5; } x } }
fn f64_nearest(a: f64) -> f64 { let i = a as i64; let f = a - i as f64; if f >= 0.5 { (i + 1) as f64 } else if f <= -0.5 { (i - 1) as f64 } else { i as f64 } }

pub struct WasmInterpreter;

impl WasmInterpreter {
    pub fn execute(body: &WasmFuncBody, args: &[i32], memory: &mut WasmMemory, globals: &mut Vec<WasmValue>) -> Option<i32> {
        let mut local_count: usize = args.len();
        for (c, _) in &body.locals { local_count += *c as usize; }
        let mut frame = WasmFrame::new(local_count);
        for (i, arg) in args.iter().enumerate() { frame.locals[i] = WasmValue::I32(*arg); }
        let code = &body.code;
        while frame.pc < code.len() {
            let op = code[frame.pc];
            frame.pc += 1;
            match op {
                // ─── Control Flow ────────────────────────────────
                0x00 => {}, // nop
                0x01 => { let (_bt, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; frame.blocks.push(BlockFrame { return_pc: 0, arity: 0, is_loop: false }); } // block
                0x02 => { let loop_start = frame.pc; let (_bt, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; frame.blocks.push(BlockFrame { return_pc: loop_start, arity: 0, is_loop: true }); } // loop
                0x03 => { // if
                    let (_bt, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc;
                    let cond = frame.pop_i32();
                    frame.blocks.push(BlockFrame { return_pc: 0, arity: 0, is_loop: false });
                    if cond == 0 {
                        let mut depth = 1;
                        while depth > 0 && frame.pc < code.len() {
                            match code[frame.pc] { 0x01..=0x03 => depth += 1, 0x05 if depth == 1 => { frame.pc += 1; break; }, 0x0B => { depth -= 1; if depth == 0 { break; } }, _ => {} }
                            frame.pc += 1;
                        }
                    }
                }
                0x04 => { // else
                    let mut depth = 1;
                    while depth > 0 && frame.pc < code.len() {
                        match code[frame.pc] { 0x01..=0x03 => depth += 1, 0x0B => { depth -= 1; if depth == 0 { break; } }, _ => {} }
                        frame.pc += 1;
                    }
                }
                0x0B => { // end
                    if let Some(block) = frame.blocks.pop() {
                        if block.is_loop { frame.pc = block.return_pc; }
                    } else { break; }
                }
                0x0C => { // br
                    let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc;
                    let target = frame.blocks.len().saturating_sub(idx as usize + 1);
                    if let Some(block) = frame.blocks.get(target) {
                        if block.is_loop { frame.pc = block.return_pc; frame.blocks.truncate(target + 1); }
                        else {
                            let mut d = 1;
                            while d > 0 && frame.pc < code.len() {
                                match code[frame.pc] { 0x01..=0x03 => d += 1, 0x0B => { d -= 1; if d == 0 { frame.pc += 1; break; } }, _ => {} }
                                frame.pc += 1;
                            }
                            frame.blocks.truncate(target);
                        }
                    }
                }
                0x0D => { // br_if
                    let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc;
                    if frame.pop_i32() != 0 {
                        let target = frame.blocks.len().saturating_sub(idx as usize + 1);
                        if let Some(block) = frame.blocks.get(target) {
                            if block.is_loop { frame.pc = block.return_pc; frame.blocks.truncate(target + 1); }
                            else {
                                let mut d = 1;
                                while d > 0 && frame.pc < code.len() {
                                    match code[frame.pc] { 0x01..=0x03 => d += 1, 0x0B => { d -= 1; if d == 0 { frame.pc += 1; break; } }, _ => {} }
                                    frame.pc += 1;
                                }
                                frame.blocks.truncate(target);
                            }
                        }
                    }
                }
                0x0E => { // br_table
                    let (count, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc;
                    let mut labels = Vec::new();
                    for _ in 0..count { let (l, np) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np; labels.push(l); }
                    let (default, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc;
                    let idx = frame.pop_i32() as u32;
                    let label = if (idx as usize) < labels.len() { labels[idx as usize] } else { default };
                    let target = frame.blocks.len().saturating_sub(label as usize + 1);
                    if let Some(block) = frame.blocks.get(target) {
                        if block.is_loop { frame.pc = block.return_pc; frame.blocks.truncate(target + 1); }
                        else {
                            let mut d = 1;
                            while d > 0 && frame.pc < code.len() {
                                match code[frame.pc] { 0x01..=0x03 => d += 1, 0x0B => { d -= 1; if d == 0 { frame.pc += 1; break; } }, _ => {} }
                                frame.pc += 1;
                            }
                            frame.blocks.truncate(target);
                        }
                    }
                }
                0x0F => { // return
                    break;
                }
                0x10 => { let (_idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; console_writeln(&Lang::get(MsgId::CompatWasmFuncCall)); } // call
                0x11 => { let (_idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; let _table = code[frame.pc]; frame.pc += 1; console_writeln(&Lang::get(MsgId::CompatWasmImport)); } // call_indirect
                // ─── Stack ───────────────────────────────────────
                0x1A => { frame.stack.pop(); } // drop
                0x1B => { let c = frame.pop_i32(); let val2 = frame.stack.pop().unwrap_or(WasmValue::I32(0)); let val1 = frame.stack.pop().unwrap_or(WasmValue::I32(0)); frame.stack.push(if c != 0 { val1 } else { val2 }); } // select
                // ─── Local / Global ──────────────────────────────
                0x20 => { let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; frame.stack.push(frame.locals[idx as usize]); } // local.get
                0x21 => { let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; frame.locals[idx as usize] = frame.stack.pop().unwrap_or(WasmValue::I32(0)); } // local.set
                0x22 => { let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; let v = frame.stack.last().copied().unwrap_or(WasmValue::I32(0)); frame.locals[idx as usize] = v; } // local.tee
                0x23 => { let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; frame.stack.push(globals.get(idx as usize).copied().unwrap_or(WasmValue::I32(0))); } // global.get
                0x24 => { let (idx, new_pc) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = new_pc; let v = frame.stack.pop().unwrap_or(WasmValue::I32(0)); if (idx as usize) < globals.len() { globals[idx as usize] = v; } } // global.set
                // ─── Memory ──────────────────────────────────────
                0x28 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_i32(memory.read_i32(addr)); } // i32.load
                0x29 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_i64(memory.read_i64(addr)); } // i64.load
                0x2A => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_f32(memory.read_f32(addr)); } // f32.load
                0x2B => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_f64(memory.read_f64(addr)); } // f64.load
                0x2C => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_i32(memory.data.get(addr).copied().unwrap_or(0) as i8 as i32); } // i32.load8_s
                0x2D => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_i32(memory.data.get(addr).copied().unwrap_or(0) as i32); } // i32.load8_u
                0x2E => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; let v = if addr + 2 <= memory.data.len() { i16::from_le_bytes([memory.data[addr], memory.data[addr+1]]) } else { 0 }; frame.push_i32(v as i32); } // i32.load16_s
                0x2F => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; let v = if addr + 2 <= memory.data.len() { u16::from_le_bytes([memory.data[addr], memory.data[addr+1]]) } else { 0 }; frame.push_i32(v as i32); } // i32.load16_u
                0x30 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_i64(memory.data.get(addr).copied().unwrap_or(0) as i8 as i64); } // i64.load8_s
                0x31 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; frame.push_i64(memory.data.get(addr).copied().unwrap_or(0) as i64); } // i64.load8_u
                0x32 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; let v = if addr + 2 <= memory.data.len() { i16::from_le_bytes([memory.data[addr], memory.data[addr+1]]) } else { 0 }; frame.push_i64(v as i64); } // i64.load16_s
                0x33 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; let v = if addr + 2 <= memory.data.len() { u16::from_le_bytes([memory.data[addr], memory.data[addr+1]]) } else { 0 }; frame.push_i64(v as i64); } // i64.load16_u
                0x34 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; let v = if addr + 4 <= memory.data.len() { i32::from_le_bytes([memory.data[addr], memory.data[addr+1], memory.data[addr+2], memory.data[addr+3]]) } else { 0 }; frame.push_i64(v as i64); } // i64.load32_s
                0x35 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let addr = frame.pop_i32() as usize + offset as usize; let v = if addr + 4 <= memory.data.len() { u32::from_le_bytes([memory.data[addr], memory.data[addr+1], memory.data[addr+2], memory.data[addr+3]]) } else { 0 }; frame.push_i64(v as i64); } // i64.load32_u
                0x36 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i32(); let addr = frame.pop_i32() as usize + offset as usize; memory.write_i32(addr, val); } // i32.store
                0x37 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i64(); let addr = frame.pop_i32() as usize + offset as usize; memory.write_i64(addr, val); } // i64.store
                0x38 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_f32(); let addr = frame.pop_i32() as usize + offset as usize; memory.write_f32(addr, val); } // f32.store
                0x39 => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_f64(); let addr = frame.pop_i32() as usize + offset as usize; memory.write_f64(addr, val); } // f64.store
                0x3A => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i32() as i8; let addr = frame.pop_i32() as usize + offset as usize; if addr < memory.data.len() { memory.data[addr] = val as u8; } } // i32.store8
                0x3B => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i32() as i16; let addr = frame.pop_i32() as usize + offset as usize; if addr + 2 <= memory.data.len() { let b = val.to_le_bytes(); memory.data[addr..addr+2].copy_from_slice(&b); } } // i32.store16
                0x3C => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i64() as i8; let addr = frame.pop_i32() as usize + offset as usize; if addr < memory.data.len() { memory.data[addr] = val as u8; } } // i64.store8
                0x3D => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i64() as i16; let addr = frame.pop_i32() as usize + offset as usize; if addr + 2 <= memory.data.len() { let b = val.to_le_bytes(); memory.data[addr..addr+2].copy_from_slice(&b); } } // i64.store16
                0x3E => { let (_align, np1) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np1; let (offset, np2) = Leb128Decoder::decode_u32(code, frame.pc); frame.pc = np2; let val = frame.pop_i64() as i32; let addr = frame.pop_i32() as usize + offset as usize; if addr + 4 <= memory.data.len() { let b = val.to_le_bytes(); memory.data[addr..addr+4].copy_from_slice(&b); } } // i64.store32
                0x3F => { frame.pc += 1; frame.push_i32(memory.pages as i32); } // memory.size
                0x40 => { frame.pc += 1; let delta = frame.pop_i32() as u32; let old = memory.pages; memory.data.resize(((old + delta) * 65536) as usize, 0u8); memory.pages += delta; frame.push_i32(old as i32); } // memory.grow
                // ─── Constants ───────────────────────────────────
                0x41 => { let (v, new_pc) = Leb128Decoder::decode_i32(code, frame.pc); frame.pc = new_pc; frame.push_i32(v); } // i32.const
                0x42 => { let (v, new_pc) = Leb128Decoder::decode_i64(code, frame.pc); frame.pc = new_pc; frame.push_i64(v); } // i64.const
                0x43 => { let v = f32::from_le_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3]]); frame.pc += 4; frame.push_f32(v); } // f32.const
                0x44 => { let v = f64::from_le_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3], code[frame.pc+4], code[frame.pc+5], code[frame.pc+6], code[frame.pc+7]]); frame.pc += 8; frame.push_f64(v); } // f64.const
                // ─── i32 Comparison ──────────────────────────────
                0x45 => { let a = frame.pop_i32(); frame.push_i32(if a == 0 { 1 } else { 0 }); } // i32.eqz
                0x46 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if a == b { 1 } else { 0 }); } // i32.eq
                0x47 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if a != b { 1 } else { 0 }); } // i32.ne
                0x48 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if a < b { 1 } else { 0 }); } // i32.lt_s
                0x49 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if (a as u32) < (b as u32) { 1 } else { 0 }); } // i32.lt_u
                0x4A => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if a > b { 1 } else { 0 }); } // i32.gt_s
                0x4B => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if (a as u32) > (b as u32) { 1 } else { 0 }); } // i32.gt_u
                0x4C => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if a <= b { 1 } else { 0 }); } // i32.le_s
                0x4D => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if (a as u32) <= (b as u32) { 1 } else { 0 }); } // i32.le_u
                0x4E => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if a >= b { 1 } else { 0 }); } // i32.ge_s
                0x4F => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(if (a as u32) >= (b as u32) { 1 } else { 0 }); } // i32.ge_u
                // ─── i64 Comparison ──────────────────────────────
                0x50 => { let a = frame.pop_i64(); frame.push_i32(if a == 0 { 1 } else { 0 }); } // i64.eqz
                0x51 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if a == b { 1 } else { 0 }); } // i64.eq
                0x52 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if a != b { 1 } else { 0 }); } // i64.ne
                0x53 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if a < b { 1 } else { 0 }); } // i64.lt_s
                0x54 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if (a as u64) < (b as u64) { 1 } else { 0 }); } // i64.lt_u
                0x55 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if a > b { 1 } else { 0 }); } // i64.gt_s
                0x56 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if (a as u64) > (b as u64) { 1 } else { 0 }); } // i64.gt_u
                0x57 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if a <= b { 1 } else { 0 }); } // i64.le_s
                0x58 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if (a as u64) <= (b as u64) { 1 } else { 0 }); } // i64.le_u
                0x59 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if a >= b { 1 } else { 0 }); } // i64.ge_s
                0x5A => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i32(if (a as u64) >= (b as u64) { 1 } else { 0 }); } // i64.ge_u
                // ─── f32 Comparison ──────────────────────────────
                0x5B => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_i32(if a == b { 1 } else { 0 }); } // f32.eq
                0x5C => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_i32(if a != b { 1 } else { 0 }); } // f32.ne
                0x5D => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_i32(if a < b { 1 } else { 0 }); } // f32.lt
                0x5E => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_i32(if a > b { 1 } else { 0 }); } // f32.gt
                0x5F => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_i32(if a <= b { 1 } else { 0 }); } // f32.le
                0x60 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_i32(if a >= b { 1 } else { 0 }); } // f32.ge
                // ─── f64 Comparison ──────────────────────────────
                0x61 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_i32(if a == b { 1 } else { 0 }); } // f64.eq
                0x62 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_i32(if a != b { 1 } else { 0 }); } // f64.ne
                0x63 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_i32(if a < b { 1 } else { 0 }); } // f64.lt
                0x64 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_i32(if a > b { 1 } else { 0 }); } // f64.gt
                0x65 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_i32(if a <= b { 1 } else { 0 }); } // f64.le
                0x66 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_i32(if a >= b { 1 } else { 0 }); } // f64.ge
                // ─── i32 Numeric ─────────────────────────────────
                0x67 => { let a = frame.pop_i32(); frame.push_i32(a.leading_zeros() as i32); } // i32.clz
                0x68 => { let a = frame.pop_i32(); frame.push_i32(a.trailing_zeros() as i32); } // i32.ctz
                0x69 => { let a = frame.pop_i32(); frame.push_i32(a.count_ones() as i32); } // i32.popcnt
                0x6A => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(a.wrapping_add(b)); } // i32.add
                0x6B => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(a.wrapping_sub(b)); } // i32.sub
                0x6C => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(a.wrapping_mul(b)); } // i32.mul
                0x6D => { let b = frame.pop_i32(); let a = frame.pop_i32(); if b != 0 { frame.push_i32(a / b) } else { frame.push_i32(0); } } // i32.div_s
                0x6E => { let b = frame.pop_i32(); let a = frame.pop_i32(); if b != 0 { frame.push_i32((a as u32 / b as u32) as i32) } else { frame.push_i32(0); } } // i32.div_u
                0x6F => { let b = frame.pop_i32(); let a = frame.pop_i32(); if b != 0 { frame.push_i32(a % b) } else { frame.push_i32(0); } } // i32.rem_s
                0x70 => { let b = frame.pop_i32(); let a = frame.pop_i32(); if b != 0 { frame.push_i32((a as u32 % b as u32) as i32) } else { frame.push_i32(0); } } // i32.rem_u
                0x71 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(a & b); } // i32.and
                0x72 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(a | b); } // i32.or
                0x73 => { let b = frame.pop_i32(); let a = frame.pop_i32(); frame.push_i32(a ^ b); } // i32.xor
                0x74 => { let b = frame.pop_i32() & 0x1F; let a = frame.pop_i32(); frame.push_i32(a << b); } // i32.shl
                0x75 => { let b = frame.pop_i32() & 0x1F; let a = frame.pop_i32(); frame.push_i32(a >> b); } // i32.shr_s
                0x76 => { let b = frame.pop_i32() & 0x1F; let a = frame.pop_i32(); frame.push_i32((a as u32 >> b) as i32); } // i32.shr_u
                0x77 => { let b = frame.pop_i32() & 0x1F; let a = frame.pop_i32(); frame.push_i32(a.rotate_left(b as u32)); } // i32.rotl
                0x78 => { let b = frame.pop_i32() & 0x1F; let a = frame.pop_i32(); frame.push_i32(a.rotate_right(b as u32)); } // i32.rotr
                // ─── i64 Numeric ─────────────────────────────────
                0x79 => { let a = frame.pop_i64(); frame.push_i64(a.leading_zeros() as i64); } // i64.clz
                0x7A => { let a = frame.pop_i64(); frame.push_i64(a.trailing_zeros() as i64); } // i64.ctz
                0x7B => { let a = frame.pop_i64(); frame.push_i64(a.count_ones() as i64); } // i64.popcnt
                0x7C => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i64(a.wrapping_add(b)); } // i64.add
                0x7D => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i64(a.wrapping_sub(b)); } // i64.sub
                0x7E => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i64(a.wrapping_mul(b)); } // i64.mul
                0x7F => { let b = frame.pop_i64(); let a = frame.pop_i64(); if b != 0 { frame.push_i64(a / b) } else { frame.push_i64(0); } } // i64.div_s
                0x80 => { let b = frame.pop_i64(); let a = frame.pop_i64(); if b != 0 { frame.push_i64((a as u64 / b as u64) as i64) } else { frame.push_i64(0); } } // i64.div_u
                0x81 => { let b = frame.pop_i64(); let a = frame.pop_i64(); if b != 0 { frame.push_i64(a % b) } else { frame.push_i64(0); } } // i64.rem_s
                0x82 => { let b = frame.pop_i64(); let a = frame.pop_i64(); if b != 0 { frame.push_i64((a as u64 % b as u64) as i64) } else { frame.push_i64(0); } } // i64.rem_u
                0x83 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i64(a & b); } // i64.and
                0x84 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i64(a | b); } // i64.or
                0x85 => { let b = frame.pop_i64(); let a = frame.pop_i64(); frame.push_i64(a ^ b); } // i64.xor
                0x86 => { let b = frame.pop_i64() & 0x3F; let a = frame.pop_i64(); frame.push_i64(a << b); } // i64.shl
                0x87 => { let b = frame.pop_i64() & 0x3F; let a = frame.pop_i64(); frame.push_i64(a >> b); } // i64.shr_s
                0x88 => { let b = frame.pop_i64() & 0x3F; let a = frame.pop_i64(); frame.push_i64((a as u64 >> b) as i64); } // i64.shr_u
                0x89 => { let b = frame.pop_i64() & 0x3F; let a = frame.pop_i64(); frame.push_i64(a.rotate_left(b as u32)); } // i64.rotl
                0x8A => { let b = frame.pop_i64() & 0x3F; let a = frame.pop_i64(); frame.push_i64(a.rotate_right(b as u32)); } // i64.rotr
                // ─── f32 Numeric ─────────────────────────────────
                0x8B => { let a = frame.pop_f32(); frame.push_f32(if a < 0.0 { -a } else { a }); } // f32.abs
                0x8C => { let a = frame.pop_f32(); frame.push_f32(-a); } // f32.neg
                0x8D => { let a = frame.pop_f32(); frame.push_f32(f32_ceil(a)); } // f32.ceil
                0x8E => { let a = frame.pop_f32(); frame.push_f32(f32_floor(a)); } // f32.floor
                0x8F => { let a = frame.pop_f32(); frame.push_f32(f32_trunc(a)); } // f32.trunc
                0x90 => { let a = frame.pop_f32(); frame.push_f32(f32_nearest(a)); } // f32.nearest
                0x91 => { let a = frame.pop_f32(); frame.push_f32(f32_sqrt(a)); } // f32.sqrt
                0x92 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(a + b); } // f32.add
                0x93 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(a - b); } // f32.sub
                0x94 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(a * b); } // f32.mul
                0x95 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(if b != 0.0 { a / b } else { 0.0 }); } // f32.div
                0x96 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(if a < b { a } else { b }); } // f32.min
                0x97 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(if a > b { a } else { b }); } // f32.max
                0x98 => { let b = frame.pop_f32(); let a = frame.pop_f32(); frame.push_f32(if a.is_sign_negative() { -b.abs() } else { b.abs() }); } // f32.copysign
                // ─── f64 Numeric ─────────────────────────────────
                0x99 => { let a = frame.pop_f64(); frame.push_f64(if a < 0.0 { -a } else { a }); } // f64.abs
                0x9A => { let a = frame.pop_f64(); frame.push_f64(-a); } // f64.neg
                0x9B => { let a = frame.pop_f64(); frame.push_f64(f64_ceil(a)); } // f64.ceil
                0x9C => { let a = frame.pop_f64(); frame.push_f64(f64_floor(a)); } // f64.floor
                0x9D => { let a = frame.pop_f64(); frame.push_f64(f64_trunc(a)); } // f64.trunc
                0x9E => { let a = frame.pop_f64(); frame.push_f64(f64_nearest(a)); } // f64.nearest
                0x9F => { let a = frame.pop_f64(); frame.push_f64(f64_sqrt(a)); } // f64.sqrt
                0xA0 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(a + b); } // f64.add
                0xA1 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(a - b); } // f64.sub
                0xA2 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(a * b); } // f64.mul
                0xA3 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(if b != 0.0 { a / b } else { 0.0 }); } // f64.div
                0xA4 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(if a < b { a } else { b }); } // f64.min
                0xA5 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(if a > b { a } else { b }); } // f64.max
                0xA6 => { let b = frame.pop_f64(); let a = frame.pop_f64(); frame.push_f64(if a.is_sign_negative() { -b.abs() } else { b.abs() }); } // f64.copysign
                // ─── Conversions ─────────────────────────────────
                0xA7 => { let a = frame.pop_i64(); frame.push_i32(a as i32); } // i32.wrap_i64
                0xA8 => { let a = frame.pop_f32(); frame.push_i32(a as i32); } // i32.trunc_f32_s
                0xA9 => { let a = frame.pop_f32(); frame.push_i32(a as u32 as i32); } // i32.trunc_f32_u
                0xAA => { let a = frame.pop_f64(); frame.push_i32(a as i32); } // i32.trunc_f64_s
                0xAB => { let a = frame.pop_f64(); frame.push_i32(a as u32 as i32); } // i32.trunc_f64_u
                0xAC => { let a = frame.pop_i32(); frame.push_i64(a as i64); } // i64.extend_i32_s
                0xAD => { let a = frame.pop_i32(); frame.push_i64((a as u32) as i64); } // i64.extend_i32_u
                0xAE => { let a = frame.pop_f32(); frame.push_i64(a as i64); } // i64.trunc_f32_s
                0xAF => { let a = frame.pop_f32(); frame.push_i64(a as u64 as i64); } // i64.trunc_f32_u
                0xB0 => { let a = frame.pop_f64(); frame.push_i64(a as i64); } // i64.trunc_f64_s
                0xB1 => { let a = frame.pop_f64(); frame.push_i64(a as u64 as i64); } // i64.trunc_f64_u
                0xB2 => { let a = frame.pop_i32(); frame.push_f32(a as f32); } // f32.convert_i32_s
                0xB3 => { let a = frame.pop_i32(); frame.push_f32((a as u32) as f32); } // f32.convert_i32_u
                0xB4 => { let a = frame.pop_i64(); frame.push_f32(a as f32); } // f32.convert_i64_s
                0xB5 => { let a = frame.pop_i64(); frame.push_f32((a as u64) as f32); } // f32.convert_i64_u
                0xB6 => { let a = frame.pop_f64(); frame.push_f32(a as f32); } // f32.demote_f64
                0xB7 => { let a = frame.pop_i32(); frame.push_f64(a as f64); } // f64.convert_i32_s
                0xB8 => { let a = frame.pop_i32(); frame.push_f64((a as u32) as f64); } // f64.convert_i32_u
                0xB9 => { let a = frame.pop_i64(); frame.push_f64(a as f64); } // f64.convert_i64_s
                0xBA => { let a = frame.pop_i64(); frame.push_f64((a as u64) as f64); } // f64.convert_i64_u
                0xBB => { let a = frame.pop_f32(); frame.push_f64(a as f64); } // f64.promote_f32
                0xBC => { let a = frame.pop_f32(); frame.push_i32(f32::to_bits(a) as i32); } // i32.reinterpret_f32
                0xBD => { let a = frame.pop_f64(); frame.push_i64(f64::to_bits(a) as i64); } // i64.reinterpret_f64
                0xBE => { let a = frame.pop_i32(); frame.push_f32(f32::from_bits(a as u32)); } // f32.reinterpret_i32
                0xBF => { let a = frame.pop_i64(); frame.push_f64(f64::from_bits(a as u64)); } // f64.reinterpret_i64
                _ => {
                    let msg = format!("{}: 0x{:02X} @ pc={}",
                        &Lang::get(MsgId::CompatWasmTrap), op, frame.pc - 1);
                    console_writeln(&msg); return None;
                }
            }
        }
        if let Some(WasmValue::I32(v)) = frame.stack.last() { Some(*v) } else { Some(0) }
    }
}

// ─── WASM Runtime ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WasmRuntime {
    pub modules: Vec<WasmModule>, pub memory: WasmMemory, pub globals: Vec<WasmValue>,
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmRuntime {
    pub fn new() -> Self { Self { modules: Vec::new(), memory: WasmMemory::new(1), globals: Vec::new() } }
    #[must_use]
    pub fn load(&mut self, data: &[u8]) -> Option<String> {
        let module = WasmModuleParser::parse(data)?;
        let name = module.exports.first().map(|e| e.name.clone()).unwrap_or_else(|| String::from("unknown"));
        let msg = format!("{}: {} ({} funcs)",
            &Lang::get(MsgId::CompatWasmModuleLoad), name, module.func_bodies.len());
        console_writeln(&msg);
        self.modules.push(module); Some(name)
    }
    #[must_use]
    pub fn invoke(&mut self, module_idx: usize, func_idx: usize, args: &[i32]) -> Option<i32> {
        let module = self.modules.get(module_idx)?;
        let body = module.func_bodies.get(func_idx)?;
        let msg = format!("{}: module[{}].func[{}]({:?})",
            &Lang::get(MsgId::CompatWasmFuncCall), module_idx, func_idx, args);
        console_writeln(&msg);
        WasmInterpreter::execute(body, args, &mut self.memory, &mut self.globals)
    }
    pub fn dump(&self) {
        let msg = format!("{} ({}):", &Lang::get(MsgId::CompatWasmExport), self.modules.len());
        console_writeln(&msg);
        for (i, m) in self.modules.iter().enumerate() {
            let msg = format!("  [{}] {} exports, {} funcs", i, m.exports.len(), m.func_bodies.len());
            console_writeln(&msg);
            for e in &m.exports { let msg = format!("    export: {} (kind={}, idx={})", e.name, e.kind, e.index); console_writeln(&msg); }
        }
    }
}

// ─── Unit Tests ────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── LEB128 ────────────────────────────────────────────────

    #[test]
    fn test_leb128_u32_single_byte() {
        let (v, off) = Leb128Decoder::decode_u32(&[0x01], 0);
        assert_eq!(v, 1);
        assert_eq!(off, 1);
    }

    #[test]
    fn test_leb128_u32_multibyte() {
        // 300 = 0xAC 0x02
        let (v, off) = Leb128Decoder::decode_u32(&[0xAC, 0x02], 0);
        assert_eq!(v, 300);
        assert_eq!(off, 2);
    }

    #[test]
    fn test_leb128_i32_negative() {
        // -1 = 0x7F in signed LEB128
        let (v, _) = Leb128Decoder::decode_i32(&[0x7F], 0);
        assert_eq!(v, -1);
    }

    #[test]
    fn test_leb128_u64_multibyte() {
        let (v, _) = Leb128Decoder::decode_u64(&[0xAC, 0x02], 0);
        assert_eq!(v, 300);
    }

    #[test]
    fn test_leb128_empty_slice() {
        let (v, off) = Leb128Decoder::decode_u32(&[], 0);
        assert_eq!(v, 0);
        assert_eq!(off, 0);
    }

    // ─── WasmHeader ────────────────────────────────────────────

    #[test]
    fn test_wasm_header_valid() {
        // \0asm version 1
        let data = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
        let hdr = WasmHeader::parse(&data).expect("valid header");
        assert_eq!(hdr.magic, WasmHeader::WASM_MAGIC);
        assert_eq!(hdr.version, WasmHeader::WASM_VERSION);
    }

    #[test]
    fn test_wasm_header_invalid_magic() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x00, 0x00, 0x00];
        assert!(WasmHeader::parse(&data).is_none());
    }

    #[test]
    fn test_wasm_header_too_short() {
        assert!(WasmHeader::parse(&[0x00, 0x61]).is_none());
    }

    // ─── WasmValType ───────────────────────────────────────────

    #[test]
    fn test_wasm_val_type_from_byte() {
        assert!(matches!(WasmValType::from_byte(0x7F), Some(WasmValType::I32)));
        assert!(matches!(WasmValType::from_byte(0x7E), Some(WasmValType::I64)));
        assert!(matches!(WasmValType::from_byte(0x7D), Some(WasmValType::F32)));
        assert!(matches!(WasmValType::from_byte(0x7C), Some(WasmValType::F64)));
        assert!(WasmValType::from_byte(0x00).is_none());
    }

    #[test]
    fn test_wasm_val_type_name() {
        assert_eq!(WasmValType::I32.name(), "i32");
        assert_eq!(WasmValType::I64.name(), "i64");
        assert_eq!(WasmValType::F32.name(), "f32");
        assert_eq!(WasmValType::F64.name(), "f64");
        assert_eq!(WasmValType::Void.name(), "void");
    }

    // ─── WasmMemory ────────────────────────────────────────────

    #[test]
    fn test_wasm_memory_read_write_i32() {
        let mut mem = WasmMemory::new(1);
        mem.write_i32(0, 0xDEAD_BEEF_u32 as i32);
        assert_eq!(mem.read_i32(0), 0xDEAD_BEEF_u32 as i32);
    }

    #[test]
    fn test_wasm_memory_read_write_i64() {
        let mut mem = WasmMemory::new(1);
        mem.write_i64(8, 0x0102_0304_0506_0708_i64);
        assert_eq!(mem.read_i64(8), 0x0102_0304_0506_0708_i64);
    }

    #[test]
    fn test_wasm_memory_out_of_bounds() {
        let mem = WasmMemory::new(1);
        // 1 page = 65536 bytes; read beyond should return 0
        assert_eq!(mem.read_i32(65536), 0);
    }

    // ─── WasmFrame ─────────────────────────────────────────────

    #[test]
    fn test_wasm_frame_push_pop_i32() {
        let mut f = WasmFrame::new(0);
        f.push_i32(42);
        assert_eq!(f.pop_i32(), 42);
    }

    #[test]
    fn test_wasm_frame_pop_empty_returns_zero() {
        let mut f = WasmFrame::new(0);
        assert_eq!(f.pop_i32(), 0);
        assert_eq!(f.pop_i64(), 0);
    }

    // ─── WasmInterpreter ───────────────────────────────────────

    fn make_memory() -> WasmMemory { WasmMemory::new(1) }
    fn make_globals() -> Vec<WasmValue> { Vec::new() }

    fn make_body(code: &[u8]) -> WasmFuncBody {
        WasmFuncBody { locals: Vec::new(), code: code.to_vec() }
    }

    #[test]
    fn test_interp_i32_const_return() {
        // i32.const 7, return
        let body = make_body(&[0x41, 0x07, 0x0F]);
        let mut mem = make_memory(); let mut globals = make_globals();
        let result = WasmInterpreter::execute(&body, &[], &mut mem, &mut globals);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn test_interp_i32_add() {
        // i32.const 10, i32.const 32, i32.add, return
        let body = make_body(&[0x41, 0x0A, 0x41, 0x20, 0x6A, 0x0F]);
        let mut mem = make_memory(); let mut globals = make_globals();
        let result = WasmInterpreter::execute(&body, &[], &mut mem, &mut globals);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_interp_i32_mul() {
        // i32.const 6, i32.const 7, i32.mul, return
        let body = make_body(&[0x41, 0x06, 0x41, 0x07, 0x6C, 0x0F]);
        let mut mem = make_memory(); let mut globals = make_globals();
        let result = WasmInterpreter::execute(&body, &[], &mut mem, &mut globals);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_interp_local_get() {
        // arg=5, local.get 0, return → should return 5
        let body = make_body(&[0x20, 0x00, 0x0F]);
        let mut mem = make_memory(); let mut globals = make_globals();
        let result = WasmInterpreter::execute(&body, &[5], &mut mem, &mut globals);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_interp_unknown_opcode_returns_none() {
        // opcode 0xFF is not defined → returns None
        let body = make_body(&[0xFF]);
        let mut mem = make_memory(); let mut globals = make_globals();
        let result = WasmInterpreter::execute(&body, &[], &mut mem, &mut globals);
        assert!(result.is_none());
    }

    // ─── WasmRuntime ───────────────────────────────────────────

    #[test]
    fn test_runtime_load_invalid() {
        let mut rt = WasmRuntime::new();
        assert!(rt.load(&[0xDE, 0xAD]).is_none());
    }

    #[test]
    fn test_runtime_invoke_out_of_bounds() {
        let mut rt = WasmRuntime::new();
        assert!(rt.invoke(0, 0, &[]).is_none());
    }
}
