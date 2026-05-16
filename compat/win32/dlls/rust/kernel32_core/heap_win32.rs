// SPDX-License-Identifier: GPL-3.0-only

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

type Bool = i32;
type Dword = u32;
type Uint = u32;
type Word = u16;
type Byte = u8;
type Handle = u64;
type Lpvoid = *mut u8;
type Lpcvoid = *const u8;
type SizeT = usize;

const TRUE: Bool = 1;
const FALSE: Bool = 0;

const HEAP_NO_SERIALIZE: Dword = 0x00000001;
const HEAP_GENERATE_EXCEPTIONS: Dword = 0x00000004;
const HEAP_ZERO_MEMORY: Dword = 0x00000008;
const HEAP_REALLOC_IN_PLACE_ONLY: Dword = 0x00000010;

const ERROR_NOT_ENOUGH_MEMORY: Dword = 8;
const ERROR_INVALID_HANDLE: Dword = 6;
const ERROR_INVALID_PARAMETER: Dword = 87;

const GMEM_FIXED: Uint = 0x0000;
const GMEM_MOVEABLE: Uint = 0x0002;
const GMEM_MODIFY: Uint = 0x0080;
const GMEM_DISCARDABLE: Uint = 0x0100;
const GMEM_DISCARDED: Uint = 0x4000;
const GMEM_INVALID_HANDLE: Uint = 0x8000;
const GMEM_DDESHARE: Uint = 0x2000;

fn set_last_error(_code: Dword) {}

const ALIGN: usize = 8;
const HDR: usize = core::mem::size_of::<BlockHdr>();

#[repr(C)]
struct BlockHdr {
    magic: u32,
    size: usize,
    next: *mut u8, /* pointer to next free block data (not header) */
}

const MAGIC: u32 = 0x48454150; /* "HEAP" */

struct Heap {
    pool: &'static mut [u8],
    bump: AtomicUsize,
    free_head: AtomicUsize,
    locked: AtomicBool,
}

unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

static mut HEAP_POOL: [u8; 2 * 1024 * 1024] = [0; 2 * 1024 * 1024];
static mut DEFAULT_HEAP: Heap = Heap {
    pool: &mut [],
    bump: AtomicUsize::new(0),
    free_head: AtomicUsize::new(0),
    locked: AtomicBool::new(false),
};
static HEAP_INIT: AtomicBool = AtomicBool::new(false);

fn align_up(x: usize) -> usize {
    (x + ALIGN - 1) & !(ALIGN - 1)
}

fn lock(h: &Heap) {
    while h.locked.swap(true, Ordering::Acquire) {
        core::hint::spin_loop();
    }
}

fn unlock(h: &Heap) {
    h.locked.store(false, Ordering::Release);
}

fn default_heap() -> &'static mut Heap {
    if !HEAP_INIT.load(Ordering::SeqCst) {
        unsafe {
            DEFAULT_HEAP.pool = &mut HEAP_POOL;
            DEFAULT_HEAP.bump = AtomicUsize::new(0);
            DEFAULT_HEAP.free_head = AtomicUsize::new(0);
            DEFAULT_HEAP.locked = AtomicBool::new(false);
        }
        HEAP_INIT.store(true, Ordering::SeqCst);
    }
    unsafe { &mut DEFAULT_HEAP }
}

fn heap_alloc_raw(h: &Heap, size: usize) -> *mut u8 {
    let needed = align_up(HDR + size);

    /* Try free list first */
    loop {
        let head = h.free_head.load(Ordering::Relaxed);
        if head == 0 {
            break;
        }
        let blk = head as *mut BlockHdr;
        unsafe {
            let blk_size = (*blk).size;
            if blk_size >= needed {
                let next = (*blk).next as usize;
                if h.free_head.compare_exchange_weak(
                    head, next, Ordering::SeqCst, Ordering::Relaxed
                ).is_ok() {
                    let data = (blk as *mut u8).add(HDR);
                    return data;
                }
            } else {
                /* Can't use, move to next - walk the list */
                break;
            }
        }
    }

    /* Bump allocate */
    loop {
        let off = h.bump.load(Ordering::Relaxed);
        let new_off = off + needed;
        if new_off > h.pool.len() {
            return core::ptr::null_mut();
        }
        if h.bump.compare_exchange_weak(off, new_off, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
            unsafe {
                let blk = h.pool.as_ptr().add(off) as *mut BlockHdr;
                core::ptr::write(blk, BlockHdr {
                    magic: MAGIC,
                    size: needed,
                    next: core::ptr::null_mut(),
                });
                let data = (blk as *mut u8).add(HDR);
                return data;
            }
        }
    }
}

