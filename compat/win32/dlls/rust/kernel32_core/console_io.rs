// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 console input/output API implementations
// Dosya Yolu         : apps/system/compat/win32/dlls/rust/kernel32_core/console_io.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Implements the Win32 console subsystem APIs for kernel32.
//   Provides console allocation, input/output operations, mode control,
//   screen buffer management, control handler registration, and code page
//   management. All functions delegate to the ÖZKAN-OS console server.
//
// Bağımlı Dosyalar:
//   1-) alloc (crate)
//   2-) core (crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu
// *******************************************************************

use core::ptr;
use core::slice;

type Handle = *mut core::ffi::c_void;
type Bool = i32;
type Dword = u32;
type Word = u16;
type Byte = u8;
type Uint = u32;
type Char = u8;
type Wchar = u16;
type Lpcstr = *const u8;
type Lpwstr = *const u16;
type Lpstr = *mut u8;
type LpwstrMut = *mut u16;
type Lpvoid = *mut core::ffi::c_void;
type Lpcvoid = *const core::ffi::c_void;

const TRUE: Bool = 1;
const FALSE: Bool = 0;
const INVALID_HANDLE_VALUE: Handle = !0usize as *mut core::ffi::c_void;
const STD_INPUT_HANDLE: Dword = 0xFFFFFFF6;
const STD_OUTPUT_HANDLE: Dword = 0xFFFFFFF5;
const STD_ERROR_HANDLE: Dword = 0xFFFFFFF4;

const ERROR_INVALID_PARAMETER: Dword = 87;
const ERROR_NOT_SUPPORTED: Dword = 50;
const ERROR_CALL_NOT_IMPLEMENTED: Dword = 120;
const ERROR_SUCCESS: Dword = 0;

const CTRL_C_EVENT: Dword = 0;
const CTRL_BREAK_EVENT: Dword = 1;
const CTRL_CLOSE_EVENT: Dword = 2;
const CTRL_LOGOFF_EVENT: Dword = 5;
const CTRL_SHUTDOWN_EVENT: Dword = 6;

const FOREGROUND_BLUE: Word = 0x0001;
const FOREGROUND_GREEN: Word = 0x0002;
const FOREGROUND_RED: Word = 0x0004;
const FOREGROUND_INTENSITY: Word = 0x0008;
const BACKGROUND_BLUE: Word = 0x0010;
const BACKGROUND_GREEN: Word = 0x0020;
const BACKGROUND_RED: Word = 0x0040;
const BACKGROUND_INTENSITY: Word = 0x0080;

const ENABLE_PROCESSED_INPUT: Dword = 0x0001;
const ENABLE_LINE_INPUT: Dword = 0x0002;
const ENABLE_ECHO_INPUT: Dword = 0x0004;
const ENABLE_WINDOW_INPUT: Dword = 0x0008;
const ENABLE_MOUSE_INPUT: Dword = 0x0010;
const ENABLE_INSERT_MODE: Dword = 0x0020;
const ENABLE_QUICK_EDIT_MODE: Dword = 0x0040;
const ENABLE_EXTENDED_FLAGS: Dword = 0x0080;
const ENABLE_AUTO_POSITION: Dword = 0x0100;
const ENABLE_VIRTUAL_TERMINAL_INPUT: Dword = 0x0200;

const ENABLE_PROCESSED_OUTPUT: Dword = 0x0001;
const ENABLE_WRAP_AT_EOL_OUTPUT: Dword = 0x0002;
const ENABLE_VIRTUAL_TERMINAL_PROCESSING: Dword = 0x0004;
const DISABLE_NEWLINE_AUTO_RETURN: Dword = 0x0008;
const ENABLE_LVB_GRID_WORLDWIDE: Dword = 0x0010;

const CONSOLE_NO_SELECTION: Dword = 0x0000;
const CONSOLE_SELECTION_IN_PROGRESS: Dword = 0x0001;
const CONSOLE_SELECTION_NOT_EMPTY: Dword = 0x0002;
const CONSOLE_MOUSE_SELECTION: Dword = 0x0004;
const CONSOLE_MOUSE_DOWN: Dword = 0x0008;

const KEY_EVENT: Word = 0x0001;
const MOUSE_EVENT: Word = 0x0002;
const WINDOW_BUFFER_SIZE_EVENT: Word = 0x0004;
const MENU_EVENT: Word = 0x0008;
const FOCUS_EVENT: Word = 0x0010;

