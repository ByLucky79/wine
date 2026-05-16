// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : winmm.dll clean-room Rust port — kök dosyası,
//                      ortak Win32 MM tipleri, sabitler ve modül bildirimleri
// Dosya Yolu         : compat/win32/dlls/rust/winmm_core/winmm_root.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Windows Multimedia API (winmm.dll) için clean-room Rust implementasyonu.
//   MSDN public spesifikasyonuna dayanır; Wine LGPL kaynak kodundan hiçbir
//   satır kopyalanmamıştır. Tüm fonksiyonlar stub olarak sunulmuştur;
//   ÖZKAN-OS ses/MIDI/joystick altyapısına bağlandığında doldurulacaktır.
//
// Bağımlı Dosyalar:
//   1-) core (no_std)
//   2-) alloc (no_std alloc crate)
//
//              Dosyaya Müdahaleler
// 2026-05-16      Dosya oluşturuldu — session 82
// *******************************************************************

#![no_std]
#![allow(non_camel_case_types)]

extern crate alloc;

// ── Alt modüller ────────────────────────────────────────────────────────────
#[path = "src/mm_driver.rs"]   pub mod mm_driver;
#[path = "src/mm_sound.rs"]    pub mod mm_sound;
#[path = "src/mm_joystick.rs"] pub mod mm_joystick;
#[path = "src/mm_mci.rs"]      pub mod mm_mci;
#[path = "src/mm_midi.rs"]     pub mod mm_midi;
#[path = "src/mm_mixer.rs"]    pub mod mm_mixer;
#[path = "src/mm_mmio.rs"]     pub mod mm_mmio;
#[path = "src/mm_wave.rs"]     pub mod mm_wave;
#[path = "src/mm_timer.rs"]    pub mod mm_timer;

// ── Temel Win32 tipleri ─────────────────────────────────────────────────────
pub type BOOL      = i32;
pub type BYTE      = u8;
pub type WORD      = u16;
pub type DWORD     = u32;
pub type UINT      = u32;
pub type INT       = i32;
pub type LONG      = i32;
pub type ULONG     = u32;
pub type CHAR      = u8;
pub type WCHAR     = u16;
pub type LPSTR     = *mut u8;
pub type LPCSTR    = *const u8;
pub type LPWSTR    = *mut u16;
pub type LPCWSTR   = *const u16;
pub type LPVOID    = *mut core::ffi::c_void;
pub type LPCVOID   = *const core::ffi::c_void;
pub type LPDWORD   = *mut DWORD;
pub type LPUINT    = *mut UINT;
pub type LPWORD    = *mut WORD;
pub type HANDLE    = *mut core::ffi::c_void;
pub type HWND      = *mut core::ffi::c_void;
pub type HINSTANCE = *mut core::ffi::c_void;
pub type HTASK     = *mut core::ffi::c_void;

// MM'e özgü handle tipleri
pub type HDRVR     = *mut core::ffi::c_void;
pub type HMIDIOUT  = *mut core::ffi::c_void;
pub type HMIDIIN   = *mut core::ffi::c_void;
pub type HMIDISTRM = *mut core::ffi::c_void;
pub type HWAVEOUT  = *mut core::ffi::c_void;
pub type HWAVEIN   = *mut core::ffi::c_void;
pub type HMIXER    = *mut core::ffi::c_void;
pub type HMIXEROBJ = *mut core::ffi::c_void;
pub type HMMIO     = *mut core::ffi::c_void;

// Dönüş ve ID tipleri
pub type MMRESULT    = u32;
pub type FOURCC      = u32;
pub type MCIDEVICEID = u32;
pub type LRESULT     = isize;
pub type UINT_PTR    = usize;
pub type DWORD_PTR   = usize;
pub type LPARAM      = isize;
pub type WPARAM      = usize;

// Pointer-to-handle tipleri
pub type LPHMIDIOUT  = *mut HMIDIOUT;
pub type LPHMIDIIN   = *mut HMIDIIN;
pub type LPHMIDISTRM = *mut HMIDISTRM;
pub type LPHWAVEOUT  = *mut HWAVEOUT;
pub type LPHWAVEIN   = *mut HWAVEIN;
pub type LPHMIXER    = *mut HMIXER;
pub type LPHMMIO     = *mut HMMIO;

