// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 API Emulasyon - Birim Testleri
// Dosya Yolu         : apps/system/compat/win32/syscall_emu/api_emulator_impl.rs
// Yazar              : Ozkan Yildirim
// Oluşturma Tarihi   : 2026-05-14
// Lisans             : GPLv3
//
// Destekledigi Islemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64,
//   Alpha, VAX, HPPA, SH-4, IA-64
//
// Aciklama:
//   Win32 API Emulasyon birim testleri.
//   api_emulator.rs dosyasından bölünmüştür (KURAL 29).
//
// Bagimli Dosyalar:
//   1-) apps/system/compat/win32/syscall_emu/api_emulator.rs
// *******************************************************************

use super::*;
use crate::win_layer::shared_defs::NtStatus;

// ─── Dosya API testleri ───────────────────────────────────────────

#[test]
fn test_create_write_read_file() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_file(&mut mgr, "C:\\TEST.TXT", 0xC0000000, 0, 2, 0);
    assert_ne!(h, 0xFFFFFFFF);
    let (ok_w, written) = Win32ApiEmulator::write_file(&mut mgr, h, b"Hello OZKAN-OS");
    assert!(ok_w);
    assert_eq!(written, 14);
    Win32ApiEmulator::set_file_pointer(&mut mgr, h, 0, 0);
    let mut buf = [0u8; 64];
    let (ok_r, read) = Win32ApiEmulator::read_file(&mut mgr, h, &mut buf);
    assert!(ok_r);
    assert_eq!(read, 14);
    assert_eq!(&buf[..14], b"Hello OZKAN-OS");
}

#[test]
fn test_get_file_size() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_file(&mut mgr, "C:\\SIZE.TXT", 0x40000000, 0, 2, 0);
    Win32ApiEmulator::write_file(&mut mgr, h, b"12345");
    let size = Win32ApiEmulator::get_file_size(&mgr, h);
    assert_eq!(size, 5);
}

#[test]
fn test_delete_file() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_file(&mut mgr, "C:\\DEL.TXT", 0x40000000, 0, 2, 0);
    assert_ne!(h, 0xFFFFFFFF);
    assert!(Win32ApiEmulator::delete_file(&mut mgr, "C:\\DEL.TXT"));
    assert!(!Win32ApiEmulator::delete_file(&mut mgr, "C:\\DEL.TXT"));
}

#[test]
fn test_copy_move_file() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::create_file(&mut mgr, "C:\\SRC.TXT", 0x40000000, 0, 2, 0);
    assert!(Win32ApiEmulator::copy_file(&mut mgr, "C:\\SRC.TXT", "C:\\DST.TXT"));
    assert!(mgr.vfs.contains_key("C:\\DST.TXT"));
    assert!(Win32ApiEmulator::move_file(&mut mgr, "C:\\SRC.TXT", "C:\\MOV.TXT"));
    assert!(!mgr.vfs.contains_key("C:\\SRC.TXT"));
    assert!(mgr.vfs.contains_key("C:\\MOV.TXT"));
}

#[test]
fn test_close_handle() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_file(&mut mgr, "C:\\CLOSE.TXT", 0x40000000, 0, 2, 0);
    assert!(Win32ApiEmulator::close_handle(&mut mgr, h));
    assert!(!Win32ApiEmulator::close_handle(&mut mgr, h));
}

// ─── VirtualMemory testleri ───────────────────────────────────────

#[test]
fn test_virtual_alloc_free() {
    let mut pmm = ProcessMemoryManager::new();
    let addr = Win32ApiEmulator::virtual_alloc(&mut pmm, 0, 0x1000, 0x1000, 0x04);
    assert_ne!(addr, 0);
    assert!(Win32ApiEmulator::virtual_free(&mut pmm, addr, 0, 0x8000));
}

#[test]
fn test_virtual_protect() {
    let mut pmm = ProcessMemoryManager::new();
    let addr = Win32ApiEmulator::virtual_alloc(&mut pmm, 0, 0x1000, 0x1000, 0x04);
    assert!(Win32ApiEmulator::virtual_protect(&mut pmm, addr, 0x1000, 0x40));
    let region = Win32ApiEmulator::virtual_query(&pmm, addr).unwrap();
    assert_eq!(region.protection, 0x40);
}

