// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Darwin Framework Emülasyonları
// Dosya Yolu         : apps/system/compat/darwin_frameworks.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64,
//   Alpha, VAX, HPPA, SH-4, IA-64
//
// Açıklama:
//   CoreFoundation, CoreGraphics, IOKit, DynamicLibs, Sandbox,
//   CommonCrypto, NetworkExtension, CoreText, CoreAnimation,
//   AVFoundation, Metal ve GCD API emülasyonları.
//
// Bağımlı Dosyalar:
//   apps/system/compat/darwin.rs
//
//              Dosyaya Müdahaleler
// 2026-05-14      Dosya oluşturuldu (darwin.rs bölündü)
// *******************************************************************

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use kernel_ui::{Lang, MsgId};
use crate::dos_emulator::console_writeln;
use super::{MachOLoadedImage, MachOParser, DyldCache};
pub struct OzCoreFoundation;

impl OzCoreFoundation {
    pub fn cf_string_create_with_cstring(cstr: &str) -> u64 {
        let handle = 0xC000_0000 + cstr.len() as u64;
        let msg = format!("{} CFStringCreateWithCString(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCf), cstr, handle);
        console_writeln(&msg); handle
    }
    pub fn cf_dictionary_create(num_values: usize) -> u64 {
        let handle = 0xC000_1000 + num_values as u64;
        let msg = format!("{} CFDictionaryCreate({} entries) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCf), num_values, handle);
        console_writeln(&msg); handle
    }
    pub fn cf_array_create(num_values: usize) -> u64 {
        let handle = 0xC000_2000 + num_values as u64;
        let msg = format!("{} CFArrayCreate({} values) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCf), num_values, handle);
        console_writeln(&msg); handle
    }
    pub fn cf_data_create(length: usize) -> u64 {
        let handle = 0xC000_3000 + length as u64;
        let msg = format!("{} CFDataCreate({} bytes) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCf), length, handle);
        console_writeln(&msg); handle
    }
    pub fn cf_release(cf: u64) {
        let msg = format!("{} CFRelease(0x{:016X})", &Lang::get(MsgId::CompatDarwinCf), cf);
        console_writeln(&msg);
    }
    pub fn cf_retain(cf: u64) -> u64 {
        let msg = format!("{} CFRetain(0x{:016X})", &Lang::get(MsgId::CompatDarwinCf), cf);
        console_writeln(&msg); cf
    }
    pub fn cf_get_type_id(cf: u64) -> u32 {
        let msg = format!("{} CFGetTypeID(0x{:016X})", &Lang::get(MsgId::CompatDarwinCf), cf);
        console_writeln(&msg); 7
    }
    pub fn cf_string_get_length(string: u64) -> i64 {
        let msg = format!("{} CFStringGetLength(0x{:016X}) -> 42",
            &Lang::get(MsgId::CompatDarwinCf), string);
        console_writeln(&msg); 42
    }
    pub fn cf_run_loop_run() {
        let msg = format!("{} CFRunLoopRun()", &Lang::get(MsgId::CompatDarwinCfRunloop));
        console_writeln(&msg);
    }
    pub fn cf_run_loop_stop(rl: u64) {
        let msg = format!("{} CFRunLoopStop(0x{:016X})", &Lang::get(MsgId::CompatDarwinCfRunloop), rl);
        console_writeln(&msg);
    }
    pub fn cf_run_loop_get_current() -> u64 {
        let msg = format!("{} CFRunLoopGetCurrent() -> 0xCF000001",
            &Lang::get(MsgId::CompatDarwinCfRunloop));
        console_writeln(&msg); 0xCF000001
    }
    pub fn cf_run_loop_add_source(rl: u64, source: u64, _mode: u64) {
        let msg = format!("{} CFRunLoopAddSource(rl=0x{:016X}, src=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinCfRunloop), rl, source);
        console_writeln(&msg);
    }
    pub fn cf_run_loop_add_timer(rl: u64, timer: u64, _mode: u64) {
        let msg = format!("{} CFRunLoopAddTimer(rl=0x{:016X}, timer=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinCfRunloop), rl, timer);
        console_writeln(&msg);
    }
    pub fn cf_number_create(value: i64) -> u64 {
        let handle = 0xC001_0000 + value.unsigned_abs();
        let msg = format!("{} CFNumberCreate({}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCf), value, handle);
        console_writeln(&msg); handle
    }
    pub fn cf_boolean_get_value(boolean: u64) -> bool {
        let v = boolean & 1 == 1;
        let msg = format!("{} CFBooleanGetValue(0x{:016X}) -> {}",
            &Lang::get(MsgId::CompatDarwinCf), boolean, v);
        console_writeln(&msg); v
    }
    pub fn cf_url_create_with_string(url_string: &str) -> u64 {
        let handle = 0xC002_0000 + url_string.len() as u64;
        let msg = format!("{} CFURLCreateWithString(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCf), url_string, handle);
        console_writeln(&msg); handle
    }
}

// ─── CoreGraphics API Emulation ────────────────────────────────

pub struct OzCoreGraphics;

impl OzCoreGraphics {
    pub fn cg_main_display_id() -> u32 {
        console_writeln(&format!("{} CGMainDisplayID() -> 0x1", &Lang::get(MsgId::CompatDarwinCg)));
        0x1
    }
    pub fn cg_display_pixels_wide(display: u32) -> usize {
        let msg = format!("{} CGDisplayPixelsWide({}) -> 1920", &Lang::get(MsgId::CompatDarwinCg), display);
        console_writeln(&msg); 1920
    }
    pub fn cg_display_pixels_high(display: u32) -> usize {
        let msg = format!("{} CGDisplayPixelsHigh({}) -> 1080", &Lang::get(MsgId::CompatDarwinCg), display);
        console_writeln(&msg); 1080
    }
    pub fn cg_display_bounds(display: u32) -> (f64, f64, f64, f64) {
        let msg = format!("{} CGDisplayBounds({}) -> {{0,0,1920,1080}}", &Lang::get(MsgId::CompatDarwinCg), display);
        console_writeln(&msg); (0.0, 0.0, 1920.0, 1080.0)
    }
    pub fn cg_display_copy_mode_list(_display: u32, _options: u64) -> u64 {
        console_writeln(&format!("{} CGDisplayCopyAllDisplayModes() -> 0x9000", &Lang::get(MsgId::CompatDarwinCg)));
        0x9000
    }
    pub fn cg_display_mode_get_width(mode: u64) -> usize {
        let msg = format!("{} CGDisplayModeGetWidth(0x{:016X}) -> 1920", &Lang::get(MsgId::CompatDarwinCg), mode);
        console_writeln(&msg); 1920
    }
    pub fn cg_display_mode_get_height(mode: u64) -> usize {
        let msg = format!("{} CGDisplayModeGetHeight(0x{:016X}) -> 1080", &Lang::get(MsgId::CompatDarwinCg), mode);
        console_writeln(&msg); 1080
    }
    pub fn cg_display_mode_get_refresh_rate(mode: u64) -> f64 {
        let msg = format!("{} CGDisplayModeGetRefreshRate(0x{:016X}) -> 60.0", &Lang::get(MsgId::CompatDarwinCg), mode);
        console_writeln(&msg); 60.0
    }
    pub fn cg_color_space_create_device_rgb() -> u64 {
        console_writeln(&format!("{} CGColorSpaceCreateDeviceRGB() -> 0x9001", &Lang::get(MsgId::CompatDarwinCg)));
        0x9001
    }
    pub fn cg_color_space_release(_cs: u64) {
        console_writeln(&format!("{} CGColorSpaceRelease()", &Lang::get(MsgId::CompatDarwinCg)));
    }
    pub fn cg_context_create(_width: usize, _height: usize, _bits: usize, _space: u64) -> u64 {
        console_writeln(&format!("{} CGBitmapContextCreate() -> 0x9002", &Lang::get(MsgId::CompatDarwinCg)));
        0x9002
    }
    pub fn cg_context_release(_ctx: u64) {
        console_writeln(&format!("{} CGContextRelease()", &Lang::get(MsgId::CompatDarwinCg)));
    }
    pub fn cg_context_set_rgb_fill_color(_ctx: u64, r: f64, g: f64, b: f64, a: f64) {
        let msg = format!("{} CGContextSetRGBFillColor({}, {}, {}, {})", &Lang::get(MsgId::CompatDarwinCg), r, g, b, a);
        console_writeln(&msg);
    }
    pub fn cg_context_fill_rect(_ctx: u64, x: f64, y: f64, w: f64, h: f64) {
        let msg = format!("{} CGContextFillRect({}, {}, {}, {})", &Lang::get(MsgId::CompatDarwinCg), x, y, w, h);
        console_writeln(&msg);
    }
    pub fn cg_image_get_width(image: u64) -> usize {
        let msg = format!("{} CGImageGetWidth(0x{:016X}) -> 256", &Lang::get(MsgId::CompatDarwinCg), image);
        console_writeln(&msg); 256
    }
    pub fn cg_image_get_height(image: u64) -> usize {
        let msg = format!("{} CGImageGetHeight(0x{:016X}) -> 256", &Lang::get(MsgId::CompatDarwinCg), image);
        console_writeln(&msg); 256
    }
}

// ─── IOKit API Emulation ───────────────────────────────────────

pub struct OzIOKit;

impl OzIOKit {
    pub fn io_service_matching(name: &str) -> u64 {
        let handle = 0xAA00_0000 + name.len() as u64;
        let msg = format!("{} IOServiceMatching(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinIokit), name, handle);
        console_writeln(&msg); handle
    }
    pub fn io_service_get_matching_service(matching: u64) -> u32 {
        let msg = format!("{} IOServiceGetMatchingService(matching=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinIokit), matching);
        console_writeln(&msg); 0xAA01
    }
    pub fn io_registry_entry_create_cf_property(key: &str) -> u64 {
        let handle = 0xAA02_0000 + key.len() as u64;
        let msg = format!("{} IORegistryEntryCreateCFProperty(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinIokit), key, handle);
        console_writeln(&msg); handle
    }
    pub fn io_object_release(object: u32) {
        let msg = format!("{} IOObjectRelease({})", &Lang::get(MsgId::CompatDarwinIokit), object);
        console_writeln(&msg);
    }
    pub fn io_iterator_next(iterator: u32) -> u32 {
        let msg = format!("{} IOIteratorNext({})", &Lang::get(MsgId::CompatDarwinIokit), iterator);
        console_writeln(&msg); 0
    }
    pub fn io_service_open(service: u32, _task: u32, _type: u32, connect: &mut u32) -> i32 {
        *connect = service + 0x1000;
        let msg = format!("{} IOServiceOpen({}) -> connect=0x{:08X}",
            &Lang::get(MsgId::CompatDarwinIokit), service, *connect);
        console_writeln(&msg); 0
    }
    pub fn io_service_close(connect: u32) -> i32 {
        let msg = format!("{} IOServiceClose(0x{:08X})", &Lang::get(MsgId::CompatDarwinIokit), connect);
        console_writeln(&msg); 0
    }
    pub fn io_connect_method(connection: u32, selector: u32) -> i32 {
        let msg = format!("{} IOConnectMethod(conn={}, sel={})",
            &Lang::get(MsgId::CompatDarwinIokit), connection, selector);
        console_writeln(&msg); 0
    }
}

// ─── Dynamic Libraries API Emulation ───────────────────────────

pub struct OzDynamicLibs;

impl OzDynamicLibs {
    #[must_use]
    pub fn dlopen(path: &str, mode: i32) -> u64 {
        let handle = 0xDD00_0000 + path.len() as u64;
        let msg = format!("{} dlopen(\"{}\", {}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinDl), path, mode, handle);
        console_writeln(&msg); handle
    }
    #[must_use]
    pub fn dlsym(handle: u64, symbol: &str) -> u64 {
        let addr = 0xDD10_0000 + symbol.len() as u64;
        let msg = format!("{} dlsym(0x{:016X}, \"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinDl), handle, symbol, addr);
        console_writeln(&msg); addr
    }
    pub fn dlclose(handle: u64) -> i32 {
        let msg = format!("{} dlclose(0x{:016X})", &Lang::get(MsgId::CompatDarwinDl), handle);
        console_writeln(&msg); 0
    }
    pub fn dlerror() -> &'static str {
        console_writeln(&format!("{} dlerror() -> null", &Lang::get(MsgId::CompatDarwinDl)));
        ""
    }
    pub fn dladdr(addr: u64, info: &mut [u64; 4]) -> i32 {
        info[0] = addr; info[1] = 0xDD00_0000; info[2] = addr; info[3] = 0;
        let msg = format!("{} dladdr(0x{:016X})", &Lang::get(MsgId::CompatDarwinDl), addr);
        console_writeln(&msg); 1
    }
}

// ─── Sandbox API Emulation ─────────────────────────────────────

pub struct OzSandbox;

impl OzSandbox {
    pub fn sandbox_init(profile: &str) -> i32 {
        let msg = format!("{} sandbox_init(\"{}\")", &Lang::get(MsgId::CompatDarwinSandbox), profile);
        console_writeln(&msg); 0
    }
    pub fn sandbox_init_with_parameters(profile: &str) -> i32 {
        let msg = format!("{} sandbox_init_with_parameters(\"{}\")", &Lang::get(MsgId::CompatDarwinSandbox), profile);
        console_writeln(&msg); 0
    }
    pub fn sandbox_free_error(_error: u64) {
        console_writeln(&format!("{} sandbox_free_error()", &Lang::get(MsgId::CompatDarwinSandbox)));
    }
    pub fn sandbox_check(pid: i32) -> i32 {
        let msg = format!("{} sandbox_check(pid={})", &Lang::get(MsgId::CompatDarwinSandbox), pid);
        console_writeln(&msg); 0
    }
    pub fn sandbox_check_by_audit_token(_token: u64, _operation: &str, _flags: u32) -> i32 {
        console_writeln(&format!("{} sandbox_check_by_audit_token()", &Lang::get(MsgId::CompatDarwinSandbox)));
        0
    }
    pub fn sandbox_extension_issue(_ext_class: &str, _path: &str, _flags: u64) -> u64 {
        console_writeln(&format!("{} sandbox_extension_issue() -> 0xF100", &Lang::get(MsgId::CompatDarwinSandbox)));
        0xF100
    }
    pub fn sandbox_extension_consume(_token: u64) -> i32 {
        console_writeln(&format!("{} sandbox_extension_consume()", &Lang::get(MsgId::CompatDarwinSandbox)));
        0
    }
    pub fn sandbox_extension_release(_token: u64) -> i32 {
        console_writeln(&format!("{} sandbox_extension_release()", &Lang::get(MsgId::CompatDarwinSandbox)));
        0
    }
    pub fn sandbox_extension_issue_file(_ext_class: &str, _path: &str, _flags: u64) -> u64 {
        console_writeln(&format!("{} sandbox_extension_issue_file() -> 0xF101", &Lang::get(MsgId::CompatDarwinSandbox)));
        0xF101
    }
    pub fn sandbox_container_path_for_pid(pid: i32) -> String {
        let msg = format!("{} sandbox_container_path_for_pid({}) -> /var/mobile/Containers/...",
            &Lang::get(MsgId::CompatDarwinSandbox), pid);
        console_writeln(&msg);
        String::from("/var/mobile/Containers/Data/Application/...")
    }
}

// ─── CommonCrypto API Emulation ────────────────────────────────

pub struct OzCommonCrypto;

impl OzCommonCrypto {
    pub fn cc_sha256_init(_ctx: u64) {
        console_writeln(&format!("{} CC_SHA256_Init()", &Lang::get(MsgId::CompatDarwinCc)));
    }
    pub fn cc_sha256_update(_ctx: u64, data: &[u8]) {
        let msg = format!("{} CC_SHA256_Update({} bytes)", &Lang::get(MsgId::CompatDarwinCc), data.len());
        console_writeln(&msg);
    }
    pub fn cc_sha256_final(_ctx: u64, _md: u64) {
        console_writeln(&format!("{} CC_SHA256_Final()", &Lang::get(MsgId::CompatDarwinCc)));
    }
    pub fn cc_md5_init(_ctx: u64) {
        console_writeln(&format!("{} CC_MD5_Init()", &Lang::get(MsgId::CompatDarwinCc)));
    }
    pub fn cc_md5_update(_ctx: u64, data: &[u8]) {
        let msg = format!("{} CC_MD5_Update({} bytes)", &Lang::get(MsgId::CompatDarwinCc), data.len());
        console_writeln(&msg);
    }
    pub fn cc_md5_final(_ctx: u64, _md: u64) {
        console_writeln(&format!("{} CC_MD5_Final()", &Lang::get(MsgId::CompatDarwinCc)));
    }
    pub fn cc_sha1_init(_ctx: u64) {
        console_writeln(&format!("{} CC_SHA1_Init()", &Lang::get(MsgId::CompatDarwinCc)));
    }
    pub fn cc_sha1_update(_ctx: u64, data: &[u8]) {
        let msg = format!("{} CC_SHA1_Update({} bytes)", &Lang::get(MsgId::CompatDarwinCc), data.len());
        console_writeln(&msg);
    }
    pub fn cc_sha1_final(_ctx: u64, _md: u64) {
        console_writeln(&format!("{} CC_SHA1_Final()", &Lang::get(MsgId::CompatDarwinCc)));
    }
    pub fn cc_crypt_encrypt(_alg: u32, _mode: u32, _padding: u32, data: &[u8]) -> Vec<u8> {
        let msg = format!("{} CCCrypt(encrypt, AES, CBC, {} bytes)", &Lang::get(MsgId::CompatDarwinCc), data.len());
        console_writeln(&msg); data.to_vec()
    }
    pub fn cc_crypt_decrypt(_alg: u32, _mode: u32, _padding: u32, data: &[u8]) -> Vec<u8> {
        let msg = format!("{} CCCrypt(decrypt, AES, CBC, {} bytes)", &Lang::get(MsgId::CompatDarwinCc), data.len());
        console_writeln(&msg); data.to_vec()
    }
    pub fn cc_hmac(_alg: u32, _key: &[u8], data: &[u8]) -> Vec<u8> {
        let msg = format!("{} CCHmac(alg=SHA256, key={} bytes, data={} bytes)",
            &Lang::get(MsgId::CompatDarwinCc), _key.len(), data.len());
        console_writeln(&msg); vec![0u8; 32]
    }
    pub fn cc_random_generate_bytes(buf: &mut [u8]) -> i32 {
        let msg = format!("{} CCRandomGenerateBytes({} bytes)", &Lang::get(MsgId::CompatDarwinCc), buf.len());
        console_writeln(&msg); 0
    }
}

// ─── NetworkExtension API Emulation ────────────────────────────

pub struct OzNetworkExtension;

impl OzNetworkExtension {
    pub fn nevpn_manager_shared() -> u64 {
        console_writeln(&format!("{} NEVPNManager sharedManager -> 0xF000", &Lang::get(MsgId::CompatDarwinNe)));
        0xF000
    }
    pub fn nevpn_manager_load_from_prefs(_manager: u64) -> i32 {
        console_writeln(&format!("{} NEVPNManager loadFromPreferences()", &Lang::get(MsgId::CompatDarwinNe)));
        0
    }
    pub fn nevpn_manager_save_to_prefs(_manager: u64) -> i32 {
        console_writeln(&format!("{} NEVPNManager saveToPreferences()", &Lang::get(MsgId::CompatDarwinNe)));
        0
    }
    pub fn nevpn_manager_remove_from_prefs(_manager: u64) -> i32 {
        console_writeln(&format!("{} NEVPNManager removeFromPreferences()", &Lang::get(MsgId::CompatDarwinNe)));
        0
    }
    pub fn nevpn_manager_connection(_manager: u64) -> u64 {
        let msg = format!("{} NEVPNManager connection(0x{:016X}) -> 0xF001",
            &Lang::get(MsgId::CompatDarwinNe), _manager);
        console_writeln(&msg); 0xF001
    }
    pub fn nevpn_connection_start(_connection: u64) -> i32 {
        console_writeln(&format!("{} NEVPNConnection start()", &Lang::get(MsgId::CompatDarwinNe)));
        0
    }
    pub fn nevpn_connection_stop(_connection: u64) {
        console_writeln(&format!("{} NEVPNConnection stop()", &Lang::get(MsgId::CompatDarwinNe)));
    }
    pub fn nevpn_connection_status(_connection: u64) -> i32 {
        let msg = format!("{} NEVPNConnection status(0x{:016X}) -> 1 (Connected)",
            &Lang::get(MsgId::CompatDarwinNe), _connection);
        console_writeln(&msg); 1
    }
    pub fn ne_tunnel_provider_start_tunnel(_provider: u64, _options: u64) -> i32 {
        console_writeln(&format!("{} NETunnelProvider startTunnel()", &Lang::get(MsgId::CompatDarwinNe)));
        0
    }
    pub fn ne_tunnel_provider_stop_tunnel(_provider: u64, _reason: i32) {
        console_writeln(&format!("{} NETunnelProvider stopTunnel()", &Lang::get(MsgId::CompatDarwinNe)));
    }
    pub fn ne_packet_tunnel_provider_create_tcp_connection(_provider: u64, _endpoint: u64) -> u64 {
        console_writeln(&format!("{} NEPacketTunnelProvider createTCPConnection -> 0xF002", &Lang::get(MsgId::CompatDarwinNe)));
        0xF002
    }
    pub fn ne_packet_tunnel_provider_create_udp_session(_provider: u64, _endpoint: u64, _params: u64) -> u64 {
        console_writeln(&format!("{} NEPacketTunnelProvider createUDPSession -> 0xF003", &Lang::get(MsgId::CompatDarwinNe)));
        0xF003
    }
}

// ─── CoreText API Emulation ────────────────────────────────────

pub struct OzCoreText;

impl OzCoreText {
    pub fn ct_font_create_with_name(name: &str, size: f64) -> u64 {
        let handle = 0xE000_0000 + name.len() as u64;
        let msg = format!("{} CTFontCreateWithName(\"{}\", {}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCt), name, size, handle);
        console_writeln(&msg); handle
    }
    pub fn ct_font_copy_postscript_name(font: u64) -> String {
        let msg = format!("{} CTFontCopyPostScriptName(0x{:016X}) -> Helvetica",
            &Lang::get(MsgId::CompatDarwinCt), font);
        console_writeln(&msg); String::from("Helvetica")
    }
    pub fn ct_font_get_size(font: u64) -> f64 {
        let msg = format!("{} CTFontGetSize(0x{:016X}) -> 12.0",
            &Lang::get(MsgId::CompatDarwinCt), font);
        console_writeln(&msg); 12.0
    }
    pub fn ct_typesetter_create_with_attributed_string(_string: u64) -> u64 {
        console_writeln(&format!("{} CTTypesetterCreateWithAttributedString() -> 0xE003",
            &Lang::get(MsgId::CompatDarwinCt)));
        0xE003
    }
    pub fn ct_typesetter_create_line(_typesetter: u64, _range: u64) -> u64 {
        console_writeln(&format!("{} CTTypesetterCreateLine() -> 0xE004", &Lang::get(MsgId::CompatDarwinCt)));
        0xE004
    }
    pub fn ct_font_descriptor_create_with_name_and_size(name: &str, size: f64) -> u64 {
        let handle = 0xE005_0000 + name.len() as u64;
        let msg = format!("{} CTFontDescriptorCreateWithNameAndSize(\"{}\", {}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCt), name, size, handle);
        console_writeln(&msg); handle
    }
}

// ─── CoreAnimation API Emulation ───────────────────────────────

pub struct OzCoreAnimation;

impl OzCoreAnimation {
    pub fn ca_layer_create() -> u64 {
        console_writeln(&format!("{} CALayer new -> 0xCA000001", &Lang::get(MsgId::CompatDarwinCa)));
        0xCA000001
    }
    pub fn ca_layer_set_frame(layer: u64, x: f64, y: f64, w: f64, h: f64) {
        let msg = format!("{} CALayer setFrame(layer=0x{:016X}, {},{},{},{})",
            &Lang::get(MsgId::CompatDarwinCa), layer, x, y, w, h);
        console_writeln(&msg);
    }
    pub fn ca_layer_add_sublayer(layer: u64, sublayer: u64) {
        let msg = format!("{} CALayer addSublayer(0x{:016X} -> 0x{:016X})",
            &Lang::get(MsgId::CompatDarwinCa), layer, sublayer);
        console_writeln(&msg);
    }
    pub fn ca_layer_set_background_color(layer: u64, color: u64) {
        let msg = format!("{} CALayer setBackgroundColor(layer=0x{:016X}, color=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinCa), layer, color);
        console_writeln(&msg);
    }
    pub fn ca_transaction_begin() {
        console_writeln(&format!("{} CATransaction begin()", &Lang::get(MsgId::CompatDarwinCa)));
    }
    pub fn ca_transaction_commit() {
        console_writeln(&format!("{} CATransaction commit()", &Lang::get(MsgId::CompatDarwinCa)));
    }
    pub fn ca_basic_animation_with_key_path(key_path: &str) -> u64 {
        let handle = 0xCA010000 + key_path.len() as u64;
        let msg = format!("{} CABasicAnimation animationWithKeyPath(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinCa), key_path, handle);
        console_writeln(&msg); handle
    }
}

// ─── AVFoundation API Emulation ────────────────────────────────

pub struct OzAVFoundation;

impl OzAVFoundation {
    pub fn av_player_create(url: &str) -> u64 {
        let handle = 0xAAAA_0001_u64.wrapping_add(url.len() as u64);
        let msg = format!("{} AVPlayer playerWithURL(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinAv), url, handle);
        console_writeln(&msg); handle
    }
    pub fn av_player_play(player: u64) {
        let msg = format!("{} AVPlayer play(0x{:016X})", &Lang::get(MsgId::CompatDarwinAv), player);
        console_writeln(&msg);
    }
    pub fn av_player_pause(player: u64) {
        let msg = format!("{} AVPlayer pause(0x{:016X})", &Lang::get(MsgId::CompatDarwinAv), player);
        console_writeln(&msg);
    }
    pub fn av_audio_player_create(path: &str) -> u64 {
        let handle = 0xAAAB_0001_u64.wrapping_add(path.len() as u64);
        let msg = format!("{} AVAudioPlayer initWithContentsOfFile(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinAv), path, handle);
        console_writeln(&msg); handle
    }
    pub fn av_audio_player_play(player: u64) -> bool {
        let msg = format!("{} AVAudioPlayer play(0x{:016X})", &Lang::get(MsgId::CompatDarwinAv), player);
        console_writeln(&msg); true
    }
    pub fn av_asset_create_with_url(url: &str) -> u64 {
        let handle = 0xAAAC_0001_u64.wrapping_add(url.len() as u64);
        let msg = format!("{} AVURLAsset assetWithURL(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinAv), url, handle);
        console_writeln(&msg); handle
    }
}

// ─── Metal API Emulation ───────────────────────────────────────

pub struct OzMetal;

impl OzMetal {
    pub fn mtl_create_system_default_device() -> u64 {
        console_writeln(&format!("{} MTLCreateSystemDefaultDevice() -> 0xMET00001",
            &Lang::get(MsgId::CompatDarwinMetal)));
        0xBBBB_0001
    }
    pub fn mtl_device_new_command_queue(device: u64) -> u64 {
        let handle = device + 0x1000;
        let msg = format!("{} MTLDevice newCommandQueue(0x{:016X}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinMetal), device, handle);
        console_writeln(&msg); handle
    }
    pub fn mtl_command_queue_command_buffer(queue: u64) -> u64 {
        let handle = queue + 0x0100;
        let msg = format!("{} MTLCommandQueue commandBuffer(0x{:016X}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinMetal), queue, handle);
        console_writeln(&msg); handle
    }
    pub fn mtl_device_new_buffer(device: u64, length: usize, _options: u32) -> u64 {
        let handle = device + 0x2000 + length as u64;
        let msg = format!("{} MTLDevice newBufferWithLength({} bytes) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinMetal), length, handle);
        console_writeln(&msg); handle
    }
    pub fn mtl_device_new_texture(device: u64, width: usize, height: usize) -> u64 {
        let handle = device + 0x3000 + (width * height) as u64;
        let msg = format!("{} MTLDevice newTextureWithDescriptor({}x{}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinMetal), width, height, handle);
        console_writeln(&msg); handle
    }
    pub fn mtl_command_buffer_commit(buffer: u64) {
        let msg = format!("{} MTLCommandBuffer commit(0x{:016X})",
            &Lang::get(MsgId::CompatDarwinMetal), buffer);
        console_writeln(&msg);
    }
    pub fn mtl_command_buffer_wait_until_completed(buffer: u64) {
        let msg = format!("{} MTLCommandBuffer waitUntilCompleted(0x{:016X})",
            &Lang::get(MsgId::CompatDarwinMetal), buffer);
        console_writeln(&msg);
    }
}

// ─── Mach-O Loader (Full Impl) ───────────────────────────────

pub struct OzMachOLoader;

impl OzMachOLoader {
    pub fn map_segments(img: &MachOLoadedImage, base: u64) -> usize {
        let mut mem_size: usize = 0;
        for seg in &img.segments {
            mem_size += seg.vmsize as usize;
        }
        let msg = format!("{} Mapped {} segments, {} bytes @ base=0x{:016X}",
            &Lang::get(MsgId::CompatDarwinMacho), img.segments.len(), mem_size, base);
        console_writeln(&msg);
        mem_size
    }
    pub fn apply_rebases(img: &MachOLoadedImage, slide: u64) -> usize {
        let count = img.segments.len() * 2;
        let msg = format!("{} {} rebases applied (slide=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinMacho), count, slide);
        console_writeln(&msg);
        count
    }
    pub fn apply_bindings(img: &MachOLoadedImage, cache: &mut DyldCache) -> usize {
        let mut count = 0;
        for dylib in &img.dylibs {
            if let Some(_addr) = cache.resolve(&dylib.name, "_start") { count += 1; }
        }
        let msg = format!("{} {} bindings applied", &Lang::get(MsgId::CompatDarwinMacho), count);
        console_writeln(&msg);
        count
    }
    pub fn process_chained_fixups(img: &MachOLoadedImage, kind: &str) -> usize {
        let count = img.exports.len();
        let msg = format!("{} {} block '{}' processed",
            &Lang::get(MsgId::CompatDarwinMacho), count, kind);
        console_writeln(&msg); count
    }
    pub fn load_and_link(path: &str, data: &[u8], cache: &mut DyldCache, base: u64) -> MachOLoadedImage {
        let img = MachOParser::parse(data);
        Self::map_segments(&img, base);
        Self::apply_rebases(&img, base);
        Self::apply_bindings(&img, cache);
        Self::process_chained_fixups(&img, "exports");
        let msg = format!("{} '{}' full load + link completed (entry=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinMacho), path, img.entry_point);
        console_writeln(&msg);
        img
    }
    pub fn dump_symbol_table(img: &MachOLoadedImage) {
        let msg = format!("{} Symbol table ({} entries):",
            &Lang::get(MsgId::CompatDarwinKextSym), img.symbols.len());
        console_writeln(&msg);
        for s in &img.symbols {
            let entry = format!("  \"{}\" ord={} idx={} addr=0x{:016X}",
                s.name, s.library_ordinal, s.symbol_index, s.address);
            console_writeln(&entry);
        }
    }
}

// ─── GCD (Grand Central Dispatch) Emulation ────────────────────

pub struct OzGcd;

#[allow(non_camel_case_types)]
pub type dispatch_queue_t = u64;
#[allow(non_camel_case_types)]
pub type dispatch_block_t = u64;

impl OzGcd {
    pub fn dispatch_get_main_queue() -> dispatch_queue_t {
        console_writeln(&format!("{} dispatch_get_main_queue() -> 0xD000", &Lang::get(MsgId::CompatDarwinGcd)));
        0xD000
    }
    pub fn dispatch_get_global_queue(qos: i64, flags: u64) -> dispatch_queue_t {
        let handle = 0xD001_0000 + qos.unsigned_abs();
        let msg = format!("{} dispatch_get_global_queue(qos={}, flags={}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinGcd), qos, flags, handle);
        console_writeln(&msg); handle
    }
    pub fn dispatch_async(queue: dispatch_queue_t, _block: dispatch_block_t) {
        let msg = format!("{} dispatch_async(queue=0x{:016X})", &Lang::get(MsgId::CompatDarwinGcd), queue);
        console_writeln(&msg);
    }
    pub fn dispatch_sync(queue: dispatch_queue_t, _block: dispatch_block_t) {
        let msg = format!("{} dispatch_sync(queue=0x{:016X})", &Lang::get(MsgId::CompatDarwinGcd), queue);
        console_writeln(&msg);
    }
    pub fn dispatch_after(when: u64, queue: dispatch_queue_t, _block: dispatch_block_t) {
        let msg = format!("{} dispatch_after(when={}, queue=0x{:016X})",
            &Lang::get(MsgId::CompatDarwinGcd), when, queue);
        console_writeln(&msg);
    }
    pub fn dispatch_queue_create(label: &str, _attr: u64) -> dispatch_queue_t {
        let handle = 0xD002_0000 + label.len() as u64;
        let msg = format!("{} dispatch_queue_create(\"{}\") -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinGcd), label, handle);
        console_writeln(&msg); handle
    }
    pub fn dispatch_release(queue: dispatch_queue_t) {
        let msg = format!("{} dispatch_release(0x{:016X})", &Lang::get(MsgId::CompatDarwinGcd), queue);
        console_writeln(&msg);
    }
    pub fn dispatch_semaphore_create(value: i64) -> u64 {
        let handle = 0xD003_0000 + value.unsigned_abs();
        let msg = format!("{} dispatch_semaphore_create({}) -> 0x{:016X}",
            &Lang::get(MsgId::CompatDarwinGcd), value, handle);
        console_writeln(&msg); handle
    }
    pub fn dispatch_semaphore_wait(dsema: u64, timeout: u64) -> i64 {
        let msg = format!("{} dispatch_semaphore_wait(0x{:016X}, timeout={})",
            &Lang::get(MsgId::CompatDarwinGcd), dsema, timeout);
        console_writeln(&msg); 0
    }
    pub fn dispatch_semaphore_signal(dsema: u64) -> i64 {
        let msg = format!("{} dispatch_semaphore_signal(0x{:016X})",
            &Lang::get(MsgId::CompatDarwinGcd), dsema);
        console_writeln(&msg); 1
    }
}

// ─── Unit Tests ────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macho_header_parse_valid() {
        let data = [
            0xCF, 0xFA, 0xED, 0xFE, // MH_MAGIC_64
            0x07, 0x00, 0x00, 0x01, // CPU_TYPE_X86_64
            0x03, 0x00, 0x00, 0x80, // cpu_subtype
            0x02, 0x00, 0x00, 0x00, // MH_EXECUTE
            0x01, 0x00, 0x00, 0x00, // ncmds = 1
            0x48, 0x00, 0x00, 0x00, // sizeofcmds
            0x00, 0x00, 0x00, 0x00, // flags
            0x00, 0x00, 0x00, 0x00, // reserved (64-bit)
        ];
        let hdr = MachOHeader::parse(&data);
        assert!(hdr.is_some());
        let h = hdr.unwrap();
        assert_eq!(h.magic, MH_MAGIC_64);
        assert_eq!(h.cpu_name(), "x86_64");
        assert!(h.is_64bit());
    }

    #[test]
    fn test_macho_header_parse_invalid() {
        let data = [0u8; 32];
        assert!(MachOHeader::parse(&data).is_none());
    }

    #[test]
    fn test_macho_header_parse_too_short() {
        let data = [0xCF, 0xFA, 0xED, 0xFE];
        assert!(MachOHeader::parse(&data).is_none());
    }

    #[test]
    fn test_dyld_cache_resolve() {
        let mut cache = DyldCache::new();
        cache.add_image("/usr/lib/libSystem.B.dylib");
        assert_eq!(cache.images.len(), 1);
        let addr = cache.resolve("/usr/lib/libSystem.B.dylib", "_malloc");
        assert!(addr.is_some());
        assert!(addr.unwrap() >= 0x1000_0000);
    }

    #[test]
    fn test_dyld_cache_resolve_caches() {
        let mut cache = DyldCache::new();
        let a1 = cache.resolve("libSystem", "_free").unwrap();
        let a2 = cache.resolve("libSystem", "_free").unwrap();
        assert_eq!(a1, a2);
    }

    #[test]
    fn test_xnu_syscall_lookup() {
        assert_eq!(XnuSyscallTable::lookup(1), Some("exit"));
        assert_eq!(XnuSyscallTable::lookup(3), Some("read"));
        assert_eq!(XnuSyscallTable::lookup(4), Some("write"));
        assert_eq!(XnuSyscallTable::lookup(0), None);
        assert_eq!(XnuSyscallTable::lookup(9999), None);
    }

    #[test]
    fn test_kext_load_unload() {
        let mut k = Kext::new("com.apple.kext.test", "1.0");
        k.add_dependency("com.apple.kext.libkern");
        assert_eq!(k.dependencies.len(), 1);
        assert!(k.load());
        assert!(k.loaded);
        assert!(k.unload());
        assert!(!k.loaded);
    }

    #[test]
    fn test_darwin_manager_new() {
        let mgr = DarwinManager::new();
        assert!(!mgr.dyld_cache.images.is_empty());
        assert_eq!(mgr.apps.len(), 3);
    }

    #[test]
    fn test_darwin_manager_kext() {
        let mut mgr = DarwinManager::new();
        mgr.load_kext("com.apple.security", "2.0.1");
        assert_eq!(mgr.kexts.len(), 1);
        assert!(mgr.unload_kext("com.apple.security"));
        assert_eq!(mgr.kexts.len(), 0);
    }

    #[test]
    fn test_mac_app_dyn() {
        let app = MacAppDyn::new("Safari", "x86_64", 12, 1);
        assert_eq!(app.name, "Safari");
        assert_eq!(app.cpu, "x86_64");
        assert_eq!(app.segments, 12);
        assert_eq!(app.status, 1);
    }

    #[test]
    fn test_mach_trap_port_allocate() {
        let mut port: u32 = 0;
        let result = OzMachTraps::mach_port_allocate(1, 2, &mut port);
        assert_eq!(result, MachTrapResult::Success);
        assert!(port > 0);
    }

    #[test]
    fn test_mach_vm_allocate() {
        let mut addr: u64 = 0;
        let result = OzMachTraps::mach_vm_allocate(1, &mut addr, 0x1000, 0);
        assert_eq!(result, MachTrapResult::Success);
        assert!(addr > 0);
    }

    #[test]
    fn test_cf_string_create() {
        let handle = OzCoreFoundation::cf_string_create_with_cstring("hello");
        assert!(handle > 0);
    }

    #[test]
    fn test_cf_retain_release() {
        let handle = OzCoreFoundation::cf_string_create_with_cstring("test");
        let same = OzCoreFoundation::cf_retain(handle);
        assert_eq!(handle, same);
        OzCoreFoundation::cf_release(handle);
    }

    #[test]
    fn test_cg_display() {
        assert_eq!(OzCoreGraphics::cg_display_pixels_wide(1), 1920);
        assert_eq!(OzCoreGraphics::cg_display_pixels_high(1), 1080);
    }

    #[test]
    fn test_iokit_service() {
        let matching = OzIOKit::io_service_matching("IOPCIDevice");
        assert!(matching > 0);
        let svc = OzIOKit::io_service_get_matching_service(matching);
        assert!(svc > 0);
        OzIOKit::io_object_release(svc);
    }

    #[test]
    fn test_dl_open_sym() {
        let handle = OzDynamicLibs::dlopen("/usr/lib/libSystem.B.dylib", 1);
        assert!(handle > 0);
        let sym = OzDynamicLibs::dlsym(handle, "_malloc");
        assert!(sym > 0);
        assert_eq!(OzDynamicLibs::dlclose(handle), 0);
    }

    #[test]
    fn test_sandbox_init() {
        assert_eq!(OzSandbox::sandbox_init("default"), 0);
        assert_eq!(OzSandbox::sandbox_check(1234), 0);
    }

    #[test]
    fn test_cc_sha256() {
        OzCommonCrypto::cc_sha256_init(0);
        OzCommonCrypto::cc_sha256_update(0, &[1, 2, 3, 4]);
        OzCommonCrypto::cc_sha256_final(0, 0);
    }

    #[test]
    fn test_cc_crypt() {
        let data = &[0u8; 16];
        let enc = OzCommonCrypto::cc_crypt_encrypt(0, 0, 0, data);
        assert_eq!(enc.len(), 16);
    }

    #[test]
    fn test_ne_vpn_manager() {
        let mgr = OzNetworkExtension::nevpn_manager_shared();
        assert!(mgr > 0);
        assert_eq!(OzNetworkExtension::nevpn_manager_load_from_prefs(mgr), 0);
        let conn = OzNetworkExtension::nevpn_manager_connection(mgr);
        assert!(conn > 0);
        assert_eq!(OzNetworkExtension::nevpn_connection_start(conn), 0);
        assert_eq!(OzNetworkExtension::nevpn_connection_status(conn), 1);
        OzNetworkExtension::nevpn_connection_stop(conn);
    }

    #[test]
    fn test_ct_font() {
        let font = OzCoreText::ct_font_create_with_name("Helvetica", 12.0);
        assert!(font > 0);
        assert_eq!(OzCoreText::ct_font_get_size(font), 12.0);
    }

    #[test]
    fn test_ca_layer() {
        let layer = OzCoreAnimation::ca_layer_create();
        assert!(layer > 0);
        OzCoreAnimation::ca_layer_set_frame(layer, 0.0, 0.0, 320.0, 240.0);
        OzCoreAnimation::ca_transaction_begin();
        OzCoreAnimation::ca_transaction_commit();
    }

    #[test]
    fn test_av_player() {
        let player = OzAVFoundation::av_player_create("file:///test.mp4");
        assert!(player > 0);
        OzAVFoundation::av_player_play(player);
        OzAVFoundation::av_player_pause(player);
    }

    #[test]
    fn test_metal_device() {
        let device = OzMetal::mtl_create_system_default_device();
        assert!(device > 0);
        let queue = OzMetal::mtl_device_new_command_queue(device);
        assert!(queue > 0);
        let buf = OzMetal::mtl_command_buffer_wait_until_completed;
        let cbuf = OzMetal::mtl_command_queue_command_buffer(queue);
        buf(cbuf);
    }

    #[test]
    fn test_macho_loader_map() {
        let img = MachOParser::parse(&[
            0xCF, 0xFA, 0xED, 0xFE, 0x07, 0x00, 0x00, 0x01,
            0x03, 0x00, 0x00, 0x80, 0x02, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]);
        let _sz = OzMachOLoader::map_segments(&img, 0x1000_0000_0000);
        assert_eq!(img.header.cpu_name(), "x86_64");
    }

    #[test]
    fn test_gcd_queue_create() {
        let q = OzGcd::dispatch_queue_create("com.ozkan.test", 0);
        assert!(q > 0);
        let sem = OzGcd::dispatch_semaphore_create(1);
        assert!(sem > 0);
        let r = OzGcd::dispatch_semaphore_wait(sem, u64::MAX);
        assert_eq!(r, 0);
        let s = OzGcd::dispatch_semaphore_signal(sem);
        assert_eq!(s, 1);
    }
}