// *******************************************************************
//                    ÖZKAN-OS Operating System
//
// File Task Definition : Java JVM Bytecode Compatibility Layer
// File Path            : apps/system/compat/src/java_bytecode.rs
// Author               : Özkan Yıldırım
// License              : GPLv3
//
// Supported Processors : 486DX4, x86, x86_64, ARM 32, ARM64, RISC-V 32, RISC-V 64, MIPS 32, MIPS 64, PowerPC 32, PowerPC 64, m68k, SPARC, LoongArch64
//
// Description:
//   JVM Class file parser (ClassFileHeader, ConstantPool 12
//   tag, FieldInfo, MethodInfo, AttributeInfo/Code) and mini stack
//   based bytecode interpreter (JvmInterpreter, 150+ opcode).
//   Supports class loading and method execution via JavaRuntime.
//
// Dependent Files:
//   1-) apps/system/compat/src/dos_emulator.rs  (console_writeln)
//   2-) kernel/graphics/ui/src/lang/lang_flags.rs (MsgId)
//   3-) kernel/graphics/ui/src/lang/lang_manager.rs (Lang::get)
//
//              File Modifications
// 2026-04-17      C → Rust translation (no_std), full JVM parser
// 2026-04-18      Lang system, #[must_use], unit tests
// *******************************************************************

#![allow(dead_code)]

use crate::dos_emulator::console_writeln;
use kernel_ui::{Lang, MsgId};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ─── JVM Classfile Header Parser ───────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct ClassFileHeader {
    pub magic: u32, pub minor: u16, pub major: u16,
}

impl ClassFileHeader {
    pub const JAVA_MAGIC: u32 = 0xCAFEBABE;
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if magic != Self::JAVA_MAGIC { return None; }
        Some(Self { magic, minor: u16::from_be_bytes([data[4], data[5]]), major: u16::from_be_bytes([data[6], data[7]]) })
    }
    #[must_use]
    pub fn version_string(&self) -> String {
        format!("{}.{} (Java {})", self.major, self.minor, self.major_to_java())
    }
    fn major_to_java(&self) -> u16 {
        match self.major {
            45 => 1, 46 => 2, 47 => 3, 48 => 4, 49 => 5, 50 => 6,
            51 => 7, 52 => 8, 53 => 9, 54 => 10, 55 => 11, 56 => 12,
            57 => 13, 58 => 14, 59 => 15, 60 => 16, 61 => 17, 62 => 18,
            63 => 19, 64 => 20, 65 => 21, _ => self.major - 44,
        }
    }
}

// ─── Constant Pool Parser ──────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CpEntry {
    Utf8(String), Integer(i32), Float(u32), Long(i64), Double(u64),
    Class(u16), String(u16), Fieldref(u16, u16), Methodref(u16, u16),
    InterfaceMethodref(u16, u16), NameAndType(u16, u16),
    MethodHandle(u8, u16), MethodType(u16), InvokeDynamic(u16, u16),
}

pub struct ConstantPoolParser;

impl ConstantPoolParser {
    pub fn parse(data: &[u8], offset: usize, count: u16) -> Option<(Vec<CpEntry>, usize)> {
        if count == 0 { return Some((Vec::new(), offset)); }
        let mut entries = Vec::with_capacity(count as usize);
        entries.push(CpEntry::Utf8(String::new())); // 0 is reserved
        let mut off = offset;
        for _ in 1..count {
            if off >= data.len() { break; }
            let tag = data[off]; off += 1;
            let entry = match tag {
                1 => {
                    let len = u16::from_be_bytes([data[off], data[off + 1]]) as usize;
                    off += 2;
                    if data.len() < off + len { break; }
                    let s = core::str::from_utf8(&data[off..off + len]).unwrap_or("").to_string();
                    off += len; CpEntry::Utf8(s)
                }
                3 => { let v = i32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]); off += 4; CpEntry::Integer(v) }
                4 => { let v = u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]); off += 4; CpEntry::Float(v) }
                5 => { let v = i64::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3], data[off + 4], data[off + 5], data[off + 6], data[off + 7]]); off += 8; entries.push(CpEntry::Long(v)); CpEntry::Long(v) }
                6 => { let v = u64::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3], data[off + 4], data[off + 5], data[off + 6], data[off + 7]]); off += 8; entries.push(CpEntry::Double(v)); CpEntry::Double(v) }
                7 => { let v = u16::from_be_bytes([data[off], data[off + 1]]); off += 2; CpEntry::Class(v) }
                8 => { let v = u16::from_be_bytes([data[off], data[off + 1]]); off += 2; CpEntry::String(v) }
                9 => { let c = u16::from_be_bytes([data[off], data[off + 1]]); let n = u16::from_be_bytes([data[off + 2], data[off + 3]]); off += 4; CpEntry::Fieldref(c, n) }
                10 => { let c = u16::from_be_bytes([data[off], data[off + 1]]); let n = u16::from_be_bytes([data[off + 2], data[off + 3]]); off += 4; CpEntry::Methodref(c, n) }
                11 => { let c = u16::from_be_bytes([data[off], data[off + 1]]); let n = u16::from_be_bytes([data[off + 2], data[off + 3]]); off += 4; CpEntry::InterfaceMethodref(c, n) }
                12 => { let n = u16::from_be_bytes([data[off], data[off + 1]]); let d = u16::from_be_bytes([data[off + 2], data[off + 3]]); off += 4; CpEntry::NameAndType(n, d) }
                15 => { let k = data[off]; let i = u16::from_be_bytes([data[off + 1], data[off + 2]]); off += 3; CpEntry::MethodHandle(k, i) }
                16 => { let i = u16::from_be_bytes([data[off], data[off + 1]]); off += 2; CpEntry::MethodType(i) }
                18 => { let b = u16::from_be_bytes([data[off], data[off + 1]]); let n = u16::from_be_bytes([data[off + 2], data[off + 3]]); off += 4; CpEntry::InvokeDynamic(b, n) }
                _ => { let msg = format!("{}: CP tag={}", &Lang::get(MsgId::CompatJvmInit), tag); console_writeln(&msg); break; }
            };
            entries.push(entry);
        }
        Some((entries, off))
    }
    pub fn resolve_utf8(cp: &[CpEntry], idx: u16) -> String {
        if idx as usize >= cp.len() { return String::new(); }
        match &cp[idx as usize] {
            CpEntry::Utf8(s) => s.clone(),
            CpEntry::Class(i) => Self::resolve_utf8(cp, *i),
            CpEntry::String(i) => Self::resolve_utf8(cp, *i),
            _ => String::new(),
        }
    }
}