#[test]
fn test_virtual_query() {
    let mut pmm = ProcessMemoryManager::new();
    let addr = Win32ApiEmulator::virtual_alloc(&mut pmm, 0, 0x2000, 0x1000, 0x02);
    let region = Win32ApiEmulator::virtual_query(&pmm, addr).unwrap();
    assert_eq!(region.base, addr);
    assert_eq!(region.protection, 0x02);
}

// ─── Registry API testleri ───────────────────────────────────────

#[test]
fn test_reg_create_open_key() {
    let mut mgr = Win32Manager::new();
    let _ = Win32ApiEmulator::reg_create_key_ex(&mut mgr, "HKLM\\SOFTWARE\\TestApp");
    let h = Win32ApiEmulator::reg_open_key_ex(&mut mgr, "HKLM\\SOFTWARE\\TestApp");
    assert_ne!(h, 0);
}

#[test]
fn test_reg_set_query_value() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::reg_create_key_ex(&mut mgr, "HKLM\\SOFTWARE\\ValTest");
    let val = RegistryValue::String(String::from("1.0"));
    assert!(Win32ApiEmulator::reg_set_value_ex(&mut mgr, "HKLM\\SOFTWARE\\ValTest", "Version", val));
    let result = Win32ApiEmulator::reg_query_value_ex(&mgr, "HKLM\\SOFTWARE\\ValTest", "Version");
    assert!(result.is_some());
    assert!(matches!(result.unwrap(), RegistryValue::String(ref s) if s == "1.0"));
}

#[test]
fn test_reg_delete_value() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::reg_create_key_ex(&mut mgr, "HKLM\\SOFTWARE\\DelVal");
    Win32ApiEmulator::reg_set_value_ex(&mut mgr, "HKLM\\SOFTWARE\\DelVal", "ToDelete", RegistryValue::String(String::from("bye")));
    assert!(Win32ApiEmulator::reg_delete_value(&mut mgr, "HKLM\\SOFTWARE\\DelVal", "ToDelete"));
    assert!(Win32ApiEmulator::reg_query_value_ex(&mgr, "HKLM\\SOFTWARE\\DelVal", "ToDelete").is_none());
}

#[test]
fn test_reg_enum_value() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::reg_create_key_ex(&mut mgr, "HKLM\\SOFTWARE\\EnumValKey");
    Win32ApiEmulator::reg_set_value_ex(&mut mgr, "HKLM\\SOFTWARE\\EnumValKey", "alpha", RegistryValue::Dword(1));
    Win32ApiEmulator::reg_set_value_ex(&mut mgr, "HKLM\\SOFTWARE\\EnumValKey", "beta", RegistryValue::Dword(2));
    let (name, _val) = Win32ApiEmulator::reg_enum_value(&mgr, "HKLM\\SOFTWARE\\EnumValKey", 0).unwrap();
    assert!(!name.is_empty());
    assert!(Win32ApiEmulator::reg_enum_value(&mgr, "HKLM\\SOFTWARE\\EnumValKey", 99).is_none());
}

#[test]
fn test_reg_enum_key() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::reg_create_key_ex(&mut mgr, "HKLM\\SOFTWARE\\EnumKeyParent");
    if let Some(key) = mgr.registry.get_mut("HKLM\\SOFTWARE\\EnumKeyParent") {
        key.add_subkey("Child1");
        key.add_subkey("Child2");
    }
    assert_eq!(Win32ApiEmulator::reg_enum_key_ex(&mgr, "HKLM\\SOFTWARE\\EnumKeyParent", 0), Some(String::from("Child1")));
    assert_eq!(Win32ApiEmulator::reg_enum_key_ex(&mgr, "HKLM\\SOFTWARE\\EnumKeyParent", 1), Some(String::from("Child2")));
    assert!(Win32ApiEmulator::reg_enum_key_ex(&mgr, "HKLM\\SOFTWARE\\EnumKeyParent", 2).is_none());
}