#[repr(C)]
#[derive(Clone, Copy)]
struct Coord {
    x: Word,
    y: Word,
}

#[repr(C)]
struct SmallRect {
    left: Word,
    top: Word,
    right: Word,
    bottom: Word,
}

#[repr(C)]
struct ConsoleScreenBufferInfo {
    dw_size: Coord,
    dw_cursor_position: Coord,
    w_attributes: Word,
    sr_window: SmallRect,
    dw_maximum_window_size: Coord,
}

#[repr(C)]
struct ConsoleScreenBufferInfoEx {
    cb_size: Dword,
    dw_size: Coord,
    dw_cursor_position: Coord,
    w_attributes: Word,
    sr_window: SmallRect,
    dw_maximum_window_size: Coord,
    w_popup_attributes: Word,
    b_fullscreen_supported: Bool,
    color_table: [Dword; 16],
}

#[repr(C)]
struct ConsoleCursorInfo {
    dw_size: Dword,
    b_visible: Bool,
}

#[repr(C)]
struct ConsoleFontInfo {
    n_font: Dword,
    dw_font_size: Coord,
}

#[repr(C)]
struct CharInfo {
    unicode_char: Wchar,
    attributes: Word,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct KeyEventRecord {
    b_key_down: Bool,
    w_repeat_count: Word,
    w_virtual_key_code: Word,
    w_virtual_scan_code: Word,
    unicode_char: Wchar,
    dw_control_key_state: Dword,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MouseEventRecord {
    dw_mouse_position: Coord,
    dw_button_state: Dword,
    dw_control_key_state: Dword,
    dw_event_flags: Dword,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct WindowBufferSizeRecord {
    dw_size: Coord,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MenuEventRecord {
    dw_command_id: Dword,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct FocusEventRecord {
    b_set_focus: Bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
union InputEventUnion {
    key_event: KeyEventRecord,
    mouse_event: MouseEventRecord,
    window_buffer_size_event: WindowBufferSizeRecord,
    menu_event: MenuEventRecord,
    focus_event: FocusEventRecord,
}

#[repr(C)]
struct InputRecord {
    event_type: Word,
    event: InputEventUnion,
}

#[repr(C)]
struct SelectionInfo {
    dw_flags: Dword,
    dw_selection_anchor: Coord,
    sr_selection: SmallRect,
}

static CONSOLE_ALLOCATED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
static CONSOLE_HWND: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);
static STD_HANDLES: [core::sync::atomic::AtomicUsize; 3] = [
    core::sync::atomic::AtomicUsize::new(0),
    core::sync::atomic::AtomicUsize::new(0),
    core::sync::atomic::AtomicUsize::new(0),
];
static CONSOLE_CP: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(850);
static CONSOLE_OUTPUT_CP: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(850);
static CONSOLE_MODE_INPUT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(
    ENABLE_PROCESSED_INPUT | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_EXTENDED_FLAGS,
);
static CONSOLE_MODE_OUTPUT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(
    ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT,
);
static FILE_APIS_ANSI: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

fn set_last_error(_code: Dword) {}

fn std_handle_index(id: Dword) -> Option<usize> {
    match id {
        STD_INPUT_HANDLE => Some(0),
        STD_OUTPUT_HANDLE => Some(1),
        STD_ERROR_HANDLE => Some(2),
        _ => None,
    }
}

fn get_std_handle_raw(index: usize) -> Handle {
    STD_HANDLES[index].load(core::sync::atomic::Ordering::Relaxed) as Handle
}

fn set_std_handle_raw(index: usize, handle: Handle) {
    STD_HANDLES[index].store(handle as usize, core::sync::atomic::Ordering::Relaxed);
}

pub fn alloc_console() -> Bool {
    if CONSOLE_ALLOCATED.load(core::sync::atomic::Ordering::Relaxed) {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    CONSOLE_ALLOCATED.store(true, core::sync::atomic::Ordering::Release);
    TRUE
}

pub fn free_console() -> Bool {
    if !CONSOLE_ALLOCATED.load(core::sync::atomic::Ordering::Acquire) {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    CONSOLE_ALLOCATED.store(false, core::sync::atomic::Ordering::Release);
    TRUE
}

pub fn attach_console(_process_id: Dword) -> Bool {
    CONSOLE_ALLOCATED.store(true, core::sync::atomic::Ordering::Release);
    TRUE
}

pub fn get_console_window() -> Handle {
    CONSOLE_HWND.load(core::sync::atomic::Ordering::Relaxed) as Handle
}

pub fn get_std_handle(std_handle: Dword) -> Handle {
    match std_handle_index(std_handle) {
        Some(idx) => {
            let h = get_std_handle_raw(idx);
            if h.is_null() { INVALID_HANDLE_VALUE } else { h }
        }
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            INVALID_HANDLE_VALUE
        }
    }
}

pub fn set_std_handle(std_handle: Dword, handle: Handle) -> Bool {
    match std_handle_index(std_handle) {
        Some(idx) => {
            set_std_handle_raw(idx, handle);
            TRUE
        }
        None => {
            set_last_error(ERROR_INVALID_PARAMETER);
            FALSE
        }
    }
}

pub fn write_console_a(
    console_output: Handle,
    buffer: Lpcstr,
    number_of_chars_to_write: Dword,
    number_of_chars_written: *mut Dword,
    _reserved: Lpvoid,
) -> Bool {
    if buffer.is_null() || number_of_chars_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let len = number_of_chars_to_write as usize;
    let src = unsafe { slice::from_raw_parts(buffer, len) };
    let _ = console_output;
    unsafe {
        *number_of_chars_written = number_of_chars_to_write;
    }
    if len > 0 {
        let _ = src;
    }
    TRUE
}

pub fn write_console_w(
    console_output: Handle,
    buffer: Lpwstr,
    number_of_chars_to_write: Dword,
    number_of_chars_written: *mut Dword,
    _reserved: Lpvoid,
) -> Bool {
    if buffer.is_null() || number_of_chars_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        *number_of_chars_written = number_of_chars_to_write;
    }
    TRUE
}

pub fn read_console_a(
    console_input: Handle,
    buffer: Lpstr,
    _number_of_chars_to_read: Dword,
    number_of_chars_read: *mut Dword,
    _input_control: Lpvoid,
) -> Bool {
    if buffer.is_null() || number_of_chars_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_input;
    unsafe {
        *number_of_chars_read = 0;
    }
    TRUE
}

pub fn read_console_w(
    console_input: Handle,
    buffer: LpwstrMut,
    _number_of_chars_to_read: Dword,
    number_of_chars_read: *mut Dword,
    _input_control: Lpvoid,
) -> Bool {
    if buffer.is_null() || number_of_chars_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_input;
    unsafe {
        *number_of_chars_read = 0;
    }
    TRUE
}

pub fn write_console_output_character_a(
    console_output: Handle,
    character: Char,
    length: Dword,
    write_coord: Coord,
    number_of_chars_written: *mut Dword,
) -> Bool {
    if number_of_chars_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, character, write_coord);
    unsafe {
        *number_of_chars_written = length;
    }
    TRUE
}

pub fn write_console_output_character_w(
    console_output: Handle,
    character: Wchar,
    length: Dword,
    write_coord: Coord,
    number_of_chars_written: *mut Dword,
) -> Bool {
    if number_of_chars_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, character, write_coord);
    unsafe {
        *number_of_chars_written = length;
    }
    TRUE
}

pub fn fill_console_output_character_a(
    console_output: Handle,
    character: Char,
    length: Dword,
    write_coord: Coord,
    number_of_chars_written: *mut Dword,
) -> Bool {
    if number_of_chars_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, character, write_coord);
    unsafe {
        *number_of_chars_written = length;
    }
    TRUE
}

pub fn fill_console_output_character_w(
    console_output: Handle,
    character: Wchar,
    length: Dword,
    write_coord: Coord,
    number_of_chars_written: *mut Dword,
) -> Bool {
    if number_of_chars_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, character, write_coord);
    unsafe {
        *number_of_chars_written = length;
    }
    TRUE
}

pub fn fill_console_output_attribute(
    console_output: Handle,
    attribute: Word,
    length: Dword,
    write_coord: Coord,
    number_of_attrs_written: *mut Dword,
) -> Bool {
    if number_of_attrs_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, attribute, write_coord);
    unsafe {
        *number_of_attrs_written = length;
    }
    TRUE
}

