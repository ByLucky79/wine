// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : winmm.dll — Ses dosyası çalma API'leri
//                      (PlaySoundA/W, sndPlaySoundA/W)
// Dosya Yolu         : compat/win32/dlls/rust/winmm_core/src/mm_sound.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Windows Multimedia ses dosyası çalma fonksiyonlarının clean-room Rust
//   implementasyonu. PlaySoundA, PlaySoundW (WAV dosyası / kaynak / bellek
//   tamponundan ses çalma) ve eski sndPlaySoundA/W API'lerini içerir.
//   MSDN public spesifikasyonuna dayanır; stub implementasyon.
//
// Bağımlı Dosyalar:
//   1-) winmm_root.rs (üst crate — tipler)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 82
// *******************************************************************

#![allow(dead_code)]

use crate::{BOOL, DWORD, HINSTANCE, LPCSTR, LPCWSTR};

// Ses çalma bayrakları (Win32 public API sabitleri)
pub const SND_SYNC:        DWORD = 0x0000;
pub const SND_ASYNC:       DWORD = 0x0001;
pub const SND_NODEFAULT:   DWORD = 0x0002;
pub const SND_MEMORY:      DWORD = 0x0004;
pub const SND_LOOP:        DWORD = 0x0008;
pub const SND_NOSTOP:      DWORD = 0x0010;
pub const SND_NOWAIT:      DWORD = 0x0002_0000;
pub const SND_ALIAS:       DWORD = 0x0001_0000;
pub const SND_ALIAS_ID:    DWORD = 0x0011_0000;
pub const SND_FILENAME:    DWORD = 0x0002_0000;
pub const SND_RESOURCE:    DWORD = 0x0004_0040;
pub const SND_PURGE:       DWORD = 0x0040;
pub const SND_APPLICATION: DWORD = 0x0080;
pub const SND_SENTRY:      DWORD = 0x0008_0000;
pub const SND_RING:        DWORD = 0x0010_0000;
pub const SND_SYSTEM:      DWORD = 0x0020_0000;

const FALSE: BOOL = 0;

/// WAV sesi çalar — ANSI sürümü.
/// Win32: PlaySoundA (WINMM.@)
#[no_mangle]
pub unsafe extern "system" fn PlaySoundA(
    _psz_sound: LPCSTR,
    _h_mod: HINSTANCE,
    _fdw_sound: DWORD,
) -> BOOL {
    // Stub: ÖZKAN-OS ses altyapısı bağlandığında doldurulacak
    FALSE
}

/// WAV sesi çalar — Unicode sürümü.
/// Win32: PlaySoundW (WINMM.@)
#[no_mangle]
pub unsafe extern "system" fn PlaySoundW(
    _psz_sound: LPCWSTR,
    _h_mod: HINSTANCE,
    _fdw_sound: DWORD,
) -> BOOL {
    FALSE
}

/// PlaySoundA ile özdeş — eski takma ad.
/// Win32: PlaySound (WINMM.@)
#[no_mangle]
pub unsafe extern "system" fn PlaySound(
    psz_sound: LPCSTR,
    h_mod: HINSTANCE,
    fdw_sound: DWORD,
) -> BOOL {
    PlaySoundA(psz_sound, h_mod, fdw_sound)
}

/// Eski-stil ses çalma — ANSI (Win16 uyumluluğu için).
/// Win32: sndPlaySoundA (WINMM.@)
#[no_mangle]
pub unsafe extern "system" fn sndPlaySoundA(
    _lpsz_sound: LPCSTR,
    _fu_sound: DWORD,
) -> BOOL {
    FALSE
}

/// Eski-stil ses çalma — Unicode (Win16 uyumluluğu için).
/// Win32: sndPlaySoundW (WINMM.@)
#[no_mangle]
pub unsafe extern "system" fn sndPlaySoundW(
    _lpsz_sound: LPCWSTR,
    _fu_sound: DWORD,
) -> BOOL {
    FALSE
}