// ── MMSYSTEM hata kodları ───────────────────────────────────────────────────
pub const MMSYSERR_NOERROR:     MMRESULT = 0;
pub const MMSYSERR_ERROR:       MMRESULT = 1;
pub const MMSYSERR_BADDEVICEID: MMRESULT = 2;
pub const MMSYSERR_NOTENABLED:  MMRESULT = 3;
pub const MMSYSERR_ALLOCATED:   MMRESULT = 4;
pub const MMSYSERR_INVALHANDLE: MMRESULT = 5;
pub const MMSYSERR_NODRIVER:    MMRESULT = 6;
pub const MMSYSERR_NOMEM:       MMRESULT = 7;
pub const MMSYSERR_NOTSUPPORTED:MMRESULT = 9;
pub const MMSYSERR_BADERRNUM:   MMRESULT = 9;
pub const MMSYSERR_INVALFLAG:   MMRESULT = 10;
pub const MMSYSERR_INVALPARAM:  MMRESULT = 11;
pub const MMSYSERR_HANDLEBUSY:  MMRESULT = 12;
pub const MMSYSERR_INVALIDALIAS:MMRESULT = 13;
pub const MMSYSERR_BADDB:       MMRESULT = 14;
pub const MMSYSERR_KEYNOTFOUND: MMRESULT = 15;
pub const MMSYSERR_READERROR:   MMRESULT = 16;
pub const MMSYSERR_WRITEERROR:  MMRESULT = 17;
pub const MMSYSERR_DELETEERROR: MMRESULT = 18;
pub const MMSYSERR_VALNOTFOUND: MMRESULT = 19;
pub const MMSYSERR_NODRIVERCB:  MMRESULT = 20;

// Wave hata kodları
pub const WAVERR_BADFORMAT:     MMRESULT = 32;
pub const WAVERR_STILLPLAYING:  MMRESULT = 33;
pub const WAVERR_UNPREPARED:    MMRESULT = 34;
pub const WAVERR_SYNC:          MMRESULT = 35;

// MIDI hata kodları
pub const MIDIERR_UNPREPARED:   MMRESULT = 64;
pub const MIDIERR_STILLPLAYING: MMRESULT = 65;
pub const MIDIERR_NOMAP:        MMRESULT = 66;
pub const MIDIERR_NOTREADY:     MMRESULT = 67;
pub const MIDIERR_NODEVICE:     MMRESULT = 68;
pub const MIDIERR_INVALIDSETUP: MMRESULT = 69;
pub const MIDIERR_BADOPENMODE:  MMRESULT = 70;
pub const MIDIERR_DONT_CONTINUE:MMRESULT = 71;

// Mixer hata kodları
pub const MIXERR_INVALLINE:     MMRESULT = 1024;
pub const MIXERR_INVALCONTROL:  MMRESULT = 1025;
pub const MIXERR_INVALVALUE:    MMRESULT = 1026;

// Timer hata kodları
pub const TIMERR_NOERROR:  u32 = 0;
pub const TIMERR_NOCANDO:  u32 = 97;
pub const TIMERR_STRUCT:   u32 = 129;

// MMIO hata kodları
pub const MMIOERR_BASE:             u32 = 256;
pub const MMIOERR_FILENOTFOUND:     u32 = MMIOERR_BASE + 1;
pub const MMIOERR_OUTOFMEMORY:      u32 = MMIOERR_BASE + 2;
pub const MMIOERR_CANNOTOPEN:       u32 = MMIOERR_BASE + 3;
pub const MMIOERR_CANNOTCLOSE:      u32 = MMIOERR_BASE + 4;
pub const MMIOERR_CANNOTREAD:       u32 = MMIOERR_BASE + 5;
pub const MMIOERR_CANNOTWRITE:      u32 = MMIOERR_BASE + 6;
pub const MMIOERR_CANNOTSEEK:       u32 = MMIOERR_BASE + 7;
pub const MMIOERR_CANNOTEXPAND:     u32 = MMIOERR_BASE + 8;
pub const MMIOERR_CHUNKNOTFOUND:    u32 = MMIOERR_BASE + 9;
pub const MMIOERR_UNBUFFERED:       u32 = MMIOERR_BASE + 10;
pub const MMIOERR_PATHNOTFOUND:     u32 = MMIOERR_BASE + 11;
pub const MMIOERR_ACCESSDENIED:     u32 = MMIOERR_BASE + 12;
pub const MMIOERR_SHARINGVIOLATION: u32 = MMIOERR_BASE + 13;
pub const MMIOERR_NETWORKERROR:     u32 = MMIOERR_BASE + 14;
pub const MMIOERR_TOOMANYOPENFILES: u32 = MMIOERR_BASE + 15;
pub const MMIOERR_INVALIDPARAMETER: u32 = MMIOERR_BASE + 16;