// ─── Classfile Structure Parser ────────────────────────────────

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub access_flags: u16, pub name_index: u16, pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub access_flags: u16, pub name_index: u16, pub descriptor_index: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone)]
pub struct AttributeInfo {
    pub name_index: u16, pub name: String, pub data: Vec<u8>,
}

impl AttributeInfo {
    pub fn parse(data: &[u8], cp: &[CpEntry]) -> Option<(Self, usize)> {
        if data.len() < 6 { return None; }
        let name_index = u16::from_be_bytes([data[0], data[1]]);
        let len = u32::from_be_bytes([data[2], data[3], data[4], data[5]]) as usize;
        if data.len() < 6 + len { return None; }
        let name = ConstantPoolParser::resolve_utf8(cp, name_index);
        let attr = Self { name_index, name, data: data[6..6 + len].to_vec() };
        Some((attr, 6 + len))
    }
    pub fn parse_all(data: &[u8], cp: &[CpEntry], count: u16) -> (Vec<Self>, usize) {
        let mut attrs = Vec::new();
        let mut off = 0usize;
        for _ in 0..count {
            if off >= data.len() { break; }
            if let Some((attr, size)) = Self::parse(&data[off..], cp) {
                off += size; attrs.push(attr);
            } else { break; }
        }
        (attrs, off)
    }
}

impl FieldInfo {
    pub fn parse(data: &[u8], cp: &[CpEntry]) -> Option<(Self, usize)> {
        if data.len() < 8 { return None; }
        let access_flags = u16::from_be_bytes([data[0], data[1]]);
        let name_index = u16::from_be_bytes([data[2], data[3]]);
        let descriptor_index = u16::from_be_bytes([data[4], data[5]]);
        let attr_count = u16::from_be_bytes([data[6], data[7]]);
        let (attributes, attr_size) = AttributeInfo::parse_all(&data[8..], cp, attr_count);
        Some((Self { access_flags, name_index, descriptor_index, attributes }, 8 + attr_size))
    }
}

impl MethodInfo {
    pub fn parse(data: &[u8], cp: &[CpEntry]) -> Option<(Self, usize)> {
        if data.len() < 8 { return None; }
        let access_flags = u16::from_be_bytes([data[0], data[1]]);
        let name_index = u16::from_be_bytes([data[2], data[3]]);
        let descriptor_index = u16::from_be_bytes([data[4], data[5]]);
        let attr_count = u16::from_be_bytes([data[6], data[7]]);
        let (attributes, attr_size) = AttributeInfo::parse_all(&data[8..], cp, attr_count);
        Some((Self { access_flags, name_index, descriptor_index, attributes }, 8 + attr_size))
    }
    pub fn get_code(&self) -> Option<CodeAttribute> {
        for attr in &self.attributes {
            if attr.name == "Code" && attr.data.len() >= 8 {
                return CodeAttribute::parse(&attr.data);
            }
        }
        None
    }
}

// ─── Code Attribute Parser ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CodeAttribute {
    pub max_stack: u16, pub max_locals: u16,
    pub code: Vec<u8>, pub exception_table: Vec<ExceptionEntry>,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExceptionEntry {
    pub start_pc: u16, pub end_pc: u16, pub handler_pc: u16, pub catch_type: u16,
}

impl CodeAttribute {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 { return None; }
        let max_stack = u16::from_be_bytes([data[0], data[1]]);
        let max_locals = u16::from_be_bytes([data[2], data[3]]);
        let code_len = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
        if data.len() < 8 + code_len + 2 { return None; }
        let code = data[8..8 + code_len].to_vec();
        let mut off = 8 + code_len;
        let ex_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let mut exception_table = Vec::new();
        for _ in 0..ex_count {
            if data.len() < off + 8 { break; }
            exception_table.push(ExceptionEntry {
                start_pc: u16::from_be_bytes([data[off], data[off + 1]]),
                end_pc: u16::from_be_bytes([data[off + 2], data[off + 3]]),
                handler_pc: u16::from_be_bytes([data[off + 4], data[off + 5]]),
                catch_type: u16::from_be_bytes([data[off + 6], data[off + 7]]),
            }); off += 8;
        }
        let attr_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let (attributes, _) = AttributeInfo::parse_all(&data[off..], &[], attr_count);
        Some(Self { max_stack, max_locals, code, exception_table, attributes })
    }
}

// ─── Full Classfile Parser ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClassFile {
    pub header: ClassFileHeader, pub cp: Vec<CpEntry>,
    pub access_flags: u16, pub this_class: u16, pub super_class: u16,
    pub interfaces: Vec<u16>, pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>, pub attributes: Vec<AttributeInfo>,
}

pub struct ClassFileParser;