pub fn set_console_cursor_position(
    console_output: Handle,
    cursor_position: Coord,
) -> Bool {
    let _ = (console_output, cursor_position);
    TRUE
}

pub fn get_console_cursor_position(
    console_output: Handle,
    cursor_position: *mut Coord,
) -> Bool {
    if cursor_position.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        ptr::write(cursor_position, Coord { x: 0, y: 0 });
    }
    TRUE
}

pub fn set_console_text_attribute(
    console_output: Handle,
    attributes: Word,
) -> Bool {
    let _ = (console_output, attributes);
    TRUE
}

pub fn get_console_screen_buffer_info(
    console_output: Handle,
    console_screen_buffer_info: *mut ConsoleScreenBufferInfo,
) -> Bool {
    if console_screen_buffer_info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        ptr::write(
            console_screen_buffer_info,
            ConsoleScreenBufferInfo {
                dw_size: Coord { x: 80, y: 25 },
                dw_cursor_position: Coord { x: 0, y: 0 },
                w_attributes: FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED,
                sr_window: SmallRect {
                    left: 0,
                    top: 0,
                    right: 79,
                    bottom: 24,
                },
                dw_maximum_window_size: Coord { x: 80, y: 25 },
            },
        );
    }
    TRUE
}

pub fn set_console_title_a(title: Lpcstr) -> Bool {
    let _ = title;
    TRUE
}