// ── Temel MM yapıları ────────────────────────────────────────────────────────

/// Wave ses biçimi tanımlayıcısı (WAVEFORMATEX)
#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEFORMATEX {
    pub wFormatTag:      WORD,
    pub nChannels:       WORD,
    pub nSamplesPerSec:  DWORD,
    pub nAvgBytesPerSec: DWORD,
    pub nBlockAlign:     WORD,
    pub wBitsPerSample:  WORD,
    pub cbSize:          WORD,
}

/// Wave çıkış tampon başlığı (WAVEHDR)
#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEHDR {
    pub lpData:          LPSTR,
    pub dwBufferLength:  DWORD,
    pub dwBytesRecorded: DWORD,
    pub dwUser:          DWORD_PTR,
    pub dwFlags:         DWORD,
    pub dwLoops:         DWORD,
    pub lpNext:          *mut WAVEHDR,
    pub reserved:        DWORD_PTR,
}

/// MIDI başlık yapısı (MIDIHDR)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIHDR {
    pub lpData:          LPSTR,
    pub dwBufferLength:  DWORD,
    pub dwBytesRecorded: DWORD,
    pub dwUser:          DWORD_PTR,
    pub dwFlags:         DWORD,
    pub lpNext:          *mut MIDIHDR,
    pub reserved:        DWORD_PTR,
    pub dwOffset:        DWORD,
    pub dwReserved:      [DWORD_PTR; 8],
}

/// MIDI stream tempo özelliği (MIDISTRMBUFFVER)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDISTRMBUFFVER {
    pub dwVersion:    DWORD,
    pub dwMid:        DWORD,
    pub dwOEMVersion: DWORD,
}

/// MIDI stream zamanlama birimi (MIDIPROPTIMEDIV)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIPROPTIMEDIV {
    pub cbStruct:  DWORD,
    pub dwTimeDiv: DWORD,
}

/// MIDI stream tempo bilgisi (MIDIPROPTEMPO)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIPROPTEMPO {
    pub cbStruct: DWORD,
    pub dwTempo:  DWORD,
}

/// Multimedya zaman damgası (MMTIME)
#[repr(C)]
pub union MmTimeUnion {
    pub ms:       DWORD,
    pub sample:   DWORD,
    pub cb:       DWORD,
    pub ticks:    DWORD,
    pub smpte_raw:[BYTE; 8],
    pub midi_raw: [DWORD; 2],
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct MMTIME {
    pub wType: UINT,
    pub u:     MmTimeUnion,
}

/// Zamanlayıcı kapasitesi (TIMECAPS)
#[repr(C)]
#[allow(non_snake_case)]
pub struct TIMECAPS {
    pub wPeriodMin: UINT,
    pub wPeriodMax: UINT,
}

/// MMIO dosya bilgisi (MMIOINFO)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MMIOINFO {
    pub dwFlags:      DWORD,
    pub fccIOProc:    FOURCC,
    pub pIOProc:      Option<unsafe extern "system" fn(LPSTR, UINT, LPARAM, LPARAM) -> LRESULT>,
    pub wErrorRet:    UINT,
    pub htask:        HTASK,
    pub cchBuffer:    LONG,
    pub pchBuffer:    LPSTR,
    pub pchNext:      LPSTR,
    pub pchEndRead:   LPSTR,
    pub pchEndWrite:  LPSTR,
    pub lBufOffset:   LONG,
    pub lDiskOffset:  LONG,
    pub adwInfo:      [DWORD; 3],
    pub dwReserved1:  DWORD,
    pub dwReserved2:  DWORD,
    pub hmmio:        HMMIO,
}

/// RIFF chunk bilgisi (MMCKINFO)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MMCKINFO {
    pub ckid:         FOURCC,
    pub cksize:       DWORD,
    pub fccType:      FOURCC,
    pub dwDataOffset: DWORD,
    pub dwFlags:      DWORD,
}

/// Wave çıkış cihazı kapasitesi — ANSI (WAVEOUTCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEOUTCAPSA {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [CHAR; 32],
    pub dwFormats:      DWORD,
    pub wChannels:      WORD,
    pub wReserved1:     WORD,
    pub dwSupport:      DWORD,
}