#[test]
fn test_reg_delete_key() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::reg_create_key_ex(&mut mgr, "HKLM\\SOFTWARE\\DelMe");
    assert!(Win32ApiEmulator::reg_delete_key(&mut mgr, "HKLM\\SOFTWARE\\DelMe"));
    assert!(!Win32ApiEmulator::reg_delete_key(&mut mgr, "HKLM\\SOFTWARE\\DelMe"));
}

// ─── Event / Mutex testleri ───────────────────────────────────────

#[test]
fn test_create_set_reset_event() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_event(&mut mgr, true, false);
    assert_ne!(h, 0);
    assert_eq!(Win32ApiEmulator::wait_for_single_object(&mgr, h, 0), 0x102); // WAIT_TIMEOUT
    assert!(Win32ApiEmulator::set_event(&mut mgr, h));
    assert_eq!(Win32ApiEmulator::wait_for_single_object(&mgr, h, 0), 0); // WAIT_OBJECT_0
    assert!(Win32ApiEmulator::reset_event(&mut mgr, h));
}

#[test]
fn test_create_release_mutex() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_mutex(&mut mgr, true);
    assert_ne!(h, 0);
    assert_eq!(Win32ApiEmulator::wait_for_single_object(&mgr, h, 0), 0x102); // owned → timeout
    assert!(Win32ApiEmulator::release_mutex(&mut mgr, h));
    assert_eq!(Win32ApiEmulator::wait_for_single_object(&mgr, h, 0), 0); // free → object_0
}

// ─── Window Message Dispatch testleri ────────────────────────────

#[test]
fn test_def_window_proc_close() {
    let mut mgr = Win32Manager::new();
    let hwnd = mgr.create_window("Button", "Test", 0, 0, 100, 50);
    assert!(mgr.window_manager.get_window(hwnd).is_some());
    let ret = Win32ApiEmulator::def_window_proc(&mut mgr, hwnd, WinMsg::Close, 0, 0);
    assert_eq!(ret, 0);
    assert!(mgr.window_manager.get_window(hwnd).is_none());
}

#[test]
fn test_dispatch_message() {
    let mut mgr = Win32Manager::new();
    let hwnd = mgr.create_window("Static", "Test", 0, 0, 200, 100);
    let msg = WindowMessage::new(hwnd, WinMsg::Paint, 0, 0);
    let ret = Win32ApiEmulator::dispatch_message(&mut mgr, &msg);
    assert_eq!(ret, 0);
}

#[test]
fn test_send_message_size() {
    let mut mgr = Win32Manager::new();
    let hwnd = mgr.create_window("Edit", "Test", 0, 0, 100, 50);
    let ret = Win32ApiEmulator::send_message(&mut mgr, hwnd, WinMsg::Size, 0, (800 << 16) | 600);
    assert_eq!(ret, 0);
    let win = mgr.window_manager.get_window(hwnd).unwrap();
    assert_eq!(win.width, 600);
    assert_eq!(win.height, 800);
}

#[test]
fn test_post_quit_message() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::post_quit_message(&mut mgr, 42);
    assert!(mgr.window_manager.messages.iter().any(|m| matches!(m.msg, WinMsg::Quit) && m.wparam == 42));
}

// ─── SEH testleri ────────────────────────────────────────────────

#[test]
fn test_seh_raise_exception_no_handler() {
    let mut mgr = Win32Manager::new();
    let ret = Win32ApiEmulator::raise_exception(&mut mgr, 0xC0000005, 0, &[]);
    assert_eq!(ret, 1); // ExceptionContinueSearch
}

#[test]
fn test_seh_raise_exception_with_handler() {
    fn handler(_r: &ExceptionRecord, _c: &mut ContextRecord) -> ExceptionDisposition {
        ExceptionDisposition::ExceptionContinueExecution
    }
    let mut mgr = Win32Manager::new();
    mgr.seh.add_handler(1, handler);
    let ret = Win32ApiEmulator::raise_exception(&mut mgr, 0xC0000005, 0, &[]);
    assert_eq!(ret, 0); // ExceptionContinueExecution
}