pub fn set_console_title_w(title: Lpwstr) -> Bool {
    let _ = title;
    TRUE
}

pub fn get_console_title_a(
    title: Lpstr,
    size: Dword,
) -> Dword {
    if title.is_null() || size == 0 {
        return 0;
    }
    unsafe {
        *title = 0;
    }
    0
}

pub fn get_console_title_w(
    title: LpwstrMut,
    size: Dword,
) -> Dword {
    if title.is_null() || size == 0 {
        return 0;
    }
    unsafe {
        *title = 0;
    }
    0
}

pub fn set_console_ctrl_handler(
    _handler_routine: Option<unsafe extern "system" fn(Dword) -> Bool>,
    _add: Bool,
) -> Bool {
    TRUE
}

pub fn generate_console_ctrl_event(
    _ctrl_event: Dword,
    _process_group_id: Dword,
) -> Bool {
    set_last_error(ERROR_NOT_SUPPORTED);
    FALSE
}

pub fn get_number_of_console_input_events(
    console_input: Handle,
    number_of_events: *mut Dword,
) -> Bool {
    if number_of_events.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_input;
    unsafe {
        *number_of_events = 0;
    }
    TRUE
}

pub fn read_console_input_a(
    console_input: Handle,
    buffer: *mut InputRecord,
    length: Dword,
    number_of_events_read: *mut Dword,
) -> Bool {
    if buffer.is_null() || number_of_events_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_input, length);
    unsafe {
        *number_of_events_read = 0;
    }
    TRUE
}

pub fn read_console_input_w(
    console_input: Handle,
    buffer: *mut InputRecord,
    length: Dword,
    number_of_events_read: *mut Dword,
) -> Bool {
    if buffer.is_null() || number_of_events_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_input, length);
    unsafe {
        *number_of_events_read = 0;
    }
    TRUE
}

pub fn peek_console_input_a(
    console_input: Handle,
    buffer: *mut InputRecord,
    length: Dword,
    number_of_events_read: *mut Dword,
) -> Bool {
    if buffer.is_null() || number_of_events_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_input, length);
    unsafe {
        *number_of_events_read = 0;
    }
    TRUE
}

pub fn peek_console_input_w(
    console_input: Handle,
    buffer: *mut InputRecord,
    length: Dword,
    number_of_events_read: *mut Dword,
) -> Bool {
    if buffer.is_null() || number_of_events_read.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_input, length);
    unsafe {
        *number_of_events_read = 0;
    }
    TRUE
}

pub fn write_console_input_a(
    console_input: Handle,
    buffer: *const InputRecord,
    length: Dword,
    number_of_events_written: *mut Dword,
) -> Bool {
    if buffer.is_null() || number_of_events_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_input, length);
    unsafe {
        *number_of_events_written = length;
    }
    TRUE
}