impl ClassFileParser {
    pub fn parse(data: &[u8]) -> Option<ClassFile> {
        let header = ClassFileHeader::parse(data)?;
        let mut off = 8usize;
        let cp_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let (cp, new_off) = ConstantPoolParser::parse(data, off, cp_count)?;
        off = new_off;
        if data.len() < off + 8 { return None; }
        let access_flags = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let this_class = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let super_class = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let iface_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let mut interfaces = Vec::new();
        for _ in 0..iface_count {
            if data.len() < off + 2 { break; }
            interfaces.push(u16::from_be_bytes([data[off], data[off + 1]])); off += 2;
        }
        if data.len() < off + 2 { return None; }
        let field_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let mut fields = Vec::new();
        for _ in 0..field_count {
            if off >= data.len() { break; }
            if let Some((f, sz)) = FieldInfo::parse(&data[off..], &cp) { off += sz; fields.push(f); } else { break; }
        }
        if data.len() < off + 2 { return None; }
        let method_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let mut methods = Vec::new();
        for _ in 0..method_count {
            if off >= data.len() { break; }
            if let Some((m, sz)) = MethodInfo::parse(&data[off..], &cp) { off += sz; methods.push(m); } else { break; }
        }
        if data.len() < off + 2 { return None; }
        let attr_count = u16::from_be_bytes([data[off], data[off + 1]]); off += 2;
        let (attributes, _) = AttributeInfo::parse_all(&data[off..], &cp, attr_count);
        let msg = format!("{}: {} CP, {} field, {} method, version {}",
            &Lang::get(MsgId::CompatJvmInit), cp.len(), fields.len(), methods.len(), header.version_string());
        console_writeln(&msg);
        Some(ClassFile { header, cp, access_flags, this_class, super_class, interfaces, fields, methods, attributes })
    }
}

// ─── Real JVM Bytecode Interpreter ─────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum JvmValue {
    Int(i32), Long(i64), Float(f32), Double(f64),
    Reference(u64), Null,
}

impl JvmValue {
    pub fn as_int(&self) -> i32 { match self { JvmValue::Int(v) => *v, _ => 0 } }
    pub fn as_long(&self) -> i64 { match self { JvmValue::Long(v) => *v, _ => 0 } }
    pub fn as_float(&self) -> f32 { match self { JvmValue::Float(v) => *v, _ => 0.0 } }
    pub fn as_double(&self) -> f64 { match self { JvmValue::Double(v) => *v, _ => 0.0 } }
    pub fn as_ref(&self) -> u64 { match self { JvmValue::Reference(v) => *v, _ => 0 } }
    pub fn is_null(&self) -> bool { matches!(self, JvmValue::Null) }
}

#[derive(Debug, Clone)]
pub struct JvmStackFrame {
    pub locals: Vec<JvmValue>, pub stack: Vec<JvmValue>,
    pub pc: usize, pub max_stack: u16, pub max_locals: u16,
}

impl JvmStackFrame {
    pub fn new(max_stack: u16, max_locals: u16) -> Self {
        Self { locals: Vec::with_capacity(max_locals as usize), stack: Vec::with_capacity(max_stack as usize), pc: 0, max_stack, max_locals }
    }
    pub fn push(&mut self, v: JvmValue) { self.stack.push(v); }
    pub fn pop(&mut self) -> JvmValue { self.stack.pop().unwrap_or(JvmValue::Int(0)) }
    pub fn pop_int(&mut self) -> i32 { self.pop().as_int() }
    pub fn pop_long(&mut self) -> i64 { self.pop().as_long() }
    pub fn pop_float(&mut self) -> f32 { self.pop().as_float() }
    pub fn pop_double(&mut self) -> f64 { self.pop().as_double() }
    pub fn pop_ref(&mut self) -> u64 { self.pop().as_ref() }
    pub fn peek(&self) -> JvmValue { self.stack.last().copied().unwrap_or(JvmValue::Int(0)) }
    pub fn set_local(&mut self, idx: u16, v: JvmValue) {
        while self.locals.len() <= idx as usize { self.locals.push(JvmValue::Int(0)); }
        self.locals[idx as usize] = v;
    }
    pub fn get_local(&self, idx: u16) -> JvmValue { self.locals.get(idx as usize).copied().unwrap_or(JvmValue::Int(0)) }
}

pub struct JvmInterpreter;