#[test]
fn test_seh_unhandled_filter() {
    fn filter(_r: &ExceptionRecord, _c: &mut ContextRecord) -> ExceptionDisposition {
        ExceptionDisposition::ExceptionContinueExecution
    }
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::set_unhandled_exception_filter(&mut mgr, filter);
    let ret = Win32ApiEmulator::raise_exception(&mut mgr, 0xE0000001, 0, &[]);
    assert_eq!(ret, 0);
}

// ─── System Information API testleri ─────────────────────────────

#[test]
fn test_get_system_info() {
    let info = Win32ApiEmulator::get_system_info();
    assert_eq!(info.processor_architecture, 9);
    assert_eq!(info.page_size, 4096);
    assert_eq!(info.number_of_processors, 1);
}

#[test]
fn test_get_version_ex() {
    let ver = Win32ApiEmulator::get_version_ex();
    assert_eq!(ver.major_version, 10);
    assert_eq!(ver.build_number, 19045);
}

#[test]
fn test_get_computer_user_name() {
    assert_eq!(Win32ApiEmulator::get_computer_name(), "OZKAN-PC");
    assert_eq!(Win32ApiEmulator::get_user_name(), "User");
}

// ─── Pipe / Mailslot API testleri ────────────────────────────────

#[test]
fn test_create_pipe_read_write() {
    let mut mgr = Win32Manager::new();
    let (r, w) = Win32ApiEmulator::create_pipe(&mut mgr, 1024, 1024);
    assert_ne!(r, 0);
    assert_ne!(w, 0);
    let written = Win32ApiEmulator::write_pipe(&mut mgr, w, b"pipe_test");
    assert_eq!(written, 9);
    let mut buf = [0u8; 64];
    let read = Win32ApiEmulator::read_pipe(&mut mgr, r, &mut buf);
    assert_eq!(read, 9);
    assert_eq!(&buf[..9], b"pipe_test");
}

#[test]
fn test_create_named_pipe() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_named_pipe(&mut mgr, "\\\\.\\pipe\\test", 3, 0, 1, 4096, 4096, 0);
    assert_ne!(h, 0);
    assert!(Win32ApiEmulator::connect_named_pipe(&mut mgr, h));
}

#[test]
fn test_mailslot_read_write() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_mailslot(&mut mgr, "\\\\.\\mailslot\\test", 0, 0);
    assert_ne!(h, 0);
    assert!(Win32ApiEmulator::write_mailslot(&mut mgr, "\\\\.\\mailslot\\test", b"mailslot_msg"));
    let mut buf = [0u8; 64];
    let read = Win32ApiEmulator::read_mailslot(&mut mgr, h, &mut buf);
    assert_eq!(read, 12);
    assert_eq!(&buf[..12], b"mailslot_msg");
}

// ─── Ortam Değişkeni testleri ─────────────────────────────────────

#[test]
fn test_env_get_set() {
    let mut mgr = Win32Manager::new();
    assert_eq!(Win32ApiEmulator::get_environment_variable(&mgr, "USERNAME"), "User");
    Win32ApiEmulator::set_environment_variable(&mut mgr, "MYVAR", "123");
    assert_eq!(Win32ApiEmulator::get_environment_variable(&mgr, "MYVAR"), "123");
}

#[test]
fn test_expand_environment_strings() {
    let mut mgr = Win32Manager::new();
    let expanded = Win32ApiEmulator::expand_environment_strings(&mgr, "%SYSTEMROOT%\\notepad.exe");
    assert_eq!(expanded, "C:\\WINDOWS\\notepad.exe");
}

#[test]
fn test_system_directories() {
    assert_eq!(Win32ApiEmulator::get_windows_directory(), "C:\\WINDOWS");
    assert_eq!(Win32ApiEmulator::get_system_directory(), "C:\\WINDOWS\\SYSTEM32");
    assert_eq!(Win32ApiEmulator::get_temp_path(), "C:\\TEMP");
}

// ─── Disk / Memory testleri ───────────────────────────────────────

#[test]
fn test_disk_drive_apis() {
    let (_sectors, bytes, _free, _total) = Win32ApiEmulator::get_disk_free_space("C:\\");
    assert_eq!(bytes, 512);
    assert_eq!(Win32ApiEmulator::get_drive_type("C:\\"), 3); // DRIVE_FIXED
    assert_eq!(Win32ApiEmulator::get_logical_drives(), 0x1F);
}

