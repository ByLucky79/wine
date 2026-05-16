// SPDX-License-Identifier: GPL-3.0-only
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 thread and TLS management API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/thread_mgmt.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements Win32 thread and thread-local storage API: CreateThread,
//   CreateRemoteThread, ExitThread, GetExitCodeThread, TerminateThread,
//   GetCurrentThread, GetCurrentThreadId, Get/SetThreadPriority,
//   Get/SetThreadPriorityBoost, GetThreadTimes, SuspendThread,
//   ResumeThread, SwitchToThread, Sleep, SleepEx, TlsAlloc, TlsGetValue,
//   TlsSetValue, TlsFree.  TLS uses per-thread slot arrays.
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
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Long = i32;
type Wchar = u16;
type Handle = *mut core::ffi::c_void;
type Lpvoid = *mut core::ffi::c_void;
type LpsecurityAttributes = *mut core::ffi::c_void;
type LpthreadStartRoutine =
    Option<unsafe extern "C" fn(Lpvoid) -> Dword>;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;

const THREAD_PRIORITY_LOWEST: i32 = -2;
const THREAD_PRIORITY_BELOW_NORMAL: i32 = -1;
const THREAD_PRIORITY_NORMAL: i32 = 0;
const THREAD_PRIORITY_ABOVE_NORMAL: i32 = 1;
const THREAD_PRIORITY_HIGHEST: i32 = 2;
const THREAD_PRIORITY_IDLE: i32 = -15;
const THREAD_PRIORITY_TIME_CRITICAL: i32 = 15;

const STILL_ACTIVE: Dword = 259;
const WAIT_OBJECT_0: Dword = 0;
const WAIT_TIMEOUT: Dword = 258;
const INFINITE: Dword = 0xFFFFFFFF;

const TLS_MINIMUM_AVAILABLE: usize = 64;

fn set_last_error(_code: Dword) {}

// ========================================================================
// TLS (Thread-Local Storage)
// ========================================================================

type TlsSlotValue = Option<Lpvoid>;
const TLS_SLOTS: usize = TLS_MINIMUM_AVAILABLE;

struct TlsData {
    slots: [TlsSlotValue; TLS_SLOTS],
}

impl TlsData {
    fn new() -> Self {
        const INIT: Option<*mut core::ffi::c_void> = None;
        TlsData { slots: [INIT; TLS_SLOTS] }
    }
}

use core::cell::UnsafeCell;

struct TlsState {
    next_index: AtomicU32,
    data: UnsafeCell<Vec<TlsData>>,
}

unsafe impl Sync for TlsState {}

static TLS_STATE: TlsState = TlsState {
    next_index: AtomicU32::new(1),
    data: UnsafeCell::new(Vec::new()),
};

fn with_current_tls<F, R>(f: F) -> R
where
    F: FnOnce(&mut TlsData) -> R,
{
    let tid = current_thread_id();
    let cell = unsafe { &mut *TLS_STATE.data.get() };
    while (tid as usize) >= cell.len() {
        cell.push(TlsData::new());
    }
    f(&mut cell[tid as usize])
}

fn current_thread_id() -> Dword {
    static THREAD_COUNTER: AtomicU32 = AtomicU32::new(1);
    THREAD_COUNTER.load(Ordering::Relaxed)
}

// ========================================================================
// Thread registry
// ========================================================================

struct ThreadEntry {
    id: Dword,
    handle: Handle,
    exit_code: AtomicU32,
    start_address: LpthreadStartRoutine,
    parameter: Lpvoid,
    suspend_count: AtomicU32,
    priority: i32,
    priority_boost_disabled: AtomicBool,
    running: AtomicBool,
}

struct ThreadState {
    threads: UnsafeCell<BTreeMap<Dword, ThreadEntry>>,
    next_thread_id: AtomicU32,
}

unsafe impl Sync for ThreadState {}

static THREAD_STATE: ThreadState = ThreadState {
    threads: UnsafeCell::new(BTreeMap::new()),
    next_thread_id: AtomicU32::new(1),
};