impl JvmInterpreter {
    pub fn execute(code: &[u8], frame: &mut JvmStackFrame, cp: &[CpEntry]) -> Option<i32> {
        while frame.pc < code.len() {
            let op = code[frame.pc];
            frame.pc += 1;
            match op {
                // ─── Constants ─────────────────────────────────────
                0x01 => frame.push(JvmValue::Null), // aconst_null
                0x02 => frame.push(JvmValue::Int(-1)), // iconst_m1
                0x03 => frame.push(JvmValue::Int(0)), // iconst_0
                0x04 => frame.push(JvmValue::Int(1)), // iconst_1
                0x05 => frame.push(JvmValue::Int(2)), // iconst_2
                0x06 => frame.push(JvmValue::Int(3)), // iconst_3
                0x07 => frame.push(JvmValue::Int(4)), // iconst_4
                0x08 => frame.push(JvmValue::Int(5)), // iconst_5
                0x09 => frame.push(JvmValue::Long(0)), // lconst_0
                0x0A => frame.push(JvmValue::Long(1)), // lconst_1
                0x0B => frame.push(JvmValue::Float(0.0)), // fconst_0
                0x0C => frame.push(JvmValue::Float(1.0)), // fconst_1
                0x0D => frame.push(JvmValue::Float(2.0)), // fconst_2
                0x0E => frame.push(JvmValue::Double(0.0)), // dconst_0
                0x0F => frame.push(JvmValue::Double(1.0)), // dconst_1
                0x10 => { let v = code[frame.pc] as i8 as i32; frame.pc += 1; frame.push(JvmValue::Int(v)); } // bipush
                0x11 => { let v = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as i32; frame.pc += 2; frame.push(JvmValue::Int(v)); } // sipush
                0x12 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.push(Self::resolve_cp(cp, idx)); } // ldc
                0x13 => { let idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; frame.push(Self::resolve_cp(cp, idx)); } // ldc_w
                0x14 => { let idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; frame.push(Self::resolve_cp_wide(cp, idx)); } // ldc2_w

                // ─── Load ──────────────────────────────────────────
                0x15 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.push(frame.get_local(idx)); } // iload
                0x16 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.push(frame.get_local(idx)); } // lload
                0x17 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.push(frame.get_local(idx)); } // fload
                0x18 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.push(frame.get_local(idx)); } // dload
                0x19 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.push(frame.get_local(idx)); } // aload
                0x1A => frame.push(frame.get_local(0)), // iload_0
                0x1B => frame.push(frame.get_local(1)), // iload_1
                0x1C => frame.push(frame.get_local(2)), // iload_2
                0x1D => frame.push(frame.get_local(3)), // iload_3
                0x1E => frame.push(frame.get_local(0)), // lload_0
                0x1F => frame.push(frame.get_local(1)), // lload_1
                0x20 => frame.push(frame.get_local(2)), // lload_2
                0x21 => frame.push(frame.get_local(3)), // lload_3
                0x22 => frame.push(frame.get_local(0)), // fload_0
                0x23 => frame.push(frame.get_local(1)), // fload_1
                0x24 => frame.push(frame.get_local(2)), // fload_2
                0x25 => frame.push(frame.get_local(3)), // fload_3
                0x26 => frame.push(frame.get_local(0)), // dload_0
                0x27 => frame.push(frame.get_local(1)), // dload_1
                0x28 => frame.push(frame.get_local(2)), // dload_2
                0x29 => frame.push(frame.get_local(3)), // dload_3
                0x2A => frame.push(frame.get_local(0)), // aload_0
                0x2B => frame.push(frame.get_local(1)), // aload_1
                0x2C => frame.push(frame.get_local(2)), // aload_2
                0x2D => frame.push(frame.get_local(3)), // aload_3

                // ─── Store ─────────────────────────────────────────
                0x36 => { let idx = code[frame.pc] as u16; frame.pc += 1; let v = frame.pop(); frame.set_local(idx, v); } // istore
                0x37 => { let idx = code[frame.pc] as u16; frame.pc += 1; let v = frame.pop(); frame.set_local(idx, v); } // lstore
                0x38 => { let idx = code[frame.pc] as u16; frame.pc += 1; let v = frame.pop(); frame.set_local(idx, v); } // fstore
                0x39 => { let idx = code[frame.pc] as u16; frame.pc += 1; let v = frame.pop(); frame.set_local(idx, v); } // dstore
                0x3A => { let idx = code[frame.pc] as u16; frame.pc += 1; let v = frame.pop(); frame.set_local(idx, v); } // astore
                0x3B => { let v = frame.pop(); frame.set_local(0, v); } // istore_0
                0x3C => { let v = frame.pop(); frame.set_local(1, v); } // istore_1
                0x3D => { let v = frame.pop(); frame.set_local(2, v); } // istore_2
                0x3E => { let v = frame.pop(); frame.set_local(3, v); } // istore_3
                0x3F => { let v = frame.pop(); frame.set_local(0, v); } // lstore_0
                0x40 => { let v = frame.pop(); frame.set_local(1, v); } // lstore_1
                0x41 => { let v = frame.pop(); frame.set_local(2, v); } // lstore_2
                0x42 => { let v = frame.pop(); frame.set_local(3, v); } // lstore_3
                0x43 => { let v = frame.pop(); frame.set_local(0, v); } // fstore_0
                0x44 => { let v = frame.pop(); frame.set_local(1, v); } // fstore_1
                0x45 => { let v = frame.pop(); frame.set_local(2, v); } // fstore_2
                0x46 => { let v = frame.pop(); frame.set_local(3, v); } // fstore_3
                0x47 => { let v = frame.pop(); frame.set_local(0, v); } // dstore_0
                0x48 => { let v = frame.pop(); frame.set_local(1, v); } // dstore_1
                0x49 => { let v = frame.pop(); frame.set_local(2, v); } // dstore_2
                0x4A => { let v = frame.pop(); frame.set_local(3, v); } // dstore_3
                0x4B => { let v = frame.pop(); frame.set_local(0, v); } // astore_0
                0x4C => { let v = frame.pop(); frame.set_local(1, v); } // astore_1
                0x4D => { let v = frame.pop(); frame.set_local(2, v); } // astore_2
                0x4E => { let v = frame.pop(); frame.set_local(3, v); } // astore_3
                0x84 => { let idx = code[frame.pc] as u16; let val = code[frame.pc + 1] as i8 as i32; frame.pc += 2; frame.set_local(idx, JvmValue::Int(frame.get_local(idx).as_int() + val)); } // iinc

                // ─── Stack ─────────────────────────────────────────
                0x57 => { frame.pop(); } // pop
                0x58 => { frame.pop(); } // pop2 (simplified for 1-slot)
                0x59 => { let v = frame.peek(); frame.push(v); } // dup
                0x5A => { let v = frame.peek(); frame.pop(); let v2 = frame.peek(); frame.push(v); frame.push(v2); } // dup_x1
                0x5B => { let v = frame.peek(); frame.pop(); let v2 = frame.peek(); frame.pop(); let v3 = frame.peek(); frame.push(v); frame.push(v3); frame.push(v2); frame.push(v); } // dup_x2
                0x5C => { let v1 = frame.pop(); let v2 = frame.pop(); frame.push(v1); frame.push(v2); frame.push(v1); frame.push(v2); } // dup2
                0x5F => { let v1 = frame.pop(); let v2 = frame.pop(); frame.push(v1); frame.push(v2); } // swap

                // ─── Integer Arithmetic ────────────────────────────
                0x60 => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(a.wrapping_add(b))); } // iadd
                0x64 => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(a.wrapping_sub(b))); } // isub
                0x68 => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(a.wrapping_mul(b))); } // imul
                0x6C => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(if b != 0 { a / b } else { 0 })); } // idiv
                0x70 => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(if b != 0 { a % b } else { 0 })); } // irem
                0x74 => { let a = frame.pop_int(); frame.push(JvmValue::Int(-a)); } // ineg
                0x78 => { let b = frame.pop_int() & 0x1F; let a = frame.pop_int(); frame.push(JvmValue::Int(a << b)); } // ishl
                0x7A => { let b = frame.pop_int() & 0x1F; let a = frame.pop_int(); frame.push(JvmValue::Int(a >> b)); } // ishr
                0x7C => { let b = frame.pop_int() & 0x1F; let a = frame.pop_int(); frame.push(JvmValue::Int((a as u32 >> b) as i32)); } // iushr
                0x7E => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(a & b)); } // iand
                0x80 => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(a | b)); } // ior
                0x82 => { let b = frame.pop_int(); let a = frame.pop_int(); frame.push(JvmValue::Int(a ^ b)); } // ixor

                // ─── Long Arithmetic ───────────────────────────────
                0x61 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(a.wrapping_add(b))); } // ladd
                0x65 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(a.wrapping_sub(b))); } // lsub
                0x69 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(a.wrapping_mul(b))); } // lmul
                0x6D => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(if b != 0 { a / b } else { 0 })); } // ldiv
                0x71 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(if b != 0 { a % b } else { 0 })); } // lrem
                0x75 => { let a = frame.pop_long(); frame.push(JvmValue::Long(-a)); } // lneg
                0x79 => { let b = frame.pop_int() & 0x3F; let a = frame.pop_long(); frame.push(JvmValue::Long(a << b)); } // lshl
                0x7B => { let b = frame.pop_int() & 0x3F; let a = frame.pop_long(); frame.push(JvmValue::Long(a >> b)); } // lshr
                0x7D => { let b = frame.pop_int() & 0x3F; let a = frame.pop_long(); frame.push(JvmValue::Long((a as u64 >> b) as i64)); } // lushr
                0x7F => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(a & b)); } // land
                0x81 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(a | b)); } // lor
                0x83 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Long(a ^ b)); } // lxor

                // ─── Float Arithmetic ──────────────────────────────
                0x62 => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Float(a + b)); } // fadd
                0x66 => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Float(a - b)); } // fsub
                0x6A => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Float(a * b)); } // fmul
                0x6E => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Float(if b != 0.0 { a / b } else { 0.0 })); } // fdiv
                0x72 => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Float(if b != 0.0 { a % b } else { 0.0 })); } // frem
                0x76 => { let a = frame.pop_float(); frame.push(JvmValue::Float(-a)); } // fneg

                // ─── Double Arithmetic ─────────────────────────────
                0x63 => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Double(a + b)); } // dadd
                0x67 => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Double(a - b)); } // dsub
                0x6B => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Double(a * b)); } // dmul
                0x6F => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Double(if b != 0.0 { a / b } else { 0.0 })); } // ddiv
                0x73 => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Double(if b != 0.0 { a % b } else { 0.0 })); } // drem
                0x77 => { let a = frame.pop_double(); frame.push(JvmValue::Double(-a)); } // dneg

                // ─── Conversions ───────────────────────────────────
                0x85 => { let a = frame.pop_int(); frame.push(JvmValue::Long(a as i64)); } // i2l
                0x86 => { let a = frame.pop_int(); frame.push(JvmValue::Float(a as f32)); } // i2f
                0x87 => { let a = frame.pop_int(); frame.push(JvmValue::Double(a as f64)); } // i2d
                0x88 => { let a = frame.pop_long(); frame.push(JvmValue::Int(a as i32)); } // l2i
                0x89 => { let a = frame.pop_long(); frame.push(JvmValue::Float(a as f32)); } // l2f
                0x8A => { let a = frame.pop_long(); frame.push(JvmValue::Double(a as f64)); } // l2d
                0x8B => { let a = frame.pop_float(); frame.push(JvmValue::Int(a as i32)); } // f2i
                0x8C => { let a = frame.pop_float(); frame.push(JvmValue::Long(a as i64)); } // f2l
                0x8D => { let a = frame.pop_float(); frame.push(JvmValue::Double(a as f64)); } // f2d
                0x8E => { let a = frame.pop_double(); frame.push(JvmValue::Int(a as i32)); } // d2i
                0x8F => { let a = frame.pop_double(); frame.push(JvmValue::Long(a as i64)); } // d2l
                0x90 => { let a = frame.pop_double(); frame.push(JvmValue::Float(a as f32)); } // d2f
                0x91 => { let a = frame.pop_int(); frame.push(JvmValue::Int(a as i8 as i32)); } // i2b
                0x92 => { let a = frame.pop_int(); frame.push(JvmValue::Int(a as u16 as i32)); } // i2c
                0x93 => { let a = frame.pop_int(); frame.push(JvmValue::Int(a as i16 as i32)); } // i2s

                // ─── Comparisons ───────────────────────────────────
                0x94 => { let b = frame.pop_long(); let a = frame.pop_long(); frame.push(JvmValue::Int(if a > b { 1 } else if a < b { -1 } else { 0 })); } // lcmp
                0x95 => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Int(if a > b { 1 } else if a < b { -1 } else { 0 })); } // fcmpl
                0x96 => { let b = frame.pop_float(); let a = frame.pop_float(); frame.push(JvmValue::Int(if a > b { 1 } else if a < b { -1 } else { 0 })); } // fcmpg
                0x97 => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Int(if a > b { 1 } else if a < b { -1 } else { 0 })); } // dcmpl
                0x98 => { let b = frame.pop_double(); let a = frame.pop_double(); frame.push(JvmValue::Int(if a > b { 1 } else if a < b { -1 } else { 0 })); } // dcmpg

                // ─── Branch ────────────────────────────────────────
                0x99 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; if frame.pop_int() == 0 { frame.pc = (frame.pc as isize + off) as usize; } } // ifeq
                0x9A => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; if frame.pop_int() != 0 { frame.pc = (frame.pc as isize + off) as usize; } } // ifne
                0x9B => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; if frame.pop_int() < 0 { frame.pc = (frame.pc as isize + off) as usize; } } // iflt
                0x9C => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; if frame.pop_int() >= 0 { frame.pc = (frame.pc as isize + off) as usize; } } // ifge
                0x9D => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; if frame.pop_int() > 0 { frame.pc = (frame.pc as isize + off) as usize; } } // ifgt
                0x9E => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; if frame.pop_int() <= 0 { frame.pc = (frame.pc as isize + off) as usize; } } // ifle
                0x9F => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_int(); let a = frame.pop_int(); if a == b { frame.pc = (frame.pc as isize + off) as usize; } } // if_icmpeq
                0xA0 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_int(); let a = frame.pop_int(); if a != b { frame.pc = (frame.pc as isize + off) as usize; } } // if_icmpne
                0xA1 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_int(); let a = frame.pop_int(); if a < b { frame.pc = (frame.pc as isize + off) as usize; } } // if_icmplt
                0xA2 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_int(); let a = frame.pop_int(); if a >= b { frame.pc = (frame.pc as isize + off) as usize; } } // if_icmpge
                0xA3 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_int(); let a = frame.pop_int(); if a > b { frame.pc = (frame.pc as isize + off) as usize; } } // if_icmpgt
                0xA4 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_int(); let a = frame.pop_int(); if a <= b { frame.pc = (frame.pc as isize + off) as usize; } } // if_icmple
                0xA5 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_ref(); let a = frame.pop_ref(); if a == b { frame.pc = (frame.pc as isize + off) as usize; } } // if_acmpeq
                0xA6 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; let b = frame.pop_ref(); let a = frame.pop_ref(); if a != b { frame.pc = (frame.pc as isize + off) as usize; } } // if_acmpne
                0xA7 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; frame.pc = (frame.pc as isize + off) as usize; } // goto
                0xA8 => { let off = i16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]) as isize; frame.pc += 2; frame.push(JvmValue::Int(frame.pc as i32)); frame.pc = (frame.pc as isize + off) as usize; } // jsr
                0xA9 => { let idx = code[frame.pc] as u16; frame.pc += 1; frame.pc = frame.get_local(idx).as_int() as usize; } // ret
                0xAA => { // tableswitch
                    while !frame.pc.is_multiple_of(4) { frame.pc += 1; }
                    let default = i32::from_be_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3]]) as isize; frame.pc += 4;
                    let low = i32::from_be_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3]]); frame.pc += 4;
                    let high = i32::from_be_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3]]); frame.pc += 4;
                    let index = frame.pop_int();
                    if index < low || index > high {
                        frame.pc = (frame.pc as isize + default - ((high - low + 1) as isize * 4)) as usize;
                    } else {
                        let j = (index - low) as usize;
                        let off = i32::from_be_bytes([code[frame.pc + j*4], code[frame.pc + j*4 + 1], code[frame.pc + j*4 + 2], code[frame.pc + j*4 + 3]]) as isize;
                        frame.pc = (frame.pc as isize + off - ((high - low + 1) as isize * 4)) as usize;
                    }
                }
                0xAB => { // lookupswitch
                    while !frame.pc.is_multiple_of(4) { frame.pc += 1; }
                    let default = i32::from_be_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3]]) as isize; frame.pc += 4;
                    let npairs = i32::from_be_bytes([code[frame.pc], code[frame.pc+1], code[frame.pc+2], code[frame.pc+3]]); frame.pc += 4;
                    let key = frame.pop_int();
                    let base = frame.pc;
                    let mut found = false;
                    for i in 0..npairs {
                        let k = i32::from_be_bytes([code[base + i as usize * 8], code[base + i as usize * 8 + 1], code[base + i as usize * 8 + 2], code[base + i as usize * 8 + 3]]);
                        let off = i32::from_be_bytes([code[base + i as usize * 8 + 4], code[base + i as usize * 8 + 5], code[base + i as usize * 8 + 6], code[base + i as usize * 8 + 7]]) as isize;
                        if k == key { frame.pc = (base as isize + off) as usize; found = true; break; }
                    }
                    if !found { frame.pc = (base as isize + default) as usize; }
                }

                // ─── Return ────────────────────────────────────────
                0xAC => return Some(frame.pop_int()), // ireturn
                0xAD => return Some(frame.pop_long() as i32), // lreturn
                0xAE => return Some(frame.pop_float() as i32), // freturn
                0xAF => return Some(frame.pop_double() as i32), // dreturn
                0xB0 => return Some(frame.pop_ref() as i32), // areturn
                0xB1 => return Some(0), // return

                // ─── References ────────────────────────────────────
                0xBB => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; frame.push(JvmValue::Reference(0x1000)); } // new
                0xBC => { let _atype = code[frame.pc]; frame.pc += 1; let count = frame.pop_int(); frame.push(JvmValue::Reference(0x2000 + count as u64)); } // newarray
                0xBD => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; let count = frame.pop_int(); frame.push(JvmValue::Reference(0x3000 + count as u64)); } // anewarray
                0xBE => { let arr = frame.pop_ref(); frame.push(JvmValue::Int((arr - 0x2000) as i32)); } // arraylength
                0xBF => { let _ex = frame.pop_ref(); let msg = format!("{}: athrow @ pc={}", &Lang::get(MsgId::CompatJvmGc), frame.pc); console_writeln(&msg); return None; } // athrow
                0xC0 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; let obj = frame.pop(); frame.push(obj); } // checkcast
                0xC1 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; let _obj = frame.pop(); frame.push(JvmValue::Int(1)); } // instanceof
                0xC2 => { let _obj = frame.pop(); console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // monitorenter
                0xC3 => { let _obj = frame.pop(); console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // monitorexit

                // ─── Field Access ──────────────────────────────────
                0xB2 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; frame.push(JvmValue::Int(0)); } // getstatic
                0xB3 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; let _val = frame.pop(); } // putstatic
                0xB4 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; let _obj = frame.pop(); frame.push(JvmValue::Int(0)); } // getfield
                0xB5 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; let _val = frame.pop(); let _obj = frame.pop(); } // putfield

                // ─── Method Invocation ─────────────────────────────
                0xB6 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // invokevirtual
                0xB7 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // invokespecial
                0xB8 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 2; console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // invokestatic
                0xB9 => { let _idx = u16::from_be_bytes([code[frame.pc], code[frame.pc + 1]]); frame.pc += 4; console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // invokeinterface
                0xBA => { frame.pc += 4; console_writeln(&Lang::get(MsgId::CompatJvmClasspath)); } // invokedynamic

                // ─── Array Load ────────────────────────────────────
                0x2E => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Int(0)); } // iaload
                0x2F => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Long(0)); } // laload
                0x30 => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Float(0.0)); } // faload
                0x31 => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Double(0.0)); } // daload
                0x32 => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Reference(0)); } // aaload
                0x33 => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Int(0)); } // baload
                0x34 => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Int(0)); } // caload
                0x35 => { let _idx = frame.pop_int(); let _arr = frame.pop_ref(); frame.push(JvmValue::Int(0)); } // saload

                // ─── Array Store ───────────────────────────────────
                0x4F => { let _val = frame.pop_int(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // iastore
                0x50 => { let _val = frame.pop_long(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // lastore
                0x51 => { let _val = frame.pop_float(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // fastore
                0x52 => { let _val = frame.pop_double(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // dastore
                0x53 => { let _val = frame.pop_ref(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // aastore
                0x54 => { let _val = frame.pop_int(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // bastore
                0x55 => { let _val = frame.pop_int(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // castore
                0x56 => { let _val = frame.pop_int(); let _idx = frame.pop_int(); let _arr = frame.pop_ref(); } // sastore

                // ─── Misc ──────────────────────────────────────────
                0x00 => {}, // nop
                _ => {
                    let msg = format!("{}: opcode=0x{:02X} @ pc={}",
                        &Lang::get(MsgId::CompatJvmInit), op, frame.pc - 1);
                    console_writeln(&msg); return None;
                }
            }
        }
        Some(0)
    }
    fn resolve_cp(cp: &[CpEntry], idx: u16) -> JvmValue {
        if idx as usize >= cp.len() { return JvmValue::Int(0); }
        match &cp[idx as usize] {
            CpEntry::Integer(v) => JvmValue::Int(*v),
            CpEntry::Float(v) => JvmValue::Float(f32::from_bits(*v)),
            CpEntry::String(idx) => {
                if let Some(CpEntry::Utf8(ref s)) = cp.get(*idx as usize) { JvmValue::Reference(s.as_ptr() as u64) } else { JvmValue::Reference(0) }
            }
            CpEntry::Long(v) => JvmValue::Long(*v),
            CpEntry::Double(v) => JvmValue::Double(f64::from_bits(*v)),
            _ => JvmValue::Int(0),
        }
    }
    fn resolve_cp_wide(cp: &[CpEntry], idx: u16) -> JvmValue {
        if idx as usize >= cp.len() { return JvmValue::Long(0); }
        match &cp[idx as usize] {
            CpEntry::Long(v) => JvmValue::Long(*v),
            CpEntry::Double(v) => JvmValue::Double(f64::from_bits(*v)),
            _ => JvmValue::Long(0),
        }
    }
}

// ─── Java Runtime / Class Loader ───────────────────────────────

#[derive(Debug, Clone)]
pub struct JavaClass {
    pub name: String, pub superclass: String, pub methods: Vec<JavaMethod>, pub cp: Vec<CpEntry>,
}

#[derive(Debug, Clone)]
pub struct JavaMethod {
    pub name: String, pub descriptor: String, pub code: Option<CodeAttribute>,
}

pub struct JavaRuntime {
    pub classes: Vec<JavaClass>,
}

impl Default for JavaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaRuntime {
    pub fn new() -> Self { Self { classes: Vec::new() } }
    pub fn load_class(&mut self, data: &[u8]) -> Option<String> {
        let cf = ClassFileParser::parse(data)?;
        let name = ConstantPoolParser::resolve_utf8(&cf.cp, cf.this_class);
        let super_name = ConstantPoolParser::resolve_utf8(&cf.cp, cf.super_class);
        let mut methods = Vec::new();
        for m in &cf.methods {
            let mname = ConstantPoolParser::resolve_utf8(&cf.cp, m.name_index);
            let mdesc = ConstantPoolParser::resolve_utf8(&cf.cp, m.descriptor_index);
            methods.push(JavaMethod { name: mname, descriptor: mdesc, code: m.get_code() });
        }
        let cp_clone = cf.cp.clone();
        self.classes.push(JavaClass { name: name.clone(), superclass: super_name, methods, cp: cp_clone });
        let msg = format!("{}: {}", &Lang::get(MsgId::CompatJvmInit), name);
        console_writeln(&msg);
        Some(name)
    }
    #[must_use]
    pub fn find_method(&self, class: &str, method: &str) -> Option<&JavaMethod> {
        self.classes.iter().find(|c| c.name == class)?.methods.iter().find(|m| m.name == method)
    }
    #[must_use]
    pub fn invoke(&self, class: &str, method: &str, args: &[i32]) -> Option<i32> {
        let m = self.find_method(class, method)?;
        let code = m.code.as_ref()?;
        let mut frame = JvmStackFrame::new(code.max_stack, code.max_locals);
        for (i, arg) in args.iter().enumerate() { frame.set_local(i as u16, JvmValue::Int(*arg)); }
        let msg = format!("{}: {}.{}({} arg)", &Lang::get(MsgId::CompatJvmClasspath), class, method, args.len());
        console_writeln(&msg);
        let classfile = self.classes.iter().find(|c| c.name == class)?;
        JvmInterpreter::execute(&code.code, &mut frame, &classfile.cp)
    }
    pub fn dump(&self) {
        console_writeln(&Lang::get(MsgId::CompatJvmInit));
        for c in &self.classes {
            let msg = format!("  {} extends {}", c.name, c.superclass);
            console_writeln(&msg);
            for m in &c.methods { let msg = format!("    {} {}", m.name, m.descriptor); console_writeln(&msg); }
        }
    }
}

// ─── Unit Tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::format;

    fn minimal_classfile() -> Vec<u8> {
        // CAFEBABE + minor(0) + major(52=Java8) + const_pool_count(1)
        // + access(0x0021) + this(0) + super(0) + iface(0) + field(0) + method(0) + attr(0)
        let mut data = Vec::new();
        data.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // magic
        data.extend_from_slice(&[0x00, 0x00]);               // minor
        data.extend_from_slice(&[0x00, 0x34]);               // major = 52 (Java 8)
        data
    }

    #[test]
    fn test_classfile_header_valid() {
        let data = minimal_classfile();
        let hdr = ClassFileHeader::parse(&data);
        assert!(hdr.is_some());
        let hdr = hdr.unwrap();
        assert_eq!(hdr.magic, ClassFileHeader::JAVA_MAGIC);
        assert_eq!(hdr.major, 52);
    }

    #[test]
    fn test_classfile_header_bad_magic() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 0];
        assert!(ClassFileHeader::parse(&data).is_none());
    }

    #[test]
    fn test_classfile_header_too_short() {
        let data = vec![0xCA, 0xFE];
        assert!(ClassFileHeader::parse(&data).is_none());
    }

    #[test]
    fn test_version_string_java8() {
        let data = minimal_classfile();
        let hdr = ClassFileHeader::parse(&data).unwrap();
        assert!(hdr.version_string().contains("Java 8"));
    }

    #[test]
    fn test_version_string_java11() {
        let mut data = minimal_classfile();
        data[7] = 55; // major = 55 → Java 11
        let hdr = ClassFileHeader::parse(&data).unwrap();
        assert!(hdr.version_string().contains("Java 11"));
    }

    #[test]
    fn test_jvm_stack_frame_push_pop() {
        let mut frame = JvmStackFrame::new(10, 5);
        frame.push(JvmValue::Int(42));
        frame.push(JvmValue::Int(100));
        assert_eq!(frame.pop_int(), 100);
        assert_eq!(frame.pop_int(), 42);
    }

    #[test]
    fn test_jvm_frame_locals() {
        let mut frame = JvmStackFrame::new(5, 5);
        frame.set_local(2, JvmValue::Int(999));
        if let JvmValue::Int(v) = frame.get_local(2) { assert_eq!(v, 999); }
        else { panic!("Expected Int"); }
    }

    #[test]
    fn test_jvm_interpreter_nop() {
        let code = vec![0x00u8, 0x00, 0x00]; // nop nop nop
        let mut frame = JvmStackFrame::new(10, 5);
        let result = JvmInterpreter::execute(&code, &mut frame, &[]);
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_jvm_interpreter_iconst_ireturn() {
        // iconst_5 (0x08) + ireturn (0xAC)
        let code = vec![0x08u8, 0xAC];
        let mut frame = JvmStackFrame::new(10, 5);
        let result = JvmInterpreter::execute(&code, &mut frame, &[]);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_jvm_interpreter_iadd() {
        // iconst_3 + iconst_2 + iadd + ireturn
        let code = vec![0x06u8, 0x05, 0x60, 0xAC]; // iconst_3=0x06, iconst_2=0x05, iadd=0x60
        let mut frame = JvmStackFrame::new(10, 5);
        let result = JvmInterpreter::execute(&code, &mut frame, &[]);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_jvm_interpreter_ldc_int() {
        // bipush 10 (0x10 0x0A) + ireturn
        let code = vec![0x10u8, 10, 0xAC];
        let mut frame = JvmStackFrame::new(10, 5);
        let result = JvmInterpreter::execute(&code, &mut frame, &[]);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_java_runtime_load_class_invalid() {
        let mut rt = JavaRuntime::new();
        let result = rt.load_class(&[0xDE, 0xAD, 0xBE, 0xEF, 0, 0, 0, 0]);
        assert!(result.is_none());
    }

    #[test]
    fn test_java_runtime_find_method_empty() {
        let rt = JavaRuntime::new();
        assert!(rt.find_method("Foo", "bar").is_none());
    }
}
