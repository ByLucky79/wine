extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ffi::CStr;

const MAX_INT_ATOM: u16 = 0xBFFF;

type Atom = u16;
type Bool = i32;
type Dword = u32;
type Uint = u32;
type Wchar = u16;

const TRUE: Bool = 1;

const ERROR_MORE_DATA: Dword = 234;

struct AtomEntry {
    name: Vec<Wchar>,
    ref_count: u32,
}

struct AtomTable {
    atoms: BTreeMap<Atom, AtomEntry>,
    next_handle: u16,
}

impl AtomTable {
    fn new() -> Self {
        AtomTable {
            atoms: BTreeMap::new(),
            next_handle: 1,
        }
    }

    fn add(&mut self, name: &[Wchar]) -> Atom {
        for (atom, entry) in &self.atoms {
            if entry.name == name {
                return *atom;
            }
        }

        let atom = self.next_handle;
        self.next_handle = self.next_handle.wrapping_add(1);
        if self.next_handle > MAX_INT_ATOM {
            self.next_handle = 1;
        }
        self.atoms.insert(atom, AtomEntry {
            name: name.to_vec(),
            ref_count: 1,
        });
        atom
    }

    fn find(&self, name: &[Wchar]) -> Option<Atom> {
        for (atom, entry) in &self.atoms {
            if entry.name == name {
                return Some(*atom);
            }
        }
        None
    }

    fn delete(&mut self, atom: Atom) -> bool {
        if let Some(entry) = self.atoms.get_mut(&atom) {
            entry.ref_count -= 1;
            if entry.ref_count == 0 {
                self.atoms.remove(&atom);
            }
            true
        } else {
            false
        }
    }

    fn query(&self, atom: Atom) -> Option<&[Wchar]> {
        self.atoms.get(&atom).map(|e| e.name.as_slice())
    }
}

use core::cell::UnsafeCell;

struct GlobalState {
    local_table: UnsafeCell<Option<AtomTable>>,
}

unsafe impl Sync for GlobalState {}

static GLOBAL: GlobalState = GlobalState {
    local_table: UnsafeCell::new(None),
};

fn with_local_table<F, R>(f: F) -> R
where
    F: FnOnce(&mut AtomTable) -> R,
{
    let cell = unsafe { &mut *GLOBAL.local_table.get() };
    if cell.is_none() {
        *cell = Some(AtomTable::new());
    }
    f(cell.as_mut().unwrap())
}

fn set_last_error(_code: Dword) {}

pub fn init_atom_table(_entries: Dword) -> Bool {
    with_local_table(|_| TRUE)
}

pub fn global_add_atom_a(str_ptr: *const u8) -> Atom {
    let cstr = unsafe { CStr::from_ptr(str_ptr as *const i8) };
    let bytes = cstr.to_bytes();
    let wide: Vec<Wchar> = bytes.iter().map(|&b| b as Wchar).collect();
    with_local_table(|t| t.add(&wide))
}

pub fn add_atom_a(str_ptr: *const u8) -> Atom {
    global_add_atom_a(str_ptr)
}

pub fn global_add_atom_w(str_ptr: *const u16) -> Atom {
    let mut wide = Vec::new();
    let mut i = 0;
    loop {
        let c = unsafe { *str_ptr.add(i) };
        if c == 0 { break; }
        wide.push(c);
        i += 1;
    }
    with_local_table(|t| t.add(&wide))
}

pub fn add_atom_w(str_ptr: *const u16) -> Atom {
    global_add_atom_w(str_ptr)
}

pub fn global_delete_atom(atom: Atom) -> Atom {
    if atom <= MAX_INT_ATOM {
        with_local_table(|t| {
            if t.delete(atom) { 0 } else { atom }
        })
    } else {
        0
    }
}

pub fn delete_atom(atom: Atom) -> Atom {
    if atom <= MAX_INT_ATOM {
        with_local_table(|t| {
            if t.delete(atom) { 0 } else { atom }
        })
    } else {
        atom
    }
}

pub fn global_find_atom_a(str_ptr: *const u8) -> Atom {
    let cstr = unsafe { CStr::from_ptr(str_ptr as *const i8) };
    let bytes = cstr.to_bytes();
    let wide: Vec<Wchar> = bytes.iter().map(|&b| b as Wchar).collect();
    with_local_table(|t| t.find(&wide).unwrap_or(0))
}

pub fn find_atom_a(str_ptr: *const u8) -> Atom {
    global_find_atom_a(str_ptr)
}

pub fn global_find_atom_w(str_ptr: *const u16) -> Atom {
    let mut wide = Vec::new();
    let mut i = 0;
    loop {
        let c = unsafe { *str_ptr.add(i) };
        if c == 0 { break; }
        wide.push(c);
        i += 1;
    }
    with_local_table(|t| t.find(&wide).unwrap_or(0))
}

pub fn find_atom_w(str_ptr: *const u16) -> Atom {
    global_find_atom_w(str_ptr)
}

pub fn global_get_atom_name_a(atom: Atom, buffer: *mut u8, count: i32) -> Uint {
    if count <= 0 {
        set_last_error(ERROR_MORE_DATA);
        return 0;
    }

    let name = with_local_table(|t| t.query(atom).map(|n| n.to_vec()));
    match name {
        Some(wide) => {
            let mut len = 0;
            for &c in &wide {
                if (len as i32) < count - 1 {
                    unsafe { *buffer.add(len) = c as u8; }
                    len += 1;
                } else {
                    set_last_error(ERROR_MORE_DATA);
                    return 0;
                }
            }
            unsafe { *buffer.add(len) = 0; }
            len as Uint
        }
        None => 0,
    }
}

pub fn get_atom_name_a(atom: Atom, buffer: *mut u8, count: i32) -> Uint {
    global_get_atom_name_a(atom, buffer, count)
}

pub fn global_get_atom_name_w(atom: Atom, buffer: *mut u16, count: i32) -> Uint {
    if count <= 0 {
        set_last_error(ERROR_MORE_DATA);
        return 0;
    }

    let name = with_local_table(|t| t.query(atom).map(|n| n.to_vec()));
    match name {
        Some(wide) => {
            let copy_len = wide.len().min((count - 1) as usize);
            for i in 0..copy_len {
                unsafe { *buffer.add(i) = wide[i]; }
            }
            unsafe { *buffer.add(copy_len) = 0; }
            copy_len as Uint
        }
        None => 0,
    }
}

pub fn get_atom_name_w(atom: Atom, buffer: *mut u16, count: i32) -> Uint {
    global_get_atom_name_w(atom, buffer, count)
}