/// Wave çıkış cihazı kapasitesi — Unicode (WAVEOUTCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEOUTCAPSW {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [WCHAR; 32],
    pub dwFormats:      DWORD,
    pub wChannels:      WORD,
    pub wReserved1:     WORD,
    pub dwSupport:      DWORD,
}

/// Wave giriş cihazı kapasitesi — ANSI (WAVEINCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEINCAPSA {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [CHAR; 32],
    pub dwFormats:      DWORD,
    pub wChannels:      WORD,
    pub wReserved1:     WORD,
}

/// Wave giriş cihazı kapasitesi — Unicode (WAVEINCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEINCAPSW {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [WCHAR; 32],
    pub dwFormats:      DWORD,
    pub wChannels:      WORD,
    pub wReserved1:     WORD,
}

/// MIDI çıkış cihazı kapasitesi — ANSI (MIDIOUTCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIOUTCAPSA {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [CHAR; 32],
    pub wTechnology:    WORD,
    pub wVoices:        WORD,
    pub wNotes:         WORD,
    pub wChannelMask:   WORD,
    pub dwSupport:      DWORD,
}

/// MIDI çıkış cihazı kapasitesi — Unicode (MIDIOUTCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIOUTCAPSW {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [WCHAR; 32],
    pub wTechnology:    WORD,
    pub wVoices:        WORD,
    pub wNotes:         WORD,
    pub wChannelMask:   WORD,
    pub dwSupport:      DWORD,
}

/// MIDI giriş cihazı kapasitesi — ANSI (MIDIINCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIINCAPSA {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [CHAR; 32],
    pub dwSupport:      DWORD,
}

/// MIDI giriş cihazı kapasitesi — Unicode (MIDIINCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIDIINCAPSW {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [WCHAR; 32],
    pub dwSupport:      DWORD,
}

/// Aux ses cihazı kapasitesi — ANSI (AUXCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct AUXCAPSA {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [CHAR; 32],
    pub wTechnology:    WORD,
    pub wReserved1:     WORD,
    pub dwSupport:      DWORD,
}

/// Aux ses cihazı kapasitesi — Unicode (AUXCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct AUXCAPSW {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [WCHAR; 32],
    pub wTechnology:    WORD,
    pub wReserved1:     WORD,
    pub dwSupport:      DWORD,
}

/// Joystick kapasitesi — ANSI (JOYCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct JOYCAPSA {
    pub wMid:         WORD,
    pub wPid:         WORD,
    pub szPname:      [CHAR; 32],
    pub wXmin:        UINT,
    pub wXmax:        UINT,
    pub wYmin:        UINT,
    pub wYmax:        UINT,
    pub wZmin:        UINT,
    pub wZmax:        UINT,
    pub wNumButtons:  UINT,
    pub wPeriodMin:   UINT,
    pub wPeriodMax:   UINT,
    pub wRmin:        UINT,
    pub wRmax:        UINT,
    pub wUmin:        UINT,
    pub wUmax:        UINT,
    pub wVmin:        UINT,
    pub wVmax:        UINT,
    pub wCaps:        UINT,
    pub wMaxAxes:     UINT,
    pub wNumAxes:     UINT,
    pub wMaxButtons:  UINT,
    pub szRegKey:     [CHAR; 32],
    pub szOEMVxD:     [CHAR; 260],
}

/// Joystick kapasitesi — Unicode (JOYCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct JOYCAPSW {
    pub wMid:         WORD,
    pub wPid:         WORD,
    pub szPname:      [WCHAR; 32],
    pub wXmin:        UINT,
    pub wXmax:        UINT,
    pub wYmin:        UINT,
    pub wYmax:        UINT,
    pub wZmin:        UINT,
    pub wZmax:        UINT,
    pub wNumButtons:  UINT,
    pub wPeriodMin:   UINT,
    pub wPeriodMax:   UINT,
    pub wRmin:        UINT,
    pub wRmax:        UINT,
    pub wUmin:        UINT,
    pub wUmax:        UINT,
    pub wVmin:        UINT,
    pub wVmax:        UINT,
    pub wCaps:        UINT,
    pub wMaxAxes:     UINT,
    pub wNumAxes:     UINT,
    pub wMaxButtons:  UINT,
    pub szRegKey:     [WCHAR; 32],
    pub szOEMVxD:     [WCHAR; 260],
}

