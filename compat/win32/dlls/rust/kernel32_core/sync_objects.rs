// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 synchronization object API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/sync_objects.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 kernel synchronization objects: events (Create/Open/
//   Set/Reset/Pulse), mutexes (Create/Open/Release), semaphores (Create/Open/
//   Release), waitable timers (Create/Open/Set/Cancel), and wait functions
//   (WaitForSingleObject, WaitForMultipleObjects, SignalObjectAndWait,
//   MsgWaitForMultipleObjects).  All A/W variants included.
//
// Bağımlı Dosyalar:
//   1-) alloc (crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Clean-room Rust port from Win32 API spec
// *******************************************************************

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Long = i32;
type Wchar = u16;
type Handle = *mut core::ffi::c_void;
type Lpvoid = *mut core::ffi::c_void;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const MAX_PATH: usize = 260;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_ALREADY_EXISTS: Dword = 183;
const ERROR_FILE_NOT_FOUND: Dword = 2;
const ERROR_FILENAME_EXCED_RANGE: Dword = 206;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;

const WAIT_OBJECT_0: Dword = 0;
const WAIT_TIMEOUT: Dword = 0x00000102;
const WAIT_ABANDONED: Dword = 0x00000080;
const WAIT_FAILED: Dword = 0xFFFFFFFF;

const INFINITE: Dword = 0xFFFFFFFF;
const EVENT_ALL_ACCESS: Dword = 0x001F0003;
const MUTEX_ALL_ACCESS: Dword = 0x001F0001;
const SEMAPHORE_ALL_ACCESS: Dword = 0x001F0003;
const TIMER_ALL_ACCESS: Dword = 0x001F0003;

const CREATE_WAITABLE_TIMER_MANUAL_RESET: Dword = 0x00000001;

const QS_KEY: Dword = 0x0001;
const QS_MOUSEMOVE: Dword = 0x0002;
const QS_MOUSEBUTTON: Dword = 0x0004;
const QS_POSTMESSAGE: Dword = 0x0008;
const QS_TIMER: Dword = 0x0010;
const QS_PAINT: Dword = 0x0020;
const QS_SENDMESSAGE: Dword = 0x0040;
const QS_HOTKEY: Dword = 0x0080;
const QS_ALLPOSTMESSAGE: Dword = 0x0100;
const QS_INPUT: Dword = QS_MOUSEMOVE | QS_MOUSEBUTTON | QS_KEY;
const QS_ALLEVENTS: Dword = QS_INPUT | QS_POSTMESSAGE | QS_TIMER | QS_PAINT | QS_HOTKEY;
const QS_ALLINPUT: Dword = QS_INPUT | QS_POSTMESSAGE | QS_TIMER | QS_PAINT | QS_HOTKEY | QS_SENDMESSAGE;

fn set_last_error(_code: Dword) {}

fn lstrlen_w(s: *const Wchar) -> usize {
    if s.is_null() { return 0; }
    let mut len: usize = 0;
    unsafe { while *s.add(len) != 0 { len += 1; } }
    len
}

// ========================================================================
// Kernel object base — handle table
// ========================================================================

#[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
enum ObjKind {
    Event = 1,
    Mutex = 2,
    Semaphore = 3,
    Timer = 4,
}

struct KernelObject {
    kind: ObjKind,
    signaled: AtomicBool,
    manual_reset: bool,
    count: AtomicI32,
    max_count: i32,
    due_time: AtomicU64,
    periodic: bool,
    name: Vec<Wchar>,
}

use core::cell::UnsafeCell;

struct ObjState {
    objects: UnsafeCell<BTreeMap<Handle, KernelObject>>,
    next_handle: AtomicU64,
}

unsafe impl Sync for ObjState {}

static OBJ_STATE: ObjState = ObjState {
    objects: UnsafeCell::new(BTreeMap::new()),
    next_handle: AtomicU64::new(0x1000),
};

fn alloc_handle() -> Handle {
    let val = OBJ_STATE.next_handle.fetch_add(1, Ordering::Relaxed);
    val as usize as Handle
}

fn make_named_key(name: &[Wchar], prefix: u64) -> u64 {
    let mut h: u64 = prefix;
    for &c in name {
        let uc = if c >= b'A' as Wchar && c <= b'Z' as Wchar { c + 0x20 } else { c };
        h = h.wrapping_mul(33).wrapping_add(uc as u64);
    }
    h
}

fn register_object(obj: KernelObject) -> Handle {
    let h = alloc_handle();
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    cells.insert(h, obj);
    h
}

fn find_named_object(kind: ObjKind, name: &[Wchar]) -> Option<Handle> {
    let cells = unsafe { &*OBJ_STATE.objects.get() };
    for (&h, obj) in cells.iter() {
        if obj.kind == kind && obj.name == name {
            return Some(h);
        }
    }
    None
}

fn wait_on_object(h: Handle, _ms: Dword) -> Dword {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        if !obj.manual_reset && obj.signaled.load(Ordering::Acquire) {
            obj.signaled.store(false, Ordering::Release);
            WAIT_OBJECT_0
        } else if obj.manual_reset && obj.signaled.load(Ordering::Acquire) {
            WAIT_OBJECT_0
        } else {
            WAIT_TIMEOUT
        }
    } else {
        WAIT_FAILED
    }
}