pub fn get_process_heap() -> *mut u8 {
    default_heap() as *mut Heap as *mut u8
}

pub fn heap_create(_flags: Dword, _initial_size: SizeT, _max_size: SizeT) -> *mut u8 {
    get_process_heap()
}

pub fn heap_destroy(heap: *mut u8) -> Bool {
    if heap.is_null() || heap != get_process_heap() {
        set_last_error(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    TRUE
}

pub fn heap_alloc(heap: *mut u8, flags: Dword, size: SizeT) -> Lpvoid {
    if heap.is_null() || size == 0 {
        return core::ptr::null_mut();
    }
    let h = unsafe { &*(heap as *mut Heap) };

    let serial = (flags & HEAP_NO_SERIALIZE) == 0;
    if serial { lock(h); }
    let ptr = heap_alloc_raw(h, size);
    if serial { unlock(h); }

    if ptr.is_null() {
        set_last_error(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }
    if (flags & HEAP_ZERO_MEMORY) != 0 {
        unsafe { core::ptr::write_bytes(ptr, 0, size); }
    }
    ptr
}

pub fn heap_free(heap: *mut u8, _flags: Dword, ptr: Lpvoid) -> Bool {
    if heap.is_null() || ptr.is_null() {
        return FALSE;
    }
    let h = unsafe { &*(heap as *mut Heap) };

    let blk = unsafe { (ptr as *mut u8).sub(HDR) as *mut BlockHdr };
    unsafe {
        if (*blk).magic != MAGIC {
            return FALSE;
        }
        /* Add to free list */
        let blk_ptr = (blk as *mut u8).add(HDR);
        loop {
            let head = h.free_head.load(Ordering::Relaxed) as *mut u8;
            (*blk).next = head;
            if h.free_head.compare_exchange_weak(
                head as usize,
                blk_ptr as usize,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ).is_ok() {
                break;
            }
        }
    }
    TRUE
}

pub fn heap_realloc(heap: *mut u8, flags: Dword, ptr: Lpvoid, new_size: SizeT) -> Lpvoid {
    if ptr.is_null() {
        return heap_alloc(heap, flags, new_size);
    }
    if new_size == 0 {
        heap_free(heap, flags, ptr);
        return core::ptr::null_mut();
    }
    let h = unsafe { &*(heap as *mut Heap) };
    let serial = (flags & HEAP_NO_SERIALIZE) == 0;
    if serial { lock(h); }

    unsafe {
        let blk = (ptr as *mut u8).sub(HDR) as *mut BlockHdr;
        if (*blk).magic != MAGIC {
            if serial { unlock(h); }
            return core::ptr::null_mut();
        }
        let old_size = (*blk).size - HDR;
        if old_size >= new_size {
            if serial { unlock(h); }
            return ptr;
        }
    }
    if serial { unlock(h); }

    let new_ptr = heap_alloc(heap, flags & !HEAP_ZERO_MEMORY, new_size);
    if new_ptr.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        let blk = (ptr as *mut u8).sub(HDR) as *mut BlockHdr;
        let old_size = (*blk).size - HDR;
        core::ptr::copy_nonoverlapping(ptr, new_ptr, if old_size < new_size { old_size } else { new_size });
    }
    heap_free(heap, flags, ptr);
    new_ptr
}

pub fn heap_size(heap: *mut u8, _flags: Dword, ptr: Lpvoid) -> SizeT {
    if heap.is_null() || ptr.is_null() {
        return !0;
    }
    unsafe {
        let blk = (ptr as *mut u8).sub(HDR) as *mut BlockHdr;
        if (*blk).magic != MAGIC {
            return !0;
        }
        (*blk).size - HDR
    }
}

pub fn heap_validate(_heap: *mut u8, _flags: Dword, _ptr: Lpvoid) -> Bool {
    TRUE
}

pub fn heap_compact(_heap: *mut u8, _flags: Dword) -> SizeT {
    0
}

pub fn heap_lock(heap: *mut u8) -> Bool {
    if heap.is_null() { return FALSE; }
    lock(unsafe { &*(heap as *mut Heap) });
    TRUE
}

pub fn heap_unlock(heap: *mut u8) -> Bool {
    if heap.is_null() { return FALSE; }
    unlock(unsafe { &*(heap as *mut Heap) });
    TRUE
}

pub fn heap_walk(_heap: *mut u8, _entry: *mut u8) -> Bool {
    FALSE
}

pub fn get_process_heaps(count: Dword, heaps: *mut *mut u8) -> Dword {
    if heaps.is_null() && count != 0 {
        set_last_error(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if !heaps.is_null() && count >= 1 {
        unsafe { *heaps = get_process_heap(); }
    }
    1
}

/* Global/Local memory API — stubs */

pub fn global_lock(_handle: Handle) -> Lpvoid {
    core::ptr::null_mut()
}

pub fn global_unlock(handle: Handle) -> Bool {
    if (handle & 0xF) != 0 { TRUE } else { FALSE }
}

pub fn global_handle(_ptr: Lpcvoid) -> Handle {
    0
}

pub fn global_realloc(handle: Handle, _size: SizeT, _flags: Uint) -> Handle {
    handle
}

pub fn global_size(handle: Handle) -> SizeT {
    handle as SizeT
}

pub fn global_wire(handle: Handle) -> Lpvoid {
    global_lock(handle)
}

pub fn global_unwire(handle: Handle) -> Bool {
    global_unlock(handle)
}

pub fn global_fix(handle: Handle) {
    global_lock(handle);
}

pub fn global_unfix(handle: Handle) {
    global_unlock(handle);
}

pub fn global_flags(handle: Handle) -> Uint {
    handle as Uint
}

pub fn global_compact(_minfree: Dword) -> SizeT {
    0
}

pub fn local_compact(_minfree: Uint) -> SizeT {
    0
}

pub fn local_flags(handle: Handle) -> Uint {
    global_flags(handle)
}

pub fn local_handle(_ptr: Lpcvoid) -> Handle {
    0
}

pub fn local_shrink(_handle: Handle, _newsize: Uint) -> SizeT {
    0
}

pub fn local_size(handle: Handle) -> SizeT {
    handle as SizeT
}

pub fn global_memory_status(buffer: *mut u8) {
    if buffer.is_null() {
        return;
    }
    unsafe {
        let buf = buffer as *mut Dword;
        *buf.add(0) = 32;  // dwLength
        *buf.add(1) = 0;   // dwMemoryLoad
        *buf.add(2) = 256 * 1024 * 1024;  // dwTotalPhys
        *buf.add(3) = 128 * 1024 * 1024;  // dwAvailPhys
        *buf.add(4) = 512 * 1024 * 1024;  // dwTotalPageFile
        *buf.add(5) = 256 * 1024 * 1024;  // dwAvailPageFile
        *buf.add(6) = 2 * 1024 * 1024 * 1024; // dwTotalVirtual
        *buf.add(7) = 1 * 1024 * 1024 * 1024; // dwAvailVirtual
    }
}

pub fn global_memory_status_ex(buffer: *mut u8) -> Bool {
    if buffer.is_null() {
        return FALSE;
    }
    unsafe {
        let len = *(buffer as *const Dword);
        if len < 8 {
            return FALSE;
        }
        let dw = buffer as *mut Dword;
        *dw.add(1) = 0;  // dwMemoryLoad
        let qw = buffer as *mut u64;
        if len >= core::mem::size_of::<[Dword; 2]>() as Dword {
            *qw.add(1) = 256 * 1024 * 1024;
            *qw.add(2) = 128 * 1024 * 1024;
            *qw.add(3) = 512 * 1024 * 1024;
            *qw.add(4) = 256 * 1024 * 1024;
            *qw.add(5) = 2 * 1024 * 1024 * 1024;
            *qw.add(6) = 1 * 1024 * 1024 * 1024;
            *qw.add(7) = 0;
        }
    }
    TRUE
}