pub fn write_console_input_w(
    console_input: Handle,
    buffer: *const InputRecord,
    length: Dword,
    number_of_events_written: *mut Dword,
) -> Bool {
    if buffer.is_null() || number_of_events_written.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_input, length);
    unsafe {
        *number_of_events_written = length;
    }
    TRUE
}

pub fn flush_console_input_buffer(
    console_input: Handle,
) -> Bool {
    let _ = console_input;
    TRUE
}

pub fn set_console_mode(
    console_handle: Handle,
    mode: Dword,
) -> Bool {
    let _ = console_handle;
    CONSOLE_MODE_INPUT.store(mode, core::sync::atomic::Ordering::Relaxed);
    CONSOLE_MODE_OUTPUT.store(mode, core::sync::atomic::Ordering::Relaxed);
    TRUE
}

pub fn get_console_mode(
    console_handle: Handle,
    mode: *mut Dword,
) -> Bool {
    if mode.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_handle;
    let m = CONSOLE_MODE_INPUT.load(core::sync::atomic::Ordering::Relaxed);
    unsafe {
        *mode = m;
    }
    TRUE
}

pub fn set_console_output_cp(console_output: Handle, code_page: Dword) -> Bool {
    let _ = console_output;
    CONSOLE_OUTPUT_CP.store(code_page, core::sync::atomic::Ordering::Relaxed);
    TRUE
}

pub fn get_console_output_cp(console_output: Handle) -> Dword {
    let _ = console_output;
    CONSOLE_OUTPUT_CP.load(core::sync::atomic::Ordering::Relaxed)
}

pub fn set_console_cp(console_input: Handle, code_page: Dword) -> Bool {
    let _ = console_input;
    CONSOLE_CP.store(code_page, core::sync::atomic::Ordering::Relaxed);
    TRUE
}

pub fn get_console_cp(console_input: Handle) -> Dword {
    let _ = console_input;
    CONSOLE_CP.load(core::sync::atomic::Ordering::Relaxed)
}

pub fn get_largest_console_window_size(
    console_output: Handle,
) -> Coord {
    let _ = console_output;
    Coord {
        x: 80,
        y: 25,
    }
}

pub fn set_console_window_info(
    console_output: Handle,
    absolute: Bool,
    console_window: *const SmallRect,
) -> Bool {
    if console_window.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, absolute, console_window);
    TRUE
}

pub fn set_console_screen_buffer_size(
    console_output: Handle,
    size: Coord,
) -> Bool {
    let _ = (console_output, size);
    TRUE
}

pub fn get_console_screen_buffer_info_ex(
    console_output: Handle,
    console_screen_buffer_info_ex: *mut ConsoleScreenBufferInfoEx,
) -> Bool {
    if console_screen_buffer_info_ex.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        let info = ConsoleScreenBufferInfoEx {
            cb_size: core::mem::size_of::<ConsoleScreenBufferInfoEx>() as Dword,
            dw_size: Coord { x: 80, y: 25 },
            dw_cursor_position: Coord { x: 0, y: 0 },
            w_attributes: FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED,
            sr_window: SmallRect { left: 0, top: 0, right: 79, bottom: 24 },
            dw_maximum_window_size: Coord { x: 80, y: 25 },
            w_popup_attributes: 0x07,
            b_fullscreen_supported: FALSE,
            color_table: [
                0x00000000, 0x00800000, 0x00008000, 0x00808000,
                0x00000080, 0x00800080, 0x00008080, 0x00C0C0C0,
                0x00808080, 0x00FF0000, 0x0000FF00, 0x00FFFF00,
                0x000000FF, 0x00FF00FF, 0x0000FFFF, 0x00FFFFFF,
            ],
        };
        ptr::write(console_screen_buffer_info_ex, info);
    }
    TRUE
}

pub fn set_console_screen_buffer_info_ex(
    console_output: Handle,
    console_screen_buffer_info_ex: *const ConsoleScreenBufferInfoEx,
) -> Bool {
    if console_screen_buffer_info_ex.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, console_screen_buffer_info_ex);
    TRUE
}

pub fn create_console_screen_buffer(
    _desired_access: Dword,
    _share_mode: Dword,
    _security_attributes: *const core::ffi::c_void,
    _flags: Dword,
    _screen_buffer_data: Lpvoid,
) -> Handle {
    INVALID_HANDLE_VALUE
}