// ========================================================================
// Events
// ========================================================================

pub fn create_event_w(
    sa: *mut core::ffi::c_void,
    manual_reset: Bool,
    initial_state: Bool,
    name: *const Wchar,
) -> Handle {
    let obj = KernelObject {
        kind: ObjKind::Event,
        signaled: AtomicBool::new(initial_state != 0),
        manual_reset: manual_reset != 0,
        count: AtomicI32::new(0),
        max_count: 0,
        due_time: AtomicU64::new(0),
        periodic: false,
        name: if name.is_null() {
            Vec::new()
        } else {
            let len = lstrlen_w(name);
            unsafe { core::slice::from_raw_parts(name, len) }.to_vec()
        },
    };
    let h = register_object(obj);
    if !name.is_null() {
        if let Some(existing) = find_named_object(ObjKind::Event, &[]) {
            set_last_error(ERROR_ALREADY_EXISTS);
        }
    }
    h
}

pub fn create_event_a(
    sa: *mut core::ffi::c_void,
    manual_reset: Bool,
    initial_state: Bool,
    name: *const u8,
) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        let mut i = 0;
        unsafe {
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    create_event_w(sa, manual_reset, initial_state, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn open_event_w(access: Dword, inherit: Bool, name: *const Wchar) -> Handle {
    if name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let len = lstrlen_w(name);
    let name_slice = unsafe { core::slice::from_raw_parts(name, len) };
    find_named_object(ObjKind::Event, name_slice).unwrap_or_else(|| {
        set_last_error(ERROR_FILE_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn open_event_a(access: Dword, inherit: Bool, name: *const u8) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    open_event_w(access, inherit, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn set_event(h: Handle) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        obj.signaled.store(true, Ordering::Release);
        TRUE
    } else { FALSE }
}

pub fn reset_event(h: Handle) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        obj.signaled.store(false, Ordering::Release);
        TRUE
    } else { FALSE }
}

pub fn pulse_event(h: Handle) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        obj.signaled.store(true, Ordering::Release);
        obj.signaled.store(false, Ordering::Release);
        TRUE
    } else { FALSE }
}

// ========================================================================
// Mutexes
// ========================================================================

pub fn create_mutex_w(
    sa: *mut core::ffi::c_void,
    initial_owner: Bool,
    name: *const Wchar,
) -> Handle {
    let initial_count = if initial_owner != 0 { 1 } else { 0 };
    let obj = KernelObject {
        kind: ObjKind::Mutex,
        signaled: AtomicBool::new(initial_owner == 0),
        manual_reset: false,
        count: AtomicI32::new(initial_count),
        max_count: 1,
        due_time: AtomicU64::new(0),
        periodic: false,
        name: if name.is_null() {
            Vec::new()
        } else {
            let len = lstrlen_w(name);
            unsafe { core::slice::from_raw_parts(name, len) }.to_vec()
        },
    };
    let h = register_object(obj);
    h
}

pub fn create_mutex_a(
    sa: *mut core::ffi::c_void,
    initial_owner: Bool,
    name: *const u8,
) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    create_mutex_w(sa, initial_owner, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn open_mutex_w(access: Dword, inherit: Bool, name: *const Wchar) -> Handle {
    if name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let len = lstrlen_w(name);
    let name_slice = unsafe { core::slice::from_raw_parts(name, len) };
    find_named_object(ObjKind::Mutex, name_slice).unwrap_or_else(|| {
        set_last_error(ERROR_FILE_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn open_mutex_a(access: Dword, inherit: Bool, name: *const u8) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    open_mutex_w(access, inherit, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn release_mutex(h: Handle) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        let c = obj.count.fetch_sub(1, Ordering::Release);
        if c <= 1 {
            obj.signaled.store(true, Ordering::Release);
        }
        TRUE
    } else { FALSE }
}

// ========================================================================
// Semaphores
// ========================================================================

pub fn create_semaphore_w(
    sa: *mut core::ffi::c_void,
    initial: Long,
    max: Long,
    name: *const Wchar,
) -> Handle {
    if initial < 0 || max <= 0 || initial > max {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let obj = KernelObject {
        kind: ObjKind::Semaphore,
        signaled: AtomicBool::new(initial > 0),
        manual_reset: false,
        count: AtomicI32::new(initial),
        max_count: max,
        due_time: AtomicU64::new(0),
        periodic: false,
        name: if name.is_null() {
            Vec::new()
        } else {
            let len = lstrlen_w(name);
            unsafe { core::slice::from_raw_parts(name, len) }.to_vec()
        },
    };
    register_object(obj)
}

pub fn create_semaphore_a(
    sa: *mut core::ffi::c_void,
    initial: Long,
    max: Long,
    name: *const u8,
) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    create_semaphore_w(sa, initial, max, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn open_semaphore_w(access: Dword, inherit: Bool, name: *const Wchar) -> Handle {
    if name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let len = lstrlen_w(name);
    let name_slice = unsafe { core::slice::from_raw_parts(name, len) };
    find_named_object(ObjKind::Semaphore, name_slice).unwrap_or_else(|| {
        set_last_error(ERROR_FILE_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn open_semaphore_a(access: Dword, inherit: Bool, name: *const u8) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    open_semaphore_w(access, inherit, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn release_semaphore(h: Handle, release_count: Long, previous: *mut Long) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        let old = obj.count.fetch_add(release_count, Ordering::Release);
        if !previous.is_null() {
            unsafe { *previous = old; }
        }
        if old + release_count > 0 {
            obj.signaled.store(true, Ordering::Release);
        }
        TRUE
    } else { FALSE }
}

// ========================================================================
// Waitable Timers
// ========================================================================

pub fn create_waitable_timer_w(
    sa: *mut core::ffi::c_void,
    manual_reset: Bool,
    name: *const Wchar,
) -> Handle {
    let obj = KernelObject {
        kind: ObjKind::Timer,
        signaled: AtomicBool::new(false),
        manual_reset: manual_reset != 0,
        count: AtomicI32::new(0),
        max_count: 0,
        due_time: AtomicU64::new(0),
        periodic: false,
        name: if name.is_null() {
            Vec::new()
        } else {
            let len = lstrlen_w(name);
            unsafe { core::slice::from_raw_parts(name, len) }.to_vec()
        },
    };
    register_object(obj)
}

pub fn create_waitable_timer_a(
    sa: *mut core::ffi::c_void,
    manual_reset: Bool,
    name: *const u8,
) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    create_waitable_timer_w(sa, manual_reset, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn open_waitable_timer_w(access: Dword, inherit: Bool, name: *const Wchar) -> Handle {
    if name.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let len = lstrlen_w(name);
    let name_slice = unsafe { core::slice::from_raw_parts(name, len) };
    find_named_object(ObjKind::Timer, name_slice).unwrap_or_else(|| {
        set_last_error(ERROR_FILE_NOT_FOUND);
        ptr::null_mut()
    })
}

pub fn open_waitable_timer_a(access: Dword, inherit: Bool, name: *const u8) -> Handle {
    let mut wide: Vec<Wchar> = Vec::new();
    if !name.is_null() {
        unsafe {
            let mut i = 0;
            loop {
                let c = *name.add(i);
                if c == 0 { break; }
                wide.push(c as Wchar);
                i += 1;
            }
        }
    }
    open_waitable_timer_w(access, inherit, if name.is_null() { ptr::null() } else { wide.as_ptr() })
}

pub fn set_waitable_timer(
    h: Handle,
    due_time: *mut i64,
    period: Long,
    _callback: Option<unsafe extern "C" fn(Lpvoid, Bool, Lpvoid)>,
    _arg: Lpvoid,
    _resume: Bool,
) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        let dt = unsafe { *due_time };
        obj.due_time.store(dt as u64, Ordering::Release);
        obj.periodic = period != 0;
        if dt <= 0 {
            obj.signaled.store(true, Ordering::Release);
        }
        TRUE
    } else { FALSE }
}

pub fn cancel_waitable_timer(h: Handle) -> Bool {
    let cells = unsafe { &mut *OBJ_STATE.objects.get() };
    if let Some(obj) = cells.get_mut(&h) {
        obj.signaled.store(false, Ordering::Release);
        TRUE
    } else { FALSE }
}

// ========================================================================
// Wait functions
// ========================================================================

pub fn wait_for_single_object(h: Handle, ms: Dword) -> Dword {
    wait_on_object(h, ms)
}

pub fn wait_for_multiple_objects(
    count: Dword,
    handles: *const Handle,
    wait_all: Bool,
    ms: Dword,
) -> Dword {
    if count == 0 || handles.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return WAIT_FAILED;
    }
    let slice = unsafe { core::slice::from_raw_parts(handles, count as usize) };
    if wait_all != 0 {
        for &h in slice {
            let ret = wait_on_object(h, ms);
            if ret == WAIT_TIMEOUT { return WAIT_TIMEOUT; }
        }
        WAIT_OBJECT_0
    } else {
        for (i, &h) in slice.iter().enumerate() {
            let ret = wait_on_object(h, ms);
            if ret != WAIT_TIMEOUT {
                return WAIT_OBJECT_0 + (i as Dword);
            }
        }
        WAIT_TIMEOUT
    }
}

pub fn signal_object_and_wait(
    to_signal: Handle,
    to_wait: Handle,
    ms: Dword,
    _alertable: Bool,
) -> Dword {
    let mut signaled = false;
    {
        let cells = unsafe { &mut *OBJ_STATE.objects.get() };
        if let Some(obj) = cells.get_mut(&to_signal) {
            obj.signaled.store(true, Ordering::Release);
            signaled = true;
        }
    }
    if !signaled { return WAIT_FAILED; }
    wait_on_object(to_wait, ms)
}

pub fn msg_wait_for_multiple_objects(
    count: Dword,
    handles: *const Handle,
    wait_all: Bool,
    ms: Dword,
    wake_mask: Dword,
) -> Dword {
    wait_for_multiple_objects(count, handles, wait_all, ms)
}