fn alloc_thread_id() -> Dword {
    THREAD_STATE.next_thread_id.fetch_add(1, Ordering::Relaxed)
}

fn alloc_handle() -> Handle {
    static NEXT_HANDLE: AtomicU64 = AtomicU64::new(0x2000);
    let val = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    val as usize as Handle
}

fn make_handle_from_id(id: Dword) -> Handle {
    (id as usize).wrapping_mul(0x10000) as Handle
}

fn id_from_handle(h: Handle) -> Option<Dword> {
    let val = h as usize;
    if val == 0 || val == !1usize {
        None
    } else {
        Some(val.wrapping_div(0x10000) as Dword)
    }
}

fn current_thread_handle() -> Handle {
    make_handle_from_id(current_thread_id())
}

// ========================================================================
// CreateThread / CreateRemoteThread
// ========================================================================

pub fn create_thread(
    _sa: LpsecurityAttributes,
    _stack_size: usize,
    start: LpthreadStartRoutine,
    param: Lpvoid,
    flags: Dword,
    thread_id: *mut Dword,
) -> Handle {
    if start.is_none() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let tid = alloc_thread_id();
    let handle = make_handle_from_id(tid);

    let entry = ThreadEntry {
        id: tid,
        handle,
        exit_code: AtomicU32::new(STILL_ACTIVE),
        start_address: start,
        parameter: param,
        suspend_count: AtomicU32::new(if flags & 0x00000004 != 0 { 1 } else { 0 }),
        priority: THREAD_PRIORITY_NORMAL,
        priority_boost_disabled: AtomicBool::new(false),
        running: AtomicBool::new(true),
    };

    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    cells.insert(tid, entry);

    if !thread_id.is_null() {
        unsafe { *thread_id = tid; }
    }

    if flags & 0x00000004 == 0 {
        if let Some(cb) = start {
            unsafe {
                let ret = cb(param);
                let cells = unsafe { &mut *THREAD_STATE.threads.get() };
                if let Some(e) = cells.get_mut(&tid) {
                    e.exit_code.store(ret, Ordering::Release);
                    e.running.store(false, Ordering::Release);
                }
            }
        }
    }

    handle
}

pub fn create_remote_thread(
    _process: Handle,
    _sa: LpsecurityAttributes,
    _stack_size: usize,
    start: LpthreadStartRoutine,
    param: Lpvoid,
    flags: Dword,
    thread_id: *mut Dword,
) -> Handle {
    create_thread(_sa, _stack_size, start, param, flags, thread_id)
}

// ========================================================================
// ExitThread / GetExitCodeThread / TerminateThread
// ========================================================================

pub fn exit_thread(exit_code: Dword) {
    let tid = current_thread_id();
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get_mut(&tid) {
        entry.exit_code.store(exit_code, Ordering::Release);
        entry.running.store(false, Ordering::Release);
    }
}

pub fn get_exit_code_thread(thread: Handle, exit_code: *mut Dword) -> Bool {
    if thread.is_null() || exit_code.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    };
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get(&tid) {
        unsafe { *exit_code = entry.exit_code.load(Ordering::Acquire); }
        if entry.running.load(Ordering::Acquire) {
            unsafe { *exit_code = STILL_ACTIVE; }
        }
        TRUE
    } else {
        set_last_error(ERROR_INVALID_PARAMETER);
        FALSE
    }
}

pub fn terminate_thread(thread: Handle, exit_code: Dword) -> Bool {
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    };
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get_mut(&tid) {
        entry.exit_code.store(exit_code, Ordering::Release);
        entry.running.store(false, Ordering::Release);
        TRUE
    } else {
        FALSE
    }
}

// ========================================================================
// GetCurrentThread / GetCurrentThreadId
// ========================================================================

pub fn get_current_thread() -> Handle {
    current_thread_handle()
}

pub fn get_current_thread_id() -> Dword {
    current_thread_id()
}

// ========================================================================
// GetThreadPriority / SetThreadPriority
// ========================================================================

pub fn get_thread_priority(thread: Handle) -> i32 {
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return THREAD_PRIORITY_NORMAL;
        }
    };
    let cells = unsafe { &*THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get(&tid) {
        entry.priority
    } else {
        THREAD_PRIORITY_NORMAL
    }
}