#[test]
fn test_global_local_alloc_free() {
    let mut mgr = Win32Manager::new();
    let g = Win32ApiEmulator::global_alloc(&mut mgr, 0, 256);
    assert_ne!(g, 0);
    assert!(mgr.global_mem.contains_key(&g));
    assert!(Win32ApiEmulator::global_free(&mut mgr, g));
    let l = Win32ApiEmulator::local_alloc(&mut mgr, 0, 128);
    assert_ne!(l, 0);
    assert!(mgr.local_mem.contains_key(&l));
    assert!(Win32ApiEmulator::local_free(&mut mgr, l));
}

#[test]
fn test_load_string_icon_cursor() {
    let mut mgr = Win32Manager::new();
    Win32ApiEmulator::load_string(&mut mgr, 100, "Hello");
    assert_eq!(mgr.strings.get(&100).unwrap(), "Hello");
    assert_eq!(Win32ApiEmulator::load_icon(&mut mgr, "APP.ICO"), 0x10001);
    assert_eq!(Win32ApiEmulator::load_cursor(&mut mgr, "ARROW"), 0x10002);
}

// ─── SyscallEmulator testleri ─────────────────────────────────────

#[test]
fn test_nt_open_file() {
    let mut mgr = Win32Manager::new();
    mgr.vfs.insert(String::from("C:\\EXIST.TXT"), alloc::vec![1, 2, 3]);
    let (status, handle) = SyscallEmulator::nt_open_file(&mut mgr, "C:\\EXIST.TXT", 0x80100080);
    assert!(matches!(status, NtStatus::Success));
    assert_ne!(handle, 0);
    let (status2, _h2) = SyscallEmulator::nt_open_file(&mut mgr, "C:\\MISSING.TXT", 0x80100080);
    assert!(!matches!(status2, NtStatus::Success));
}

#[test]
fn test_nt_query_set_information_file() {
    let mut mgr = Win32Manager::new();
    let h = Win32ApiEmulator::create_file(&mut mgr, "C:\\INFO.TXT", 0x40000000, 0, 2, 0);
    Win32ApiEmulator::write_file(&mut mgr, h, b"hello");
    let mut size = 0u64;
    let mut pos = 0u64;
    let status = SyscallEmulator::nt_query_information_file(&mgr, h, &mut size, &mut pos);
    assert!(matches!(status, NtStatus::Success));
    assert_eq!(size, 5);
    let status2 = SyscallEmulator::nt_set_information_file(&mut mgr, h, 2);
    assert!(matches!(status2, NtStatus::Success));
}

#[test]
fn test_nt_protect_virtual_memory() {
    let mut pmm = ProcessMemoryManager::new();
    let base = pmm.allocate(0x1000, 0x04);
    let status = SyscallEmulator::nt_protect_virtual_memory(&mut pmm, base, 0x1000, 0x40);
    assert!(matches!(status, NtStatus::Success));
    let mut q_base = 0u64;
    let mut q_size = 0u64;
    let mut q_prot = 0u32;
    let status2 = SyscallEmulator::nt_query_virtual_memory(&pmm, base, &mut q_base, &mut q_size, &mut q_prot);
    assert!(matches!(status2, NtStatus::Success));
    assert_eq!(q_base, base);
    assert_eq!(q_prot, 0x40);
}

#[test]
fn test_nt_create_map_section() {
    let mut mgr = Win32Manager::new();
    let mut pmm = ProcessMemoryManager::new();
    let (status, sec) = SyscallEmulator::nt_create_section(&mut mgr, 0x2000);
    assert!(matches!(status, NtStatus::Success));
    assert_ne!(sec, 0);
    let mut base = 0u64;
    let status2 = SyscallEmulator::nt_map_view_of_section(&mut mgr, sec, &mut pmm, &mut base);
    assert!(matches!(status2, NtStatus::Success));
    assert_ne!(base, 0);
    let status3 = SyscallEmulator::nt_unmap_view_of_section(&mut mgr, sec, &mut pmm);
    assert!(matches!(status3, NtStatus::Success));
}