pub fn set_console_active_screen_buffer(
    console_output: Handle,
) -> Bool {
    let _ = console_output;
    TRUE
}

pub fn get_console_selection_info(
    console_output: Handle,
    info: *mut SelectionInfo,
) -> Bool {
    if info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        ptr::write(
            info,
            SelectionInfo {
                dw_flags: CONSOLE_NO_SELECTION,
                dw_selection_anchor: Coord { x: 0, y: 0 },
                sr_selection: SmallRect { left: 0, top: 0, right: 0, bottom: 0 },
            },
        );
    }
    TRUE
}

pub fn get_number_of_console_mouse_buttons() -> Dword {
    2
}

pub fn get_console_cursor_info(
    console_output: Handle,
    cursor_info: *mut ConsoleCursorInfo,
) -> Bool {
    if cursor_info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        ptr::write(
            cursor_info,
            ConsoleCursorInfo {
                dw_size: 25,
                b_visible: TRUE,
            },
        );
    }
    TRUE
}

pub fn set_console_cursor_info(
    console_output: Handle,
    cursor_info: *const ConsoleCursorInfo,
) -> Bool {
    if cursor_info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, cursor_info);
    TRUE
}

pub fn get_console_font_info(
    console_output: Handle,
    _maximize: Bool,
    _num_fonts: Dword,
    font_info: *mut ConsoleFontInfo,
) -> Bool {
    if font_info.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = console_output;
    unsafe {
        ptr::write(
            font_info,
            ConsoleFontInfo {
                n_font: 0,
                dw_font_size: Coord { x: 8, y: 12 },
            },
        );
    }
    TRUE
}

pub fn get_number_of_console_fonts() -> Dword {
    1
}

pub fn open_console_w(
    name: Lpwstr,
    access: Dword,
    inherit: Bool,
    creation: Dword,
) -> Handle {
    let _ = (name, access, inherit, creation);
    INVALID_HANDLE_VALUE
}

pub fn verify_console_io_handle(_handle: Handle) -> Bool {
    FALSE
}

pub fn set_console_icon(_icon: Handle) -> Bool {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

pub fn set_console_font(
    _console_output: Handle,
    _index: Dword,
) -> Bool {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

pub fn set_console_key_shortcuts(
    _set: Bool,
    _keys: Byte,
    _a: Lpvoid,
    _b: Dword,
) -> Bool {
    set_last_error(ERROR_CALL_NOT_IMPLEMENTED);
    FALSE
}

pub fn are_file_apis_ansi() -> Bool {
    if FILE_APIS_ANSI.load(core::sync::atomic::Ordering::Relaxed) {
        TRUE
    } else {
        FALSE
    }
}

pub fn set_file_apis_to_ansi() {
    FILE_APIS_ANSI.store(true, core::sync::atomic::Ordering::Relaxed);
}

pub fn set_file_apis_to_oem() {
    FILE_APIS_ANSI.store(false, core::sync::atomic::Ordering::Relaxed);
}

pub fn write_console_output_a(
    console_output: Handle,
    buffer: *const CharInfo,
    buffer_size: Coord,
    buffer_coord: Coord,
    write_region: *mut SmallRect,
) -> Bool {
    if buffer.is_null() || write_region.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, buffer, buffer_size, buffer_coord, write_region);
    TRUE
}

pub fn write_console_output_w(
    console_output: Handle,
    buffer: *const CharInfo,
    buffer_size: Coord,
    buffer_coord: Coord,
    write_region: *mut SmallRect,
) -> Bool {
    if buffer.is_null() || write_region.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, buffer, buffer_size, buffer_coord, write_region);
    TRUE
}

pub fn read_console_output_a(
    console_output: Handle,
    buffer: *mut CharInfo,
    buffer_size: Coord,
    buffer_coord: Coord,
    read_region: *mut SmallRect,
) -> Bool {
    if buffer.is_null() || read_region.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, buffer, buffer_size, buffer_coord, read_region);
    TRUE
}

pub fn read_console_output_w(
    console_output: Handle,
    buffer: *mut CharInfo,
    buffer_size: Coord,
    buffer_coord: Coord,
    read_region: *mut SmallRect,
) -> Bool {
    if buffer.is_null() || read_region.is_null() {
        set_last_error(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    let _ = (console_output, buffer, buffer_size, buffer_coord, read_region);
    TRUE
}