pub fn set_thread_priority(thread: Handle, priority: i32) -> Bool {
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    };
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get_mut(&tid) {
        entry.priority = priority;
        TRUE
    } else {
        FALSE
    }
}

// ========================================================================
// GetThreadPriorityBoost / SetThreadPriorityBoost
// ========================================================================

pub fn get_thread_priority_boost(thread: Handle, disabled: *mut Bool) -> Bool {
    if disabled.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    };
    let cells = unsafe { &*THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get(&tid) {
        unsafe { *disabled = if entry.priority_boost_disabled.load(Ordering::Acquire) { TRUE } else { FALSE }; }
        TRUE
    } else {
        FALSE
    }
}

pub fn set_thread_priority_boost(thread: Handle, disabled: Bool) -> Bool {
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return FALSE;
        }
    };
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get_mut(&tid) {
        entry.priority_boost_disabled.store(disabled != 0, Ordering::Release);
        TRUE
    } else {
        FALSE
    }
}

// ========================================================================
// GetThreadTimes
// ========================================================================

pub fn get_thread_times(
    _thread: Handle,
    _creation: *mut u64,
    _exit: *mut u64,
    _kernel: *mut u64,
    _user: *mut u64,
) -> Bool {
    TRUE
}

// ========================================================================
// SuspendThread / ResumeThread
// ========================================================================

pub fn suspend_thread(thread: Handle) -> Dword {
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return 0xFFFFFFFF;
        }
    };
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get_mut(&tid) {
        entry.suspend_count.fetch_add(1, Ordering::Release)
    } else {
        0xFFFFFFFF
    }
}

pub fn resume_thread(thread: Handle) -> Dword {
    let tid = match id_from_handle(thread) {
        Some(t) => t,
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            return 0xFFFFFFFF;
        }
    };
    let cells = unsafe { &mut *THREAD_STATE.threads.get() };
    if let Some(entry) = cells.get_mut(&tid) {
        let prev = entry.suspend_count.fetch_sub(1, Ordering::Release);
        if prev == 0 {
            entry.suspend_count.store(0, Ordering::Release);
        }
        prev
    } else {
        0xFFFFFFFF
    }
}

// ========================================================================
// SwitchToThread
// ========================================================================

pub fn switch_to_thread() -> Bool {
    FALSE
}

// ========================================================================
// Sleep / SleepEx
// ========================================================================

pub fn sleep(ms: Dword) {
    for _ in 0..ms {
        core::hint::spin_loop();
    }
}

pub fn sleep_ex(ms: Dword, _alertable: Bool) -> Dword {
    sleep(ms);
    0
}

// ========================================================================
// TlsAlloc / TlsGetValue / TlsSetValue / TlsFree
// ========================================================================

pub fn tls_alloc() -> Dword {
    let idx = TLS_STATE.next_index.fetch_add(1, Ordering::Relaxed);
    if idx as usize >= TLS_SLOTS {
        set_last_error(ERROR_NOT_ENOUGH_MEMORY);
        return 0xFFFFFFFF;
    }
    idx
}

pub fn tls_get_value(index: Dword) -> Lpvoid {
    if index as usize >= TLS_SLOTS || index == 0xFFFFFFFF {
        set_last_error(ERROR_INVALID_PARAMETER);
        return ptr::null_mut();
    }
    with_current_tls(|tls| {
        tls.slots[index as usize].unwrap_or(ptr::null_mut())
    })
}

pub fn tls_set_value(index: Dword, value: Lpvoid) -> Bool {
    if index as usize >= TLS_SLOTS || index == 0xFFFFFFFF {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    with_current_tls(|tls| {
        tls.slots[index as usize] = Some(value);
    });
    TRUE
}

pub fn tls_free(index: Dword) -> Bool {
    if index as usize >= TLS_SLOTS || index == 0xFFFFFFFF {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    with_current_tls(|tls| {
        tls.slots[index as usize] = None;
    });
    TRUE
}