/// Joystick anlık konum bilgisi (JOYINFO)
#[repr(C)]
#[allow(non_snake_case)]
pub struct JOYINFO {
    pub wXpos:    UINT,
    pub wYpos:    UINT,
    pub wZpos:    UINT,
    pub wButtons: UINT,
}

/// Joystick genişletilmiş konum bilgisi (JOYINFOEX)
#[repr(C)]
#[allow(non_snake_case)]
pub struct JOYINFOEX {
    pub dwSize:        DWORD,
    pub dwFlags:       DWORD,
    pub dwXpos:        DWORD,
    pub dwYpos:        DWORD,
    pub dwZpos:        DWORD,
    pub dwRpos:        DWORD,
    pub dwUpos:        DWORD,
    pub dwVpos:        DWORD,
    pub dwButtons:     DWORD,
    pub dwButtonNumber:DWORD,
    pub dwPOV:         DWORD,
    pub dwReserved1:   DWORD,
    pub dwReserved2:   DWORD,
}

/// Mixer cihazı kapasitesi — ANSI (MIXERCAPSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERCAPSA {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [CHAR; 32],
    pub fdwSupport:     DWORD,
    pub cDestinations:  DWORD,
}

/// Mixer cihazı kapasitesi — Unicode (MIXERCAPSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERCAPSW {
    pub wMid:           WORD,
    pub wPid:           WORD,
    pub vDriverVersion: DWORD,
    pub szPname:        [WCHAR; 32],
    pub fdwSupport:     DWORD,
    pub cDestinations:  DWORD,
}

/// Mixer hat bilgisi — ANSI (MIXERLINEA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERLINEA {
    pub cbStruct:       DWORD,
    pub dwDestination:  DWORD,
    pub dwSource:       DWORD,
    pub dwLineID:       DWORD,
    pub fdwLine:        DWORD,
    pub dwUser:         DWORD_PTR,
    pub dwComponentType:DWORD,
    pub cChannels:      DWORD,
    pub cConnections:   DWORD,
    pub cControls:      DWORD,
    pub szShortName:    [CHAR; 16],
    pub szName:         [CHAR; 64],
    pub target_type:    DWORD,
    pub target_dwDeviceID: DWORD,
    pub target_wMid:    WORD,
    pub target_wPid:    WORD,
    pub target_vDriverVersion: DWORD,
    pub target_szPname: [CHAR; 32],
}

/// Mixer hat bilgisi — Unicode (MIXERLINEW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERLINEW {
    pub cbStruct:       DWORD,
    pub dwDestination:  DWORD,
    pub dwSource:       DWORD,
    pub dwLineID:       DWORD,
    pub fdwLine:        DWORD,
    pub dwUser:         DWORD_PTR,
    pub dwComponentType:DWORD,
    pub cChannels:      DWORD,
    pub cConnections:   DWORD,
    pub cControls:      DWORD,
    pub szShortName:    [WCHAR; 16],
    pub szName:         [WCHAR; 64],
    pub target_type:    DWORD,
    pub target_dwDeviceID: DWORD,
    pub target_wMid:    WORD,
    pub target_wPid:    WORD,
    pub target_vDriverVersion: DWORD,
    pub target_szPname: [WCHAR; 32],
}

/// Mixer kontrol detay tamponu (MIXERCONTROLDETAILS)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERCONTROLDETAILS {
    pub cbStruct:    DWORD,
    pub dwControlID: DWORD,
    pub cChannels:   DWORD,
    pub hwndOwner:   HWND,
    pub cbDetails:   DWORD,
    pub paDetails:   LPVOID,
}

/// Mixer hat kontrolleri sorgu yapısı — ANSI (MIXERLINECONTROLSA)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERLINECONTROLSA {
    pub cbStruct:      DWORD,
    pub dwLineID:      DWORD,
    pub dwControlID:   DWORD,
    pub cControls:     DWORD,
    pub cbmxctrl:      DWORD,
    pub pamxctrl:      LPVOID,
}

/// Mixer hat kontrolleri sorgu yapısı — Unicode (MIXERLINECONTROLSW)
#[repr(C)]
#[allow(non_snake_case)]
pub struct MIXERLINECONTROLSW {
    pub cbStruct:      DWORD,
    pub dwLineID:      DWORD,
    pub dwControlID:   DWORD,
    pub cControls:     DWORD,
    pub cbmxctrl:      DWORD,
    pub pamxctrl:      LPVOID,
}
