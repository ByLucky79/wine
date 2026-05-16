# ÖZKAN-OS Çoklu Mimari Gerçek Implementasyon Raporu
**Rust `#![no_std]` | 13 Mimari | VMware / QEMU / Gerçek PC**

> Bu rapor, mevcut kod tabanındaki eksikliklere dayanarak, gerçek donanımda ve sanallaştırıcı ortamlarda çalışacak şekilde nasıl implementasyon yapılması gerektiğini teknik detaylarıyla açıklar. Dosya isimleri `lib.rs`/`mod.rs` yerine görevini açıkça belirten isimlerle tanımlanmıştır.

---

## 0. DOSYA İSİMLENDİRME KURALI

`lib.rs` ve `mod.rs` isimleri yasaktır. Her dosya kendi görevini açıkça belirtmeli:

| Eski İsim (Yasak) | Yeni İsim (Zorunlu) | Açıklama |
|-------------------|---------------------|----------|
| `lib.rs` | `kernel_core.rs` | Çekirdek crate kökü |
| `mod.rs` | `architecture_hal.rs` | Mimari soyutlama katmanı |
| `x86_64/mod.rs` | `x86_64_hal.rs` | x86_64 mimari implementasyonu |
| `aarch64/mod.rs` | `aarch64_hal.rs` | ARM64 mimari implementasyonu |
| `drivers/storage/mod.rs` | `ahci_driver.rs` | AHCI SATA sürücüsü |
| `drivers/storage/mod.rs` | `nvme_driver.rs` | NVMe sürücüsü |
| `drivers/net/mod.rs` | `rtl8139_driver.rs` | RTL8139 ethernet sürücüsü |
| `kernel-net/mod.rs` | `tcpip_stack.rs` | TCP/IP ağ yığını |
| `drivers/gpu/mod.rs` | `i915_gpu.rs` | Intel GPU sürücüsü |
| `drivers/gpu/mod.rs` | `amdgpu_gpu.rs` | AMD GPU sürücüsü |
| `drivers/gpu/mod.rs` | `nouveau_gpu.rs` | NVIDIA GPU sürücüsü |
| `mm/mod.rs` | `memory_manager.rs` | Bellek yöneticisi |
| `scheduler/mod.rs` | `task_scheduler.rs` | Görev zamanlayıcı |
| `fs/vfs/mod.rs` | `virtual_filesystem.rs` | Sanal dosya sistemi |
| `fs/fat/mod.rs` | `fat_filesystem.rs` | FAT dosya sistemi |
| `fs/ozfs/mod.rs` | `ozkan_filesystem.rs` | ÖZKAN yerel dosya sistemi |

---

## 1. MEVCUT KOD TABANI GERÇEKLİK ANALİZİ

### 1.1 Gerçekten Çalışan Kodlar (✅ Gerçek Implementasyon)

| Modül | Dosya/Lokasyon | Durum | Açıklama |
|-------|----------------|-------|----------|
| **Boot Chain** | `BOOT/x86_64/boot_sector.asm` + `BOOT/common/` | ✅ %95 | Özel BIOS boot sektörü, ELF64 kernel yükleyici, VBE mod ayarı |
| **Kernel Entry** | `kernel/system/core/main.rs` (ilk 500 satır) | ✅ %90 | `kernel_main()`, `kernel_main_bare()`, temel `init_drivers()` |
| **VFS Core** | `kernel/fs/vfs/src/virtual_filesystem.rs` | ✅ %85 | `VfsNode`, `VfsFsType`, mount/unmount, `FileSystem` trait'i gerçek |
| **ÖzFS** | `kernel/fs/ozfs/src/ozkan_filesystem.rs` | ✅ %80 | ÖZKAN-OS yerel dosya sistemi, OZPT partition table, superblock |
| **FAT** | `kernel/fs/fat/src/fat_filesystem.rs` | ✅ %75 | FAT12/16/32 okuma/yazma, dizin gezme |
| **exFAT** | `kernel/fs/exfat/src/exfat_filesystem.rs` | ✅ %70 | exFAT okuma/yazma, allocation bitmap |
| **ext4** | `kernel/fs/ext4/src/ext4_filesystem.rs` | ✅ %65 | ext2/3/4 okuma (extents, journal yok) |
| **NTFS** | `kernel/fs/ntfs/src/ntfs_filesystem.rs` | ✅ %60 | NTFS MFT okuma, temel attribute parsing |
| **VBE Graphics** | `kernel/graphics/vbe_renderer.rs` | ✅ %90 | 1024x768@32bpp, çift tampon, temel primitive'ler |
| **PS/2 Input** | `kernel/hardware/drivers/input/ps2_controller.rs` | ✅ %85 | Klavye + fare scan code set 1/2, IRQ1/IRQ12 |
| **Serial HAL** | `kernel/hardware/drivers/serial/serial_hal.rs` | ✅ %90 | COM1-COM4, 16550A UART, polling + IRQ |
| **xHCI USB** | `kernel/drivers/usb/xhci_host.rs` | ✅ %70 | USB3.0 host init, port reset, temel TRB |
| **PCI** | `kernel/hardware/drivers/pci/pci_scanner.rs` | ✅ %80 | Config space okuma/yazma, bus scan, BAR ayrıştırma |
| **Sandbox** | `kernel/sandbox/kernel_sandbox.rs` (25 modül) | ✅ %95 | HVM, Behavior, Honeypot, CryptoSeal, TimeMachine |
| **Win32 Compat** | `kernel/compat/win32/win32_compat.rs` | ✅ %75 | ConsoleBuffer, PE Loader, WindowManager, GDI, VFS, SCM |
| **DOS Emulator** | `kernel/compat/dos/dos_emulator.rs` | ✅ %85 | 23 INT 21h fonksiyonu, IVT, dosya I/O, tarih/saat |
| **i18n** | `apps/system/i18n/language_system.rs` | ✅ %90 | LangId sistemi, Türkçe/İngilizce/Almanca/Fransızca |
| **Theme Engine** | `gui/theme/theme_engine.rs` | ✅ %85 | Renk paleti, font rendering, widget seti |
| **AppLoader** | `kernel/system/apploader/ozkan_apploader.rs` | ✅ %80 | OZNB format, ELF64 parsing, doğrulama |
| **Memory Manager** | `kernel/system/mm/memory_manager.rs` | ✅ %70 | Buddy allocator, slab, sayfalama (x86_64) |
| **Task Scheduler** | `kernel/system/scheduler/task_scheduler.rs` | ✅ %75 | Round-robin, priority queue, context switch (x86_64) |

### 1.2 Kısmi Implementasyon (🔄 Stub / Yarım Kod)

| Modül | Durum | Eksikler |
|-------|-------|----------|
| **AHCI** | 🔄 %40 | `probe()` + `reset()` var, `read()`/`write()` yok, IRQ handler boş |
| **NVMe** | 🔄 %20 | PCI probe var, admin queue oluşturma yok |
| **Network Stack** | 🔄 %10 | `kernel-net` crate'i var, `init()` boş, TCP/IP yok |
| **RTL8139** | 🔄 %15 | PCI probe + MAC okuma var, RX/TX ring yok |
| **e1000** | 🔄 %15 | PCI probe var, register init yok |
| **GPU - i915** | 🔄 %25 | `drm_i915` crate var, modeset stub, GEM yok |
| **GPU - amdgpu** | 🔄 %20 | `amdgpu` crate var, power management stub |
| **GPU - nouveau** | 🔄 %20 | `nouveau` crate var, NVxx ailesi decoder stub |
| **GPU - core** | 🔄 %30 | `ozkan_os_gpu.rs` var, `virt_to_phys()` stub, ring buffer init yok |
| **Audio HDA** | 🔄 %10 | PCI probe var, codec init yok |
| **ACPI** | 🔄 %40 | RSDP bulma var, AML interpreter yok |
| **SMP** | 🔄 %30 | BSP init var, AP bring-up yok (IPI, trampoline) |
| **PCIe MSI-X** | 🔄 %20 | Legacy IRQ var, MSI/MSI-X yapılandırma yok |

### 1.3 Sadece Yol Haritasında Var (❌ Henüz Kod Yok)

| Özellik | Prompt.txt Satırı | Durum |
|---------|-------------------|-------|
| **TCP/IP Stack** | ~1500 | ❌ `kernel-net/src/tcpip_stack.rs` içinde `pub fn init() {}` boş |
| **Userland init** | ~1600 | ❌ `start_native_service_chain()` ELF zinciri yok |
| **ozwm** | ~1700 | ❌ Window manager userland değil, kernel-space stub |
| **ozdesktop** | ~1750 | ❌ Desktop environment roadmap'ta |
| **SELinux** | ~2000 | ❌ Sadece isim geçiyor |
| **ASLR** | ~2100 | ❌ Memory randomization yok |
| **XDP/eBPF** | ~2200 | ❌ Network filter framework yok |
| **LivePatch** | ~2300 | ❌ `ozkan-debugger` crate'i var ama implementasyon yok |
| **OzRecovery** | ~2400 | ❌ Partition recovery stub |
| **KSM** | ~2500 | ❌ Kernel same-page merging yok |
| **ZRAM** | ~2550 | ❌ Compressed swap yok |

### 1.4 Kritik Donanım Hataları (Gerçek PC'de Çöker)

| # | Hata | Lokasyon | Sonuç |
|---|------|----------|-------|
| 1 | **GPU MMIO'ya sanal adres veriliyor** | `ozkan_os_gpu.rs:submit_command()` | GPU DMA sanal adresi fiziksel olarak çözer, kernel panic veya veri bozulması |
| 2 | **Fake MMIO adresi** | `gpgpu.rs:0xFE00_0000` | Gerçek donanımda bu adreste GPU yok, triple fault |
| 3 | **NVIDIA decoder bug** | `nvidia.rs:0x0F` mask | `0x50` olmalı, hiçbir kart eşleşmiyor |
| 4 | **Intel ring buffer base** | `ozkan_os_gpu.rs` | Ring buffer base address register programlanmamış |
| 5 | **`#![no_std]` modül içinde** | `highres_renderer.rs` | Derleme hatası, `std` kullanımı var |
| 6 | **`main.rs` 5329 satır** | `kernel/system/core/main.rs` | Monolitik, 1300+ satır inline FS wrapper, 4800 satır kaldırılmalı |

---

## 2. MİMARİ DESTEK MATRİSİ (13 Mimari)

### 2.1 Destek Durumu Özeti

| Mimari | Boot Kodu | HAL | Page Tables | IRQ | Context Switch | Timer | SMP | Durum |
|--------|-----------|-----|-------------|-----|----------------|-------|-----|-------|
| **x86_64** | ✅ Özel ASM | ✅ | ✅ 4-level | ✅ PIC/APIC | ✅ | ✅ PIT/HPET | 🔄 | Çalışıyor |
| **x86 (i686)** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece x86_64 |
| **aarch64** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **arm32** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **riscv64** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **riscv32** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **mips64** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **mips32** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **ppc64** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **ppc32** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **m68k** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **sparc** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |
| **loongarch64** | ❌ Yok | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Sadece yol haritası |

**Gerçek:** Sadece **x86_64** için boot-to-desktop akışı kodlanmış. Diğer 12 mimari için sadece `Cargo.toml` feature flag'leri (`kernel-hal-aarch64`, `kernel-hal-riscv`, vb.) var, ama bu crate'lerin içeriği boş veya stub.

---

## 3. x86_64 GERÇEK BOOT AKIŞI (Tek Çalışan Mimari)

### 3.1 Bellek Düzeni (Gerçek `linker.ld`)

```ld
OUTPUT_FORMAT("elf64-x86-64")
ENTRY(_start)

SECTIONS {
    . = 0xFFFFFFFF80000000;  /* Higher half kernel */
    
    .text : ALIGN(4K) {
        *(.text.boot)
        *(.text .text.*)
    }
    
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }
    
    .data : ALIGN(4K) {
        *(.data .data.*)
    }
    
    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss .bss.*)
    }
    
    _kernel_end = .;
}
```

### 3.2 Boot Süreci Adım Adım

**Adım 1: BIOS Boot Sector (`BOOT/x86_64/boot_sector.asm`)**

Gerçek dosya: `BOOT/x86_64/boot_sector.asm`

```
1. BIOS POST sonrası CS:IP = 0x0000:0x7C00
2. A20 hattı aç (fast A20: port 0x92, bit 1)
3. GDT yükle (flat segment: code 0x08, data 0x10)
4. Protected mode'a geç (CR0.PE = 1)
5. PAE etkinleştir (CR4.PAE = 1)
6. IA32_EFER.LME = 1 (Long mode enable)
7. 4-level page table oluştur:
   - PML4[511] -> PDP (0xFFFFFFFF80000000 için)
   - PDP[510] -> PD
   - PD[0] -> 2MB huge page (kernel fiziksel başlangıç)
8. CR3 = PML4 fiziksel adresi
9. Long mode'a geç (ljmp 0x08:.<target>)
10. 64-bit modda kernel_entry çağrılır
```

**Adım 2: Kernel Entry (`main.rs:kernel_main_bare`)**

Gerçek fonksiyon imzası:

```rust
#[no_mangle]
pub extern "C" fn kernel_main_bare(multiboot_info_phys: u64, _magic: u64) -> ! {
    // 1. Temel segment register'ları yükle
    // 2. Stack pointer kur (BOOT_STACK: [u8; 65536])
    // 3. BSS zero'la
    // 4. Boot info'yu parse et (Multiboot2 veya custom)
    // 5. Fiziksel -> Sanal çevirisi için offset hesapla
    // 6. `kernel_main()` çağır
}
```

**Adım 3: Driver Init (`init_drivers()`)**

```rust
pub fn init_drivers() {
    // Sıralama kritik:
    1. serial_init()      // COM1 115200 8N1 - panic mesajları için
    2. vbe_init()         // 1024x768@32bpp - ekran için
    3. pci_scan()         // Tüm cihazları bul
    4. ahci_probe()       // SATA diskler (boot device)
    5. ps2_init()         // Klavye/fare
    6. xhci_init()        // USB
    7. network_probe()    // Ethernet kartları (stub)
    8. gpu_probe()        // PCI display cihazları (stub)
    9. audio_probe()      // HDA (stub)
    10. apic_init()       // Yerel APIC, IO APIC
    11. timer_init()      // PIT/HPET/Local APIC timer
    12. scheduler_init()  // Task switch hazır
}
```

---

## 4. ÇOKLU MİMARİ HAL (Hardware Abstraction Layer)

### 4.1 Ortak Trait Tanımları (`kernel/architecture_hal.rs`)

Her mimari bu trait'leri implemente etmeli:

```rust
// kernel/architecture_hal.rs
#![no_std]

use core::fmt;

/// Page table seviyeleri
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageTableLevel {
    Pml4,    // x86_64
    Pdpt,    // x86_64, aarch64
    Pd,      // x86_64, aarch64, riscv
    Pt,      // Tümü
}

/// Page flags
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageFlags {
    pub present: bool,
    pub writable: bool,
    pub executable: bool,
    pub user_accessible: bool,
    pub cache_disable: bool,
    pub write_through: bool,
    pub huge: bool,
    pub global: bool,
}

/// Fiziksel sayfa çerçevesi yöneticisi
pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<u64>;  // Fiziksel adres (4K aligned)
    fn dealloc(&mut self, frame: u64);
    fn alloc_contiguous(&mut self, count: usize, align: usize) -> Option<u64>;
}

/// Sanal bellek yöneticisi
pub trait PageTable {
    fn map(&mut self, vaddr: u64, paddr: u64, flags: PageFlags, alloc: &mut dyn FrameAllocator) -> Result<(), MapError>;
    fn unmap(&mut self, vaddr: u64) -> Option<u64>;
    fn translate(&self, vaddr: u64) -> Option<u64>;
    fn flags(&self, vaddr: u64) -> Option<PageFlags>;
    fn flush_tlb(&self, vaddr: u64);
    fn flush_tlb_all(&self);
}

/// Kesme kontrolcüsü
pub trait InterruptController {
    fn init(&mut self);
    fn enable(&mut self, vector: u8);
    fn disable(&mut self, vector: u8);
    fn set_handler(&mut self, vector: u8, handler: extern "C" fn());
    fn eoi(&mut self, vector: u8);
    fn send_ipi(&self, dest: u32, vector: u8);
}

/// Zamanlayıcı
pub trait Timer {
    fn init(&mut self, freq_hz: u64);
    fn current_ticks(&self) -> u64;
    fn ms_to_ticks(&self, ms: u64) -> u64;
    fn ticks_to_ms(&self, ticks: u64) -> u64;
    fn set_deadline(&mut self, ticks: u64);
    fn periodic(&mut self, interval_ms: u64);
}

/// Context (register set)
#[derive(Clone, Copy, Debug)]
#[repr(C, align(16))]
pub struct CpuContext {
    pub x86_64_regs: X86_64Registers,
    pub aarch64_regs: AArch64Registers,
    pub riscv64_regs: Riscv64Registers,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct X86_64Registers {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rip: u64, pub rflags: u64,
    pub cs: u16, pub ds: u16, pub es: u16, pub fs: u16, pub gs: u16, pub ss: u16,
    pub fxsave: [u8; 512],
    pub cr3: u64,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct AArch64Registers {
    pub x0_x30: [u64; 31],
    pub sp_el0: u64,
    pub sp_el1: u64,
    pub elr_el1: u64,
    pub spsr_el1: u64,
    pub v0_v31: [u128; 32],
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Riscv64Registers {
    pub ra: u64, pub sp: u64, pub gp: u64, pub tp: u64,
    pub t0_t6: [u64; 7],
    pub s0_s11: [u64; 12],
    pub a0_a7: [u64; 8],
    pub pc: u64,
    pub sstatus: u64,
    pub satp: u64,
}

/// Mimari trait
pub trait ArchitectureHal {
    const NAME: &'static str;
    const POINTER_WIDTH: usize;
    const PAGE_SIZE: usize;
    const MAX_PHYS_ADDR: u64;
    
    fn init_early();
    fn init_interrupts() -> &'static dyn InterruptController;
    fn init_memory(boot_info: &BootInfo) -> &'static dyn PageTable;
    fn init_timer() -> &'static dyn Timer;
    
    fn context_switch(old: *mut CpuContext, new: *const CpuContext);
    fn idle();
    fn halt();
    fn reboot();
    fn shutdown();
    
    fn read_tsc() -> u64;
    fn cpu_id() -> u32;
    fn num_cpus() -> u32;
}

#[cfg(target_arch = "x86_64")]
pub type CurrentArch = X86_64Hal;

#[cfg(target_arch = "aarch64")]
pub type CurrentArch = AArch64Hal;

#[cfg(target_arch = "riscv64")]
pub type CurrentArch = Riscv64Hal;
```

### 4.2 x86_64 Implementasyonu (`kernel/x86_64_hal.rs`)

```rust
// kernel/x86_64_hal.rs
#![no_std]

use super::{ArchitectureHal, PageTable, PageFlags, FrameAllocator, CpuContext, InterruptController, Timer};

pub struct X86_64Hal;

impl ArchitectureHal for X86_64Hal {
    const NAME: &'static str = "x86_64";
    const POINTER_WIDTH: usize = 64;
    const PAGE_SIZE: usize = 4096;
    const MAX_PHYS_ADDR: u64 = (1 << 52) - 1;
    
    fn init_early() {
        // 1. SSE etkinleştir (CR0.EM=0, CR0.MP=1, CR4.OSFXSR=1, CR4.OSXMMEXCPT=1)
        unsafe {
            let mut cr0: u64;
            core::arch::asm!("mov {}, cr0", out(reg) cr0);
            cr0 &= !(1 << 2);   // EM = 0
            cr0 |= 1 << 1;      // MP = 1
            core::arch::asm!("mov cr0, {}", in(reg) cr0);
            
            let mut cr4: u64;
            core::arch::asm!("mov {}, cr4", out(reg) cr4);
            cr4 |= 1 << 9;      // OSFXSR
            cr4 |= 1 << 10;     // OSXMMEXCPT
            core::arch::asm!("mov cr4, {}", in(reg) cr4);
        }
        
        // 2. FPU init (FNINIT)
        unsafe { core::arch::asm!("fninit") };
        
        // 3. PAT (Page Attribute Table) yapılandır
        let pat = 0x0007040600070406u64;
        unsafe {
            core::arch::asm!("wrmsr", in("ecx") 0x277, in("edx") (pat >> 32) as u32, in("eax") pat as u32);
        }
    }
    
    fn init_interrupts() -> &'static dyn InterruptController {
        // PIC remap, IO APIC init, Local APIC init, IDT oluştur
        &X86_64_INTERRUPT_CONTROLLER
    }
    
    fn init_memory(_boot_info: &BootInfo) -> &'static dyn PageTable {
        // E820 parse, buddy init, 4-level page table, higher half mapping
        &X86_64_PAGE_TABLE
    }
    
    fn init_timer() -> &'static dyn Timer {
        // PIT/HPET/Local APIC timer init
        &X86_64_TIMER
    }
    
    fn context_switch(old: *mut CpuContext, new: *const CpuContext) {
        unsafe {
            core::arch::asm!(
                "push rax", "push rbx", "push rcx", "push rdx",
                "push rsi", "push rdi", "push rbp",
                "push r8", "push r9", "push r10", "push r11",
                "push r12", "push r13", "push r14", "push r15",
                "pushfq",
                "mov [{old_rsp}], rsp",
                "mov rax, cr3",
                "mov [{old_cr3}], rax",
                "mov rsp, [{new_rsp}]",
                "mov rax, [{new_cr3}]",
                "mov cr3, rax",
                "fxrstor [{new_fx}]",
                "mov ax, [{new_ds}]",
                "mov ds, ax", "mov es, ax", "mov fs, ax", "mov gs, ax",
                "mov rax, [{new_rax}]",
                "mov rbx, [{new_rbx}]",
                // ... diğer register'lar
                "push [{new_rflags}]",
                "popfq",
                "push [{new_cs}]",
                "push [{new_rip}]",
                "iretq",
                old_rsp = in(reg) &(*old).x86_64_regs.rsp,
                old_cr3 = in(reg) &(*old).x86_64_regs.cr3,
                new_rsp = in(reg) &(*new).x86_64_regs.rsp,
                new_cr3 = in(reg) &(*new).x86_64_regs.cr3,
                options(noreturn)
            );
        }
    }
    
    fn idle() {
        unsafe { core::arch::asm!("sti; hlt") }
    }
    
    fn halt() -> ! {
        unsafe { core::arch::asm!("cli; hlt", options(noreturn)) }
    }
    
    fn reboot() {
        unsafe {
            while inb(0x64) & 0x02 != 0 {}
            outb(0x64, 0xFE);
        }
    }
    
    fn shutdown() {
        unsafe {
            outw(0x604, 0x2000);   // QEMU
            outw(0xB004, 0x2000);  // Bochs
            outw(0x4004, 0x3400);  // VirtualBox
        }
    }
    
    fn read_tsc() -> u64 {
        let lo: u32;
        let hi: u32;
        unsafe { core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi) }
        ((hi as u64) << 32) | (lo as u64)
    }
    
    fn cpu_id() -> u32 {
        let mut id: u32 = 0;
        unsafe { core::arch::asm!("mov {}, ebx", out(reg) id) }
        id
    }
    
    fn num_cpus() -> u32 {
        1  // Şimdilik BSP only
    }
}
```

### 4.3 aarch64 Implementasyonu (`kernel/aarch64_hal.rs`)

```rust
// kernel/aarch64_hal.rs
#![no_std]

pub struct AArch64Hal;

impl ArchitectureHal for AArch64Hal {
    const NAME: &'static str = "aarch64";
    const POINTER_WIDTH: usize = 64;
    const PAGE_SIZE: usize = 4096;
    const MAX_PHYS_ADDR: u64 = (1 << 48) - 1;
    
    fn init_early() {
        // 1. EL2'den EL1'e düş
        // 2. MMU'yu kapat
        // 3. Exception vector table kur (VBAR_EL1)
        // 4. GICv3/v4 init
        // 5. ARM Generic Timer init
    }
    
    fn init_memory(_boot_info: &BootInfo) -> &'static dyn PageTable {
        // aarch64 page table (4KB granule, 48-bit VA):
        // Level 0 (PGD): 512 entries, each covers 512GB
        // Level 1 (PUD): 512 entries, each covers 1GB
        // Level 2 (PMD): 512 entries, each covers 2MB
        // Level 3 (PTE): 512 entries, each covers 4KB
        
        // MAIR_EL1: Attr0=0xFF (WB), Attr1=0x04 (Device-nGnRnE)
        // TCR_EL1: T0SZ=16, TG0=0 (4KB), IPS=2 (40-bit)
        
        &AARCH64_PAGE_TABLE
    }
    
    fn context_switch(old: *mut CpuContext, new: *const CpuContext) {
        unsafe {
            core::arch::asm!(
                "stp x19, x20, [sp, #-16]!",
                "stp x21, x22, [sp, #-16]!",
                "stp x23, x24, [sp, #-16]!",
                "stp x25, x26, [sp, #-16]!",
                "stp x27, x28, [sp, #-16]!",
                "stp x29, x30, [sp, #-16]!",
                "mrs x9, sp_el0",
                "str x9, [{old_sp_el0}]",
                "mrs x9, elr_el1",
                "str x9, [{old_elr}]",
                "mrs x9, spsr_el1",
                "str x9, [{old_spsr}]",
                "mov x9, sp",
                "str x9, [{old_sp}]",
                "ldr x9, [{new_sp}]",
                "mov sp, x9",
                "ldr x9, [{new_sp_el0}]",
                "msr sp_el0, x9",
                "ldr x9, [{new_elr}]",
                "msr elr_el1, x9",
                "ldr x9, [{new_spsr}]",
                "msr spsr_el1, x9",
                "ldp x29, x30, [sp], #16",
                "ldp x27, x28, [sp], #16",
                "ldp x25, x26, [sp], #16",
                "ldp x23, x24, [sp], #16",
                "ldp x21, x22, [sp], #16",
                "ldp x19, x20, [sp], #16",
                "eret",
                options(noreturn)
            );
        }
    }
    
    fn read_tsc() -> u64 {
        let cnt: u64;
        unsafe { core::arch::asm!("mrs {}, cntvct_el0", out(reg) cnt) }
        cnt
    }
    
    fn cpu_id() -> u32 {
        let mpidr: u64;
        unsafe { core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr) }
        (mpidr & 0xFF) as u32
    }
}
```

### 4.4 riscv64 Implementasyonu (`kernel/riscv64_hal.rs`)

```rust
// kernel/riscv64_hal.rs
#![no_std]

pub struct Riscv64Hal;

impl ArchitectureHal for Riscv64Hal {
    const NAME: &'static str = "riscv64";
    const POINTER_WIDTH: usize = 64;
    const PAGE_SIZE: usize = 4096;
    const MAX_PHYS_ADDR: u64 = (1 << 56) - 1;
    
    fn init_early() {
        // 1. M-mode -> S-mode geçiş
        // 2. PMP yapılandır (tüm belleğe S-mode erişimi)
        // 3. SATP: MODE=9 (Sv48)
        // 4. mideleg/medelege: Interrupt/fault delegation
        // 5. sscratch: hart-local trap frame
        // 6. stvec: Vectored mode (MODE=1)
    }
    
    fn init_memory(_boot_info: &BootInfo) -> &'static dyn PageTable {
        // Sv48: 4 levels, 512 entries per level
        // satp.MODE = 9
        // PTE: V/R/W/X/U/G/A/D/PPN
        &RISCV64_PAGE_TABLE
    }
    
    fn context_switch(old: *mut CpuContext, new: *const CpuContext) {
        unsafe {
            core::arch::asm!(
                "sd ra, 0({old})",
                "sd sp, 8({old})",
                "sd s0, 16({old})",
                "sd s1, 24({old})",
                // ... s2-s11
                "csrr t0, sepc",
                "sd t0, 128({old})",
                "csrr t0, sstatus",
                "sd t0, 136({old})",
                "csrr t0, satp",
                "sd t0, 144({old})",
                "ld t0, 144({new})",
                "csrw satp, t0",
                "sfence.vma zero, zero",
                "ld t0, 128({new})",
                "csrw sepc, t0",
                "ld t0, 136({new})",
                "csrw sstatus, t0",
                "ld ra, 0({new})",
                "ld sp, 8({new})",
                "ld s0, 16({new})",
                // ... s2-s11
                "sret",
                options(noreturn)
            );
        }
    }
    
    fn read_tsc() -> u64 {
        let cycles: u64;
        unsafe { core::arch::asm!("rdcycle {}", out(reg) cycles) }
        cycles
    }
    
    fn cpu_id() -> u32 {
        let hartid: u64;
        unsafe { core::arch::asm!("csrr {}, mhartid", out(reg) hartid) }
        hartid as u32
    }
}
```

---

## 5. GERÇEK DONANIM ERİŞİM KODLARI

### 5.1 PCI/PCIe Config Space (`kernel/pci_access.rs`)

```rust
// kernel/pci_access.rs
#![no_std]

/// PCI configuration space access (I/O port)
pub mod pci_io_access {
    const CONFIG_ADDRESS: u16 = 0xCF8;
    const CONFIG_DATA: u16 = 0xCFC;
    
    pub unsafe fn read32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address: u32 = ((bus as u32) << 16)
                         | ((device as u32) << 11)
                         | ((function as u32) << 8)
                         | ((offset as u32) & 0xFC)
                         | 0x8000_0000;
        core::arch::asm!("out dx, eax", in("dx") CONFIG_ADDRESS, in("eax") address);
        let val: u32;
        core::arch::asm!("in eax, dx", out("eax") val, in("dx") CONFIG_DATA);
        val
    }
    
    pub unsafe fn write32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
        let address: u32 = ((bus as u32) << 16)
                         | ((device as u32) << 11)
                         | ((function as u32) << 8)
                         | ((offset as u32) & 0xFC)
                         | 0x8000_0000;
        core::arch::asm!("out dx, eax", in("dx") CONFIG_ADDRESS, in("eax") address);
        core::arch::asm!("out dx, eax", in("dx") CONFIG_DATA, in("eax") value);
    }
}

/// PCIe ECAM (Enhanced Configuration Access Mechanism)
pub mod pcie_ecam_access {
    const ECAM_BASE: u64 = 0xE000_0000;
    
    pub unsafe fn read32(bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        let addr = ECAM_BASE
                 + ((bus as u64) << 20)
                 + ((device as u64) << 15)
                 + ((function as u64) << 12)
                 + (offset as u64);
        core::ptr::read_volatile(addr as *const u32)
    }
    
    pub unsafe fn write32(bus: u8, device: u8, function: u8, offset: u16, value: u32) {
        let addr = ECAM_BASE
                 + ((bus as u64) << 20)
                 + ((device as u64) << 15)
                 + ((function as u64) << 12)
                 + (offset as u64);
        core::ptr::write_volatile(addr as *mut u32, value);
    }
}

/// BAR (Base Address Register) parse
pub struct PciBarInfo;

impl PciBarInfo {
    pub unsafe fn parse_bar32(original: u32, size_mask: u32) -> BarResult {
        if original & 1 == 0 {
            let prefetchable = (original & 8) != 0;
            let typ = (original >> 1) & 3;
            if typ == 2 {
                BarResult::Memory64Reserved
            } else {
                let base = (original & 0xFFFF_FFF0) as u64;
                let size = (!(size_mask & 0xFFFF_FFF0) + 1) as u64;
                BarResult::Memory32 { base, size, prefetchable }
            }
        } else {
            let base = (original & 0xFFFF_FFFC) as u16;
            let size = (!(size_mask & 0xFFFF_FFFC) + 1) as u16;
            BarResult::Io { base, size }
        }
    }
}

pub enum BarResult {
    Memory32 { base: u64, size: u64, prefetchable: bool },
    Memory64Reserved,
    Io { base: u16, size: u16 },
}
```

### 5.2 x86_64 I/O Port Erişimi (`kernel/x86_io_access.rs`)

```rust
// kernel/x86_io_access.rs
#![no_std]

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack));
    value
}

#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    core::arch::asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack));
    value
}

#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack));
    value
}

#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

#[inline]
pub unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
}

#[inline]
pub unsafe fn outl(port: u16, value: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
}

#[inline]
pub unsafe fn io_wait() {
    core::arch::asm!("out 0x80, al", in("al") 0u8, options(nomem, nostack));
}
```

### 5.3 aarch64 MMIO Erişimi (`kernel/aarch64_mmio_access.rs`)

```rust
// kernel/aarch64_mmio_access.rs
#![no_std]

#[inline]
pub unsafe fn read32(addr: u64) -> u32 {
    core::ptr::read_volatile(addr as *const u32)
}

#[inline]
pub unsafe fn read64(addr: u64) -> u64 {
    core::ptr::read_volatile(addr as *const u64)
}

#[inline]
pub unsafe fn write32(addr: u64, value: u32) {
    core::ptr::write_volatile(addr as *mut u32, value);
}

#[inline]
pub unsafe fn write64(addr: u64, value: u64) {
    core::ptr::write_volatile(addr as *mut u64, value);
}

/// GICv3 register erişimi
pub unsafe fn gicr_read_typer(gicr_base: u64) -> u64 {
    read64(gicr_base + 0x0008)
}

pub unsafe fn gicd_set_irouter(gicd_base: u64, irq: u16, affinity: u64) {
    let offset = 0x6000 + (irq as u64) * 8;
    write64(gicd_base + offset, affinity);
}
```

---

## 6. GERÇEK SÜRÜCÜ IMPLEMENTASYON ÖRNEKLERİ

### 6.1 AHCI SATA Controller (`drivers/storage/ahci_driver.rs`)

```rust
// drivers/storage/ahci_driver.rs
#![no_std]

use ozkan_os_pci::{PciDevice, BarInfo};
use ozkan_os_driver_core::{Driver, DeviceClass, DriverState};

/// AHCI BAR5 (ABAR) offset'leri
const AHCI_CAP: u32 = 0x00;
const AHCI_GHC: u32 = 0x04;
const AHCI_IS: u32 = 0x08;
const AHCI_PI: u32 = 0x0C;
const AHCI_VS: u32 = 0x10;

/// Port register offset (her port 0x80 byte)
const PORT_CLB: u32 = 0x00;
const PORT_CLBU: u32 = 0x04;
const PORT_FB: u32 = 0x08;
const PORT_FBU: u32 = 0x0C;
const PORT_IS: u32 = 0x10;
const PORT_IE: u32 = 0x14;
const PORT_CMD: u32 = 0x18;
const PORT_TFD: u32 = 0x20;
const PORT_SIG: u32 = 0x24;
const PORT_SSTS: u32 = 0x28;
const PORT_SCTL: u32 = 0x2C;
const PORT_SERR: u32 = 0x30;
const PORT_SACT: u32 = 0x34;
const PORT_CI: u32 = 0x38;

/// Port Signature değerleri
const SIG_ATA: u32 = 0x0000_0101;
const SIG_ATAPI: u32 = 0xEB14_0101;
const SIG_SEMB: u32 = 0xC33C_0101;
const SIG_PM: u32 = 0x9669_0101;

#[repr(C, align(128))]
struct CommandHeader {
    dw0: u32,
    dw1: u32,
    ctba: u32,
    ctbau: u32,
    _rsv: [u32; 4],
}

#[repr(C)]
struct PrdtEntry {
    dba: u32,
    dbau: u32,
    _rsv: u32,
    dw3: u32,
}

pub struct AhciController {
    abar: u64,
    ports: [Option<AhciPort>; 32],
    dma_pool: DmaPool,
}

impl AhciController {
    pub fn new(pci_dev: &PciDevice) -> Result<Self, AhciError> {
        let bar5 = pci_dev.bar(5).ok_or(AhciError::NoBar5)?;
        let abar = match bar5 {
            BarInfo::Memory32 { base, .. } => base,
            BarInfo::Memory64 { base, .. } => base,
            _ => return Err(AhciError::InvalidBar),
        };
        
        let mut cmd = pci_dev.read16(0x04);
        cmd |= 0x04;
        pci_dev.write16(0x04, cmd);
        cmd |= 0x02;
        pci_dev.write16(0x04, cmd);
        
        let mut ghc = unsafe { read32(abar + AHCI_GHC as u64) };
        ghc |= 1;
        unsafe { write32(abar + AHCI_GHC as u64, ghc) }
        
        let cap = unsafe { read32(abar + AHCI_CAP as u64) };
        if cap & (1 << 31) != 0 {
            let bohc = unsafe { read32(abar + 0x28) };
            if bohc & 1 != 0 {
                unsafe { write32(abar + 0x28, bohc | 2) }
            }
        }
        
        Ok(Self {
            abar,
            ports: [None; 32],
            dma_pool: DmaPool::new()?,
        })
    }
    
    pub fn probe_ports(&mut self) -> u32 {
        let pi = unsafe { read32(self.abar + AHCI_PI as u64) };
        let cap = unsafe { read32(self.abar + AHCI_CAP as u64) };
        let ncs = ((cap >> 8) & 0x1F) as usize + 1;
        
        let mut found = 0;
        for i in 0..32 {
            if pi & (1 << i) == 0 { continue; }
            
            let port_base = self.abar + 0x100 + (i as u64) * 0x80;
            let ssts = unsafe { read32(port_base + PORT_SSTS as u64) };
            let det = ssts & 0x0F;
            
            if det != 3 { continue; }
            
            let sig = unsafe { read32(port_base + PORT_SIG as u64) };
            let port_type = match sig {
                SIG_ATA => AhciPortType::Sata,
                SIG_ATAPI => AhciPortType::Atapi,
                SIG_SEMB => AhciPortType::Semb,
                SIG_PM => AhciPortType::PortMultiplier,
                _ => AhciPortType::Unknown,
            };
            
            let cmd_list_size = 32 * ncs;
            let received_fis_size = 256;
            
            let cmd_list = self.dma_pool.alloc(cmd_list_size, 1024)
                .expect("AHCI command list alloc failed");
            let received_fis = self.dma_pool.alloc(received_fis_size, 256)
                .expect("AHCI FIS alloc failed");
            
            unsafe {
                core::ptr::write_bytes(cmd_list as *mut u8, 0, cmd_list_size);
                core::ptr::write_bytes(received_fis as *mut u8, 0, received_fis_size);
            }
            
            unsafe {
                write32(port_base + PORT_CLB as u64, cmd_list as u32);
                write32(port_base + PORT_CLBU as u64, (cmd_list >> 32) as u32);
                write32(port_base + PORT_FB as u64, received_fis as u32);
                write32(port_base + PORT_FBU as u64, (received_fis >> 32) as u32);
            }
            
            let mut cmd = unsafe { read32(port_base + PORT_CMD as u64) };
            cmd |= (1 << 4);
            cmd |= 1;
            unsafe { write32(port_base + PORT_CMD as u64, cmd) }
            
            self.ports[i] = Some(AhciPort {
                index: i as u8,
                port_type,
                base: port_base,
                cmd_list,
                received_fis,
                ncs,
                next_cmd_slot: 0,
            });
            
            found += 1;
        }
        
        found
    }
    
    pub fn read_sectors(&self, port_idx: usize, lba: u64, count: u16, buf: &mut [u8]) -> Result<(), AhciError> {
        let port = self.ports[port_idx].as_ref().ok_or(AhciError::NoDevice)?;
        
        if count == 0 || count > 256 {
            return Err(AhciError::InvalidCount);
        }
        if buf.len() < (count as usize) * 512 {
            return Err(AhciError::BufferTooSmall);
        }
        
        let dma_buf = self.dma_pool.alloc_contiguous((count as usize) * 512, 512)
            .ok_or(AhciError::DmaAllocFailed)?;
        
        let slot = self.find_free_slot(port)?;
        
        let cmd_header = unsafe { &mut *((port.cmd_list + slot as u64 * 32) as *mut CommandHeader) };
        cmd_header.dw0 = (5 << 0) | (1 << 16);
        
        let cmd_table_size = 0x80 + (1 * 16);
        let cmd_table = self.dma_pool.alloc(cmd_table_size, 128)
            .ok_or(AhciError::DmaAllocFailed)?;
        
        cmd_header.ctba = cmd_table as u32;
        cmd_header.ctbau = (cmd_table >> 32) as u32;
        
        let fis = unsafe { core::slice::from_raw_parts_mut(cmd_table as *mut u8, 20) };
        fis[0] = 0x27;
        fis[1] = 0x80 | (1 << 7);
        fis[2] = 0x25;
        fis[3] = 0x00;
        fis[4] = (lba >> 0) as u8;
        fis[5] = (lba >> 8) as u8;
        fis[6] = (lba >> 16) as u8;
        fis[7] = 0x40 | ((lba >> 24) as u8 & 0x0F);
        fis[8] = (lba >> 32) as u8;
        fis[9] = (lba >> 40) as u8;
        fis[12] = (count >> 0) as u8;
        fis[13] = (count >> 8) as u8;
        fis[15] = 0x00;
        
        let prdt = unsafe { &mut *((cmd_table + 0x80) as *mut PrdtEntry) };
        prdt.dba = dma_buf as u32;
        prdt.dbau = (dma_buf >> 32) as u32;
        prdt.dw3 = ((count as u32) * 512) - 1;
        
        unsafe { write32(port.base + PORT_CI as u64, 1 << slot) }
        
        let mut timeout = 1_000_000;
        while timeout > 0 {
            let ci = unsafe { read32(port.base + PORT_CI as u64) };
            if ci & (1 << slot) == 0 { break; }
            timeout -= 1;
            core::hint::spin_loop();
        }
        
        if timeout == 0 {
            return Err(AhciError::Timeout);
        }
        
        let tfd = unsafe { read32(port.base + PORT_TFD as u64) };
        if tfd & (1 << 0) != 0 {
            return Err(AhciError::AtaError);
        }
        
        unsafe {
            core::ptr::copy_nonoverlapping(
                dma_buf as *const u8,
                buf.as_mut_ptr(),
                (count as usize) * 512
            );
        }
        
        self.dma_pool.dealloc(cmd_table, cmd_table_size);
        self.dma_pool.dealloc(dma_buf, (count as usize) * 512);
        
        Ok(())
    }
    
    fn find_free_slot(&self, port: &AhciPort) -> Result<u8, AhciError> {
        let sact = unsafe { read32(port.base + PORT_SACT as u64) };
        let ci = unsafe { read32(port.base + PORT_CI as u64) };
        let used = sact | ci;
        
        for i in 0..port.ncs {
            if used & (1 << i) == 0 {
                return Ok(i as u8);
            }
        }
        
        Err(AhciError::NoFreeSlot)
    }
}

pub struct DmaPool {
    base: u64,
    size: usize,
    used: spin::Mutex<Bitmap>,
}

impl DmaPool {
    pub fn new() -> Result<Self, DmaError> {
        unimplemented!("DMA pool init")
    }
    
    pub fn alloc(&self, size: usize, align: usize) -> Option<u64> {
        unimplemented!("DMA alloc")
    }
    
    pub fn alloc_contiguous(&self, size: usize, align: usize) -> Option<u64> {
        unimplemented!("DMA contiguous alloc")
    }
    
    pub fn dealloc(&self, addr: u64, size: usize) {
        unimplemented!("DMA dealloc")
    }
}
```

### 6.2 NVMe Controller (`drivers/storage/nvme_driver.rs`)

```rust
// drivers/storage/nvme_driver.rs
#![no_std]

const NVME_PCI_CLASS: u8 = 0x01;
const NVME_PCI_SUBCLASS: u8 = 0x08;

const CAP: u32 = 0x00;
const VS: u32 = 0x08;
const INTMS: u32 = 0x0C;
const INTMC: u32 = 0x10;
const CC: u32 = 0x14;
const CSTS: u32 = 0x1C;
const AQA: u32 = 0x24;
const ASQ: u32 = 0x28;
const ACQ: u32 = 0x30;

const CC_EN: u32 = 1 << 0;
const CC_CSS_NVM: u32 = 0 << 4;
const CC_MPS_SHIFT: u32 = 7;

const CSTS_RDY: u32 = 1 << 0;
const CSTS_CFS: u32 = 1 << 1;

#[repr(C, align(64))]
pub struct NvmeSubmissionQueueEntry {
    cdw0: u32,
    nsid: u32,
    cdw2: u32,
    cdw3: u32,
    mptr: u64,
    dptr: [u64; 2],
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
}

#[repr(C, align(16))]
pub struct NvmeCompletionQueueEntry {
    cdw0: u32,
    rsvd: u32,
    sqhd: u16,
    sqid: u16,
    cid: u16,
    sf: u16,
}

const ADMIN_CREATE_IO_CQ: u8 = 0x05;
const ADMIN_CREATE_IO_SQ: u8 = 0x01;
const ADMIN_IDENTIFY: u8 = 0x06;
const ADMIN_SET_FEATURES: u8 = 0x09;

const NVM_WRITE: u8 = 0x01;
const NVM_READ: u8 = 0x02;
const NVM_FLUSH: u8 = 0x00;

pub struct NvmeController {
    regs: u64,
    doorbell_stride: u32,
    page_size: u32,
    admin_sq: DmaBuffer<NvmeSubmissionQueueEntry>,
    admin_cq: DmaBuffer<NvmeCompletionQueueEntry>,
    admin_sq_tail: u16,
    admin_cq_head: u16,
    admin_phase: bool,
    io_sq: DmaBuffer<NvmeSubmissionQueueEntry>,
    io_cq: DmaBuffer<NvmeCompletionQueueEntry>,
    io_sq_tail: u16,
    io_cq_head: u16,
    io_phase: bool,
    namespaces: heapless::Vec<NvmeNamespace, 32>,
}

impl NvmeController {
    pub fn new(pci_dev: &PciDevice) -> Result<Self, NvmeError> {
        let bar0 = match pci_dev.bar(0) {
            Some(BarInfo::Memory64 { base, .. }) => base,
            Some(BarInfo::Memory32 { base, .. }) => base,
            _ => return Err(NvmeError::InvalidBar),
        };
        
        let cmd = pci_dev.read16(0x04);
        pci_dev.write16(0x04, cmd | 0x06);
        
        let regs = bar0;
        
        let mut cc = unsafe { read32(regs + CC as u64) };
        cc &= !CC_EN;
        unsafe { write32(regs + CC as u64, cc) }
        
        let mut timeout = 500_000;
        while timeout > 0 {
            let csts = unsafe { read32(regs + CSTS as u64) };
            if csts & CSTS_RDY == 0 { break; }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(NvmeError::ResetTimeout);
        }
        
        let cap_lo = unsafe { read32(regs + CAP as u64) };
        let cap_hi = unsafe { read32(regs + (CAP + 4) as u64) };
        let cap = ((cap_hi as u64) << 32) | (cap_lo as u64);
        
        let dstrd = ((cap >> 32) & 0x0F) as u32;
        let mqes = (cap & 0xFFFF) as u16;
        
        let doorbell_stride = 4 << dstrd;
        
        let aqa = ((mqes as u32) << 16) | (mqes as u32);
        let admin_sq = DmaBuffer::alloc(((mqes as usize) + 1) * 64, 4096)?;
        let admin_cq = DmaBuffer::alloc(((mqes as usize) + 1) * 16, 4096)?;
        
        unsafe {
            write32(regs + AQA as u64, aqa);
            write64(regs + ASQ as u64, admin_sq.phys() as u64);
            write64(regs + ACQ as u64, admin_cq.phys() as u64);
        }
        
        let cc = CC_EN | (0 << CC_MPS_SHIFT) | CC_CSS_NVM;
        unsafe { write32(regs + CC as u64, cc) }
        
        timeout = 500_000;
        while timeout > 0 {
            let csts = unsafe { read32(regs + CSTS as u64) };
            if csts & CSTS_RDY != 0 { break; }
            if csts & CSTS_CFS != 0 {
                return Err(NvmeError::FatalStatus);
            }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(NvmeError::EnableTimeout);
        }
        
        let mut id_ctrl_buf = DmaBuffer::alloc(4096, 4096)?;
        Self::admin_cmd_identify(&admin_sq, &admin_cq, 1, 0, id_ctrl_buf.phys() as u64)?;
        
        let mut ns_list_buf = DmaBuffer::alloc(4096, 4096)?;
        Self::admin_cmd_identify(&admin_sq, &admin_cq, 2, 0, ns_list_buf.phys() as u64)?;
        
        Ok(Self {
            regs,
            doorbell_stride,
            page_size: 4096,
            admin_sq,
            admin_cq,
            admin_sq_tail: 0,
            admin_cq_head: 0,
            admin_phase: true,
            io_sq: DmaBuffer::uninit(),
            io_cq: DmaBuffer::uninit(),
            io_sq_tail: 0,
            io_cq_head: 0,
            io_phase: true,
            namespaces: heapless::Vec::new(),
        })
    }
    
    fn submit_admin_cmd(&mut self, cmd: &NvmeSubmissionQueueEntry) -> Result<NvmeCompletionQueueEntry, NvmeError> {
        let tail = self.admin_sq_tail as usize;
        unsafe {
            core::ptr::write(self.admin_sq.virt().add(tail), *cmd);
        }
        
        self.admin_sq_tail = (self.admin_sq_tail + 1) % self.admin_sq.len();
        
        unsafe {
            write32(self.regs + 0x1000, self.admin_sq_tail as u32);
        }
        
        let mut timeout = 1_000_000;
        while timeout > 0 {
            let cqe = unsafe { &*self.admin_cq.virt().add(self.admin_cq_head as usize) };
            let phase = (cqe.sf >> 15) & 1 != 0;
            
            if phase == self.admin_phase {
                let result = *cqe;
                
                self.admin_cq_head = (self.admin_cq_head + 1) % self.admin_cq.len();
                if self.admin_cq_head == 0 {
                    self.admin_phase = !self.admin_phase;
                }
                
                unsafe {
                    write32(self.regs + 0x1000 + self.doorbell_stride as u64, 
                           self.admin_cq_head as u32);
                }
                
                let sc = result.sf & 0xFF;
                let sct = (result.sf >> 9) & 0x07;
                if sc != 0 || sct != 0 {
                    return Err(NvmeError::CommandStatus { sc, sct });
                }
                
                return Ok(result);
            }
            
            timeout -= 1;
            core::hint::spin_loop();
        }
        
        Err(NvmeError::Timeout)
    }
}
```

---

## 7. AĞ SÜRÜCÜLERİ VE TCP/IP YIĞINI

### 7.1 RTL8139D Gerçek Implementasyonu (`drivers/net/rtl8139_driver.rs`)

```rust
// drivers/net/rtl8139_driver.rs
#![no_std]

const RTL8139_VENDOR: u16 = 0x10EC;
const RTL8139_DEVICE: u16 = 0x8139;

const MAC0_5: u16 = 0x00;
const MAR0_7: u16 = 0x08;
const TX_STATUS0: u16 = 0x10;
const TX_ADDR0: u16 = 0x20;
const RX_BUF: u16 = 0x30;
const CMD: u16 = 0x37;
const RX_PTR: u16 = 0x38;
const RX_ADDR: u16 = 0x3A;
const IMR: u16 = 0x3C;
const ISR: u16 = 0x3E;
const TX_CONFIG: u16 = 0x40;
const RX_CONFIG: u16 = 0x44;
const CONFIG1: u16 = 0x52;
const MSR: u16 = 0x58;
const BMCR: u16 = 0x62;
const BMSR: u16 = 0x64;

const CMD_RST: u8 = 1 << 4;
const CMD_RE: u8 = 1 << 3;
const CMD_TE: u8 = 1 << 2;
const CMD_BUFE: u8 = 1 << 0;

const RX_CONFIG_AAP: u32 = 1 << 0;
const RX_CONFIG_APM: u32 = 1 << 1;
const RX_CONFIG_AM: u32 = 1 << 2;
const RX_CONFIG_AR: u32 = 1 << 3;
const RX_CONFIG_AB: u32 = 1 << 4;
const RX_CONFIG_WRAP: u32 = 1 << 7;
const RX_CONFIG_MXDMA_UNLIMITED: u32 = 7 << 8;
const RX_CONFIG_RBLN_8K: u32 = 0 << 11;

const TX_CONFIG_MXDMA_2048: u32 = 7 << 8;
const TX_CONFIG_IFG_NORMAL: u32 = 3 << 24;

const INT_ROK: u16 = 1 << 0;
const INT_RER: u16 = 1 << 1;
const INT_TOK: u16 = 1 << 2;
const INT_TER: u16 = 1 << 3;
const INT_RXOVW: u16 = 1 << 4;

const RX_BUF_SIZE: usize = 8192 + 16;
const TX_BUF_SIZE: usize = 1536;

pub struct Rtl8139Driver {
    io_base: u16,
    rx_buf: DmaBuffer<u8>,
    tx_bufs: [DmaBuffer<u8>; 4],
    rx_offset: u16,
    mac: [u8; 6],
}

impl Rtl8139Driver {
    pub fn new(pci_dev: &PciDevice) -> Result<Self, NetError> {
        let io_base = match pci_dev.bar(0) {
            Some(BarInfo::Io { base, .. }) => base,
            _ => return Err(NetError::InvalidBar),
        };
        
        unsafe { outb(io_base + CONFIG1, 0x00) }
        
        unsafe { outb(io_base + CMD, CMD_RST) }
        
        let mut timeout = 100_000;
        while timeout > 0 {
            let cmd = unsafe { inb(io_base + CMD) };
            if cmd & CMD_RST == 0 { break; }
            timeout -= 1;
        }
        
        if timeout == 0 {
            return Err(NetError::ResetTimeout);
        }
        
        let mut mac = [0u8; 6];
        for i in 0..6 {
            mac[i] = unsafe { inb(io_base + MAC0_5 + i) };
        }
        
        let rx_buf = DmaBuffer::alloc(RX_BUF_SIZE, 32)?;
        let tx_bufs = [
            DmaBuffer::alloc(TX_BUF_SIZE, 32)?,
            DmaBuffer::alloc(TX_BUF_SIZE, 32)?,
            DmaBuffer::alloc(TX_BUF_SIZE, 32)?,
            DmaBuffer::alloc(TX_BUF_SIZE, 32)?,
        ];
        
        unsafe { outl(io_base + RX_BUF, rx_buf.phys() as u32) }
        unsafe { outw(io_base + IMR, INT_ROK | INT_TOK | INT_RER | INT_TER | INT_RXOVW) }
        
        let rx_config = RX_CONFIG_AAP | RX_CONFIG_APM | RX_CONFIG_AM | RX_CONFIG_AB
                      | RX_CONFIG_WRAP | RX_CONFIG_MXDMA_UNLIMITED | RX_CONFIG_RBLN_8K;
        unsafe { outl(io_base + RX_CONFIG, rx_config) }
        
        let tx_config = TX_CONFIG_MXDMA_2048 | TX_CONFIG_IFG_NORMAL;
        unsafe { outl(io_base + TX_CONFIG, tx_config) }
        
        unsafe { outb(io_base + CMD, CMD_RE | CMD_TE) }
        
        Ok(Self {
            io_base,
            rx_buf,
            tx_bufs,
            rx_offset: 0,
            mac,
        })
    }
    
    pub fn mac(&self) -> &[u8; 6] {
        &self.mac
    }
    
    pub fn transmit(&self, packet: &[u8], slot: usize) -> Result<(), NetError> {
        if packet.len() > TX_BUF_SIZE {
            return Err(NetError::PacketTooLarge);
        }
        if slot > 3 {
            return Err(NetError::InvalidTxSlot);
        }
        
        unsafe {
            core::ptr::copy_nonoverlapping(
                packet.as_ptr(),
                self.tx_bufs[slot].virt(),
                packet.len()
            );
        }
        
        let tx_status = (packet.len() & 0x1FFF) as u32;
        unsafe { outl(self.io_base + TX_STATUS0 + (slot as u16) * 4, tx_status) }
        
        unsafe { outl(self.io_base + TX_ADDR0 + (slot as u16) * 4, 
                     self.tx_bufs[slot].phys() as u32) }
        
        Ok(())
    }
    
    pub fn receive(&mut self, buf: &mut [u8]) -> Option<usize> {
        let cmd = unsafe { inb(self.io_base + CMD) };
        if cmd & CMD_BUFE != 0 {
            return None;
        }
        
        let rx_ptr = self.rx_buf.virt();
        let pkt_header = unsafe { 
            core::ptr::read_volatile((rx_ptr + self.rx_offset as usize) as *const u16)
        };
        let pkt_len = unsafe {
            core::ptr::read_volatile((rx_ptr + self.rx_offset as usize + 2) as *const u16)
        };
        
        if pkt_header & 1 == 0 {
            return None;
        }
        
        let data_len = (pkt_len as usize) - 4;
        
        if buf.len() < data_len {
            return None;
        }
        
        let src = rx_ptr + self.rx_offset as usize + 4;
        unsafe {
            core::ptr::copy_nonoverlapping(src as *const u8, buf.as_mut_ptr(), data_len);
        }
        
        self.rx_offset = (self.rx_offset + 4 + pkt_len + 3) & !3;
        if self.rx_offset as usize >= RX_BUF_SIZE {
            self.rx_offset -= RX_BUF_SIZE as u16;
        }
        
        unsafe { outw(self.io_base + RX_PTR, self.rx_offset - 0x10) }
        
        Some(data_len)
    }
    
    pub fn handle_interrupt(&self) {
        let isr = unsafe { inw(self.io_base + ISR) };
        
        if isr & INT_ROK != 0 {}
        if isr & INT_TOK != 0 {}
        if isr & INT_RXOVW != 0 {}
        
        unsafe { outw(self.io_base + ISR, isr) }
    }
}
```

### 7.2 TCP/IP Yığını (`kernel/tcpip_stack.rs`)

```rust
// kernel/tcpip_stack.rs
#![no_std]

/// Ethernet II Frame
#[repr(C)]
pub struct EthernetFrame {
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    ethertype: u16,
    payload: [u8],
}

/// ARP Packet
#[repr(C)]
pub struct ArpPacket {
    hw_type: u16,
    proto_type: u16,
    hw_len: u8,
    proto_len: u8,
    opcode: u16,
    sender_mac: [u8; 6],
    sender_ip: [u8; 4],
    target_mac: [u8; 6],
    target_ip: [u8; 4],
}

/// IPv4 Header
#[repr(C)]
pub struct Ipv4Header {
    version_ihl: u8,
    dscp_ecn: u8,
    total_length: u16,
    identification: u16,
    flags_fragment: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
}

/// TCP Header
#[repr(C)]
pub struct TcpHeader {
    src_port: u16,
    dst_port: u16,
    seq_num: u32,
    ack_num: u32,
    data_offset: u8,
    flags: u8,
    window: u16,
    checksum: u16,
    urgent_ptr: u16,
}

/// TCP Connection States
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

/// TCP Control Block (per connection)
pub struct TcpControlBlock {
    state: TcpState,
    local_addr: SocketAddrV4,
    remote_addr: SocketAddrV4,
    
    snd_una: u32,
    snd_nxt: u32,
    snd_wnd: u16,
    snd_wl1: u32,
    snd_wl2: u32,
    iss: u32,
    
    rcv_nxt: u32,
    rcv_wnd: u16,
    irs: u32,
    
    rto: u32,
    srtt: u32,
    rttvar: u32,
    
    tx_buffer: RingBuffer<8192>,
    rx_buffer: RingBuffer<8192>,
    
    cwnd: u16,
    ssthresh: u16,
}

impl TcpControlBlock {
    pub fn on_packet(&mut self, tcp: &TcpHeader, payload: &[u8]) -> Option<Vec<u8>> {
        match self.state {
            TcpState::Listen => {
                if tcp.flags & 0x02 != 0 {
                    self.irs = tcp.seq_num;
                    self.rcv_nxt = tcp.seq_num.wrapping_add(1);
                    self.iss = self.generate_iss();
                    self.snd_nxt = self.iss.wrapping_add(1);
                    self.snd_una = self.iss;
                    self.state = TcpState::SynReceived;
                    Some(self.build_syn_ack())
                } else {
                    None
                }
            }
            TcpState::SynSent => {
                if tcp.flags & 0x12 == 0x12 {
                    self.snd_una = tcp.ack_num;
                    self.irs = tcp.seq_num;
                    self.rcv_nxt = tcp.seq_num.wrapping_add(1);
                    self.state = TcpState::Established;
                    Some(self.build_ack())
                } else {
                    None
                }
            }
            TcpState::SynReceived => {
                if tcp.flags & 0x10 != 0 {
                    if tcp.ack_num == self.snd_nxt {
                        self.state = TcpState::Established;
                    }
                }
                None
            }
            TcpState::Established => {
                let seg_len = payload.len() as u32;
                if !self.is_acceptable(tcp.seq_num, seg_len) {
                    if tcp.flags & 0x04 == 0 {
                        return Some(self.build_ack());
                    }
                    return None;
                }
                
                if tcp.flags & 0x10 != 0 {
                    self.process_ack(tcp.ack_num);
                }
                
                if !payload.is_empty() {
                    let seq = tcp.seq_num;
                    let expected = self.rcv_nxt;
                    
                    if seq == expected {
                        self.rx_buffer.write(payload);
                        self.rcv_nxt = self.rcv_nxt.wrapping_add(seg_len);
                        return Some(self.build_ack_with_data(&[]));
                    } else if self.is_within_window(seq) {
                        self.rx_buffer.write_out_of_order(seq, payload);
                        return Some(self.build_ack());
                    }
                }
                
                if tcp.flags & 0x01 != 0 {
                    self.rcv_nxt = self.rcv_nxt.wrapping_add(1);
                    self.state = TcpState::CloseWait;
                    return Some(self.build_ack());
                }
                
                None
            }
            TcpState::FinWait1 => {
                if tcp.flags & 0x10 != 0 && tcp.ack_num == self.snd_nxt {
                    self.state = TcpState::FinWait2;
                }
                if tcp.flags & 0x01 != 0 {
                    self.rcv_nxt = self.rcv_nxt.wrapping_add(1);
                    self.state = TcpState::TimeWait;
                    return Some(self.build_ack());
                }
                None
            }
            _ => None,
        }
    }
    
    fn build_syn_ack(&self) -> Vec<u8> {
        let mut pkt = Vec::with_capacity(40);
        
        pkt.push(0x45);
        pkt.push(0x00);
        pkt.extend_from_slice(&u16::to_be_bytes(40));
        pkt.extend_from_slice(&0u16.to_be_bytes());
        pkt.extend_from_slice(&0x4000u16.to_be_bytes());
        pkt.push(64);
        pkt.push(6);
        pkt.extend_from_slice(&0u16.to_be_bytes());
        pkt.extend_from_slice(&self.local_addr.ip().octets());
        pkt.extend_from_slice(&self.remote_addr.ip().octets());
        
        pkt.extend_from_slice(&self.local_addr.port().to_be_bytes());
        pkt.extend_from_slice(&self.remote_addr.port().to_be_bytes());
        pkt.extend_from_slice(&self.iss.to_be_bytes());
        pkt.extend_from_slice(&self.rcv_nxt.to_be_bytes());
        pkt.push(0x50);
        pkt.push(0x12);
        pkt.extend_from_slice(&self.rcv_wnd.to_be_bytes());
        pkt.extend_from_slice(&0u16.to_be_bytes());
        pkt.extend_from_slice(&0u16.to_be_bytes());
        
        self.fill_checksums(&mut pkt);
        
        pkt
    }
    
    fn fill_checksums(&self, pkt: &mut [u8]) {
        let ip_checksum = Self::checksum(&pkt[0..20]);
        pkt[10..12].copy_from_slice(&ip_checksum.to_be_bytes());
        
        let pseudo_header = [
            &self.local_addr.ip().octets()[..],
            &self.remote_addr.ip().octets()[..],
            &[0u8, 6],
            &((pkt.len() - 20) as u16).to_be_bytes()[..],
        ].concat();
        
        let tcp_checksum = Self::checksum(&[&pseudo_header[..], &pkt[20..]].concat());
        pkt[36..38].copy_from_slice(&tcp_checksum.to_be_bytes());
    }
    
    fn checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;
        while i + 1 < data.len() {
            sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
            i += 2;
        }
        if i < data.len() {
            sum += (data[i] as u32) << 8;
        }
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        !(sum as u16)
    }
}
```

---

## 8. GPU SÜRÜCÜLERİ VE KRİTİK HATALARIN DÜZELTİLMESİ

### 8.1 Kritik Hata #1: Sanal Adres → GPU MMIO

**MEVCUT HATALI KOD** (`ozkan_os_gpu.rs`):
```rust
// HATALI - sanal adres GPU'ya veriliyor
let buf = alloc::vec![0u8; 4096];
gpu_submit_command(cmd, buf.as_ptr() as u64); // ❌ Sanal adres!
```

**GERÇEK DÜZELTME** (`kernel/dma_buffer.rs`):
```rust
// kernel/dma_buffer.rs
#![no_std]

pub struct DmaBuffer {
    phys: u64,
    virt: *mut u8,
    size: usize,
}

impl DmaBuffer {
    pub fn alloc(size: usize, align: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, align.max(4096)).ok()?;
        let virt = unsafe { alloc::alloc::alloc(layout) };
        if virt.is_null() { return None; }
        
        let phys = virt_to_phys(virt as u64)?;
        
        Some(Self { phys, virt, size })
    }
    
    pub fn phys(&self) -> u64 { self.phys }
    pub fn virt(&self) -> *mut u8 { self.virt }
    
    pub fn flush(&self) {
        unsafe {
            core::arch::asm!("clflush [{}]", in(reg) self.virt);
        }
    }
}

/// x86_64 page table walk (recursive mapping)
pub fn virt_to_phys(virt: u64) -> Option<u64> {
    const RECURSIVE_PML4: u64 = 0xFFFF_FFFF_FFFF_F000;
    
    let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
    let pd_idx = ((virt >> 21) & 0x1FF) as usize;
    let pt_idx = ((virt >> 12) & 0x1FF) as usize;
    
    let pml4_entry = unsafe { core::ptr::read_volatile((RECURSIVE_PML4 + pml4_idx * 8) as *const u64) };
    if pml4_entry & 1 == 0 { return None; }
    
    let pdpt_base = (pml4_entry & 0x000F_FFFF_FFFF_F000) | 0xFFFF_0000_0000_0000;
    let pdpt_entry = unsafe { core::ptr::read_volatile((pdpt_base + pdpt_idx * 8) as *const u64) };
    if pdpt_entry & 1 == 0 { return None; }
    
    if pdpt_entry & (1 << 7) != 0 {
        return Some((pdpt_entry & 0x000F_FFFF_C000_0000) | (virt & 0x3FFF_FFFF));
    }
    
    let pd_base = (pdpt_entry & 0x000F_FFFF_FFFF_F000) | 0xFFFF_0000_0000_0000;
    let pd_entry = unsafe { core::ptr::read_volatile((pd_base + pd_idx * 8) as *const u64) };
    if pd_entry & 1 == 0 { return None; }
    
    if pd_entry & (1 << 7) != 0 {
        return Some((pd_entry & 0x000F_FFFF_FFE0_0000) | (virt & 0x1F_FFFF));
    }
    
    let pt_base = (pd_entry & 0x000F_FFFF_FFFF_F000) | 0xFFFF_0000_0000_0000;
    let pt_entry = unsafe { core::ptr::read_volatile((pt_base + pt_idx * 8) as *const u64) };
    if pt_entry & 1 == 0 { return None; }
    
    Some((pt_entry & 0x000F_FFFF_FFFF_F000) | (virt & 0xFFF))
}
```

### 8.2 Intel i915 Modeset (`drivers/gpu/i915_gpu.rs`)

```rust
// drivers/gpu/i915_gpu.rs
#![no_std]

const INTEL_VENDOR: u16 = 0x8086;

/// Gen detection
const GEN2_IDS: [u16; 4] = [0x3577, 0x2562, 0x2572, 0x3582];
const GEN3_IDS: [u16; 7] = [0x2582, 0x258A, 0x2592, 0x2772, 0x27A2, 0x27AE, 0x2982];
const GEN4_IDS: [u16; 14] = [0x2972, 0x2982, 0x2992, 0x29A2, 0x29B2, 0x29C2, 0x29D2,
                             0x2A02, 0x2A12, 0x2A42, 0x2E02, 0x2E12, 0x2E22, 0x2E32];
const GEN5_RANGE: (u16, u16) = (0x0040, 0x004F);
const GEN6_RANGE: (u16, u16) = (0x0100, 0x012F);
const GEN7_RANGE: (u16, u16) = (0x0150, 0x016F);
const GEN8_RANGE: (u16, u16) = (0x0A00, 0x0A3F);
const GEN9_RANGE: (u16, u16) = (0x1900, 0x193F);
const GEN11_RANGE: (u16, u16) = (0x8A50, 0x8A5F);
const GEN12_RANGE1: (u16, u16) = (0x9A40, 0x9A7F);
const GEN12_RANGE2: (u16, u16) = (0x4680, 0x46FF);

/// Display register offsets (Gen 9+)
const TRANS_HTOTAL_A: u32 = 0x6E000;
const TRANS_HBLANK_A: u32 = 0x6E004;
const TRANS_HSYNC_A: u32 = 0x6E008;
const TRANS_VTOTAL_A: u32 = 0x6E00C;
const TRANS_VBLANK_A: u32 = 0x6E010;
const TRANS_VSYNC_A: u32 = 0x6E014;
const PIPEASRC: u32 = 0x6001C;
const PIPEACONF: u32 = 0x70008;

const PIPECONF_ENABLE: u32 = 1 << 31;

/// Legacy VGA CRTC registers
const VGA_CR_INDEX: u16 = 0x3D4;
const VGA_CR_DATA: u16 = 0x3D5;

const HORIZONTAL_TOTAL: u8 = 0x00;
const HORIZONTAL_DISPLAY_END: u8 = 0x01;
const HORIZONTAL_BLANK_START: u8 = 0x02;
const HORIZONTAL_BLANK_END: u8 = 0x03;
const HORIZONTAL_SYNC_START: u8 = 0x04;
const HORIZONTAL_SYNC_END: u8 = 0x05;
const VERTICAL_TOTAL: u8 = 0x06;
const OVERFLOW: u8 = 0x07;
const VERTICAL_RETRACE_START: u8 = 0x10;
const VERTICAL_RETRACE_END: u8 = 0x11;
const VERTICAL_DISPLAY_END: u8 = 0x12;
const LOGICAL_SCREEN_OFFSET: u8 = 0x13;
const VERTICAL_BLANK_START: u8 = 0x15;
const VERTICAL_BLANK_END: u8 = 0x16;
const MODE_CONTROL: u8 = 0x17;

pub struct I915GpuDriver {
    mmio_base: u64,
    aperture_base: u64,
    gen: u8,
    pipe: DisplayPipe,
}

#[derive(Clone, Copy)]
pub enum DisplayPipe {
    A, B, C, D,
}

impl I915GpuDriver {
    pub fn new(pci_dev: &PciDevice) -> Result<Self, GpuError> {
        let mmio_base = match pci_dev.bar(0) {
            Some(BarInfo::Memory64 { base, .. }) => base,
            Some(BarInfo::Memory32 { base, .. }) => base,
            _ => return Err(GpuError::InvalidBar),
        };
        
        let aperture_base = match pci_dev.bar(2) {
            Some(BarInfo::Memory64 { base, .. }) => base,
            Some(BarInfo::Memory32 { base, .. }) => base,
            _ => return Err(GpuError::InvalidBar),
        };
        
        let dev_id = pci_dev.device_id();
        let gen = Self::detect_gen(dev_id)?;
        
        Ok(Self {
            mmio_base,
            aperture_base,
            gen,
            pipe: DisplayPipe::A,
        })
    }
    
    fn detect_gen(dev_id: u16) -> Result<u8, GpuError> {
        if GEN2_IDS.contains(&dev_id) { return Ok(2); }
        if GEN3_IDS.contains(&dev_id) { return Ok(3); }
        if GEN4_IDS.contains(&dev_id) { return Ok(4); }
        if (GEN5_RANGE.0..=GEN5_RANGE.1).contains(&dev_id) { return Ok(5); }
        if (GEN6_RANGE.0..=GEN6_RANGE.1).contains(&dev_id) { return Ok(6); }
        if (GEN7_RANGE.0..=GEN7_RANGE.1).contains(&dev_id) { return Ok(7); }
        if (GEN8_RANGE.0..=GEN8_RANGE.1).contains(&dev_id) { return Ok(8); }
        if (GEN9_RANGE.0..=GEN9_RANGE.1).contains(&dev_id) { return Ok(9); }
        if (GEN11_RANGE.0..=GEN11_RANGE.1).contains(&dev_id) { return Ok(11); }
        if (GEN12_RANGE1.0..=GEN12_RANGE1.1).contains(&dev_id) { return Ok(12); }
        if (GEN12_RANGE2.0..=GEN12_RANGE2.1).contains(&dev_id) { return Ok(12); }
        Err(GpuError::UnsupportedDevice)
    }
    
    pub fn set_mode(&self, width: u16, height: u16, bpp: u8) -> Result<(), GpuError> {
        if self.gen <= 4 {
            self.set_mode_vga(width, height, bpp)
        } else if self.gen <= 7 {
            self.set_mode_pch(width, height, bpp)
        } else {
            self.set_mode_gen9(width, height, bpp)
        }
    }
    
    fn set_mode_vga(&self, width: u16, height: u16, bpp: u8) -> Result<(), GpuError> {
        let h_total = 1344;
        let h_display = 1024;
        let h_sync_start = 1048;
        let h_sync_end = 1184;
        
        let v_total = 806;
        let v_display = 768;
        let v_sync_start = 771;
        let v_sync_end = 777;
        
        unsafe {
            x86_io_access::outb(VGA_CR_INDEX, 0x01);
            let sr1 = x86_io_access::inb(VGA_CR_DATA);
            x86_io_access::outb(VGA_CR_DATA, sr1 & !0x20);
        }
        
        unsafe {
            let write_crtc = |reg: u8, val: u8| {
                x86_io_access::outb(VGA_CR_INDEX, reg);
                x86_io_access::outb(VGA_CR_DATA, val);
            };
            
            write_crtc(HORIZONTAL_TOTAL, (h_total / 8 - 5) as u8);
            write_crtc(HORIZONTAL_DISPLAY_END, (h_display / 8 - 1) as u8);
            write_crtc(HORIZONTAL_BLANK_START, (h_blank_start / 8 - 1) as u8);
            write_crtc(HORIZONTAL_BLANK_END, ((h_blank_end / 8 - 1) & 0x1F) as u8 | 0x80);
            write_crtc(HORIZONTAL_SYNC_START, (h_sync_start / 8) as u8);
            write_crtc(HORIZONTAL_SYNC_END, (h_sync_end / 8) as u8 & 0x1F);
            
            write_crtc(VERTICAL_TOTAL, (v_total & 0xFF) as u8);
            write_crtc(OVERFLOW, (
                ((v_total >> 8) & 1) as u8
                | (((v_display >> 7) & 1) << 1) as u8
                | (((v_sync_start >> 7) & 1) << 2) as u8
                | (((v_blank_start >> 7) & 1) << 3) as u8
                | ((v_total >> 9) & 1) as u8 << 4
                | ((v_display >> 9) & 1) as u8 << 5
                | ((v_sync_start >> 9) & 1) as u8 << 6
                | ((v_blank_start >> 9) & 1) as u8 << 7
            ));
            write_crtc(VERTICAL_RETRACE_START, (v_sync_start & 0xFF) as u8);
            write_crtc(VERTICAL_RETRACE_END, (v_sync_end & 0x0F) as u8 | 0x20);
            write_crtc(VERTICAL_DISPLAY_END, (v_display & 0xFF) as u8);
            write_crtc(VERTICAL_BLANK_START, (v_blank_start & 0xFF) as u8);
            write_crtc(VERTICAL_BLANK_END, (v_blank_end & 0x7F) as u8);
            
            let pitch = (width as usize) * ((bpp / 8) as usize);
            write_crtc(LOGICAL_SCREEN_OFFSET, (pitch / 8) as u8);
            write_crtc(MODE_CONTROL, 0xC3);
            
            x86_io_access::outb(VGA_CR_INDEX, 0x01);
            let sr1 = x86_io_access::inb(VGA_CR_DATA);
            x86_io_access::outb(VGA_CR_DATA, sr1 | 0x20);
        }
        
        let fb = self.aperture_base as *mut u32;
        unsafe {
            for i in 0..((width as usize) * (height as usize)) {
                core::ptr::write_volatile(fb.add(i), 0x000000);
            }
        }
        
        Ok(())
    }
    
    fn set_mode_pch(&self, width: u16, height: u16, _bpp: u8) -> Result<(), GpuError> {
        let pipe_offset = match self.pipe {
            DisplayPipe::A => 0x60000,
            DisplayPipe::B => 0x61000,
            _ => return Err(GpuError::InvalidPipe),
        };
        
        unsafe {
            let pipe_conf = read32(self.mmio_base + pipe_offset + 0x008);
            write32(self.mmio_base + pipe_offset + 0x008, pipe_conf & !PIPECONF_ENABLE);
            
            let mut timeout = 10000;
            while timeout > 0 {
                let status = read32(self.mmio_base + pipe_offset + 0x008);
                if status & (1 << 30) == 0 { break; }
                timeout -= 1;
            }
            
            write32(self.mmio_base + pipe_offset + 0x000, 
                   ((1344 - 1) << 16) | (1024 - 1));
            write32(self.mmio_base + pipe_offset + 0x004,
                   ((1048 - 1) << 16) | (1184 - 1));
            write32(self.mmio_base + pipe_offset + 0x00C,
                   ((806 - 1) << 16) | (768 - 1));
            write32(self.mmio_base + pipe_offset + 0x010,
                   ((771 - 1) << 16) | (777 - 1));
            write32(self.mmio_base + pipe_offset + 0x01C,
                   ((height as u32 - 1) << 16) | (width as u32 - 1));
            write32(self.mmio_base + pipe_offset + 0x008, PIPECONF_ENABLE);
        }
        
        Ok(())
    }
    
    fn set_mode_gen9(&self, width: u16, height: u16, _bpp: u8) -> Result<(), GpuError> {
        let trans = match self.pipe {
            DisplayPipe::A => 0x6E000,
            DisplayPipe::B => 0x6E400,
            DisplayPipe::C => 0x6E800,
            _ => return Err(GpuError::InvalidPipe),
        };
        
        unsafe {
            let conf = read32(self.mmio_base + trans + 0x008);
            write32(self.mmio_base + trans + 0x008, conf & !(1 << 31));
            
            write32(self.mmio_base + trans + 0x000, ((1344 - 1) << 16) | (1024 - 1));
            write32(self.mmio_base + trans + 0x004, 0);
            write32(self.mmio_base + trans + 0x008, ((1048 - 1) << 16) | (1184 - 1));
            write32(self.mmio_base + trans + 0x00C, ((806 - 1) << 16) | (768 - 1));
            write32(self.mmio_base + trans + 0x010, ((771 - 1) << 16) | (777 - 1));
            write32(self.mmio_base + trans + 0x014, 0);
            write32(self.mmio_base + trans + 0x018, ((768 - 1) << 16) | (1024 - 1));
            
            write32(self.mmio_base + trans + 0x008, conf | (1 << 31));
        }
        
        Ok(())
    }
}
```

---

## 9. MİMARİYE ÖZGÜ BOOT DETAYLARI

### 9.1 486DX4 / i686 Boot (Gerçek Donanım)

```asm
; BOOT/x86_32_bootloader.asm
; 486DX4: 32-bit protected mode, NO PAE, NO long mode
; A20 line zorunlu
; 1-4MB RAM tipik, 16MB maksimum

bits 16
org 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    
    ; A20 line enable (fast A20 via 0x92)
    in al, 0x92
    or al, 2
    out 0x92, al
    
    ; GDT setup (flat 32-bit segments)
    lgdt [gdtr]
    
    ; Protected mode enable
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    
    jmp 0x08:.pmode

bits 32
.pmode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000
    
    ; 486DX4: 2-level page table (no PAE)
    ; CR4.PAE must be 0
    ; Page directory: 1024 entries, each 4MB
    ; Page table: 1024 entries, each 4KB
    
    ; Identity map first 4MB (kernel code + data)
    ; Higher half map at 0xC0000000
    
    ; Enable paging
    mov eax, page_directory
    mov cr3, eax
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax
    
    ; Jump to higher half
    jmp 0xC0000000 + kernel_entry

gdtr:
    dw gdt_end - gdt - 1
    dd gdt

gdt:
    dq 0x0000000000000000  ; Null
    dq 0x00CF9A000000FFFF  ; Code: base=0, limit=4GB, 32-bit, ring 0
    dq 0x00CF92000000FFFF  ; Data: base=0, limit=4GB, ring 0
gdt_end:

align 4096
page_directory:
    times 1024 dd 0
```

### 9.2 ARM64 (aarch64) Boot (QEMU virt / Raspberry Pi 4)

```asm
; BOOT/aarch64_bootloader.S
; Entry: EL3 (Secure) veya EL2 (Hypervisor)
; Hedef: EL1 (OS) with MMU enabled

.section .text.boot
.global _start

_start:
    // Her CPU core için ayrı init
    mrs x0, mpidr_el1
    and x0, x0, #0xFF
    cbz x0, primary_core
    
secondary_core:
    wfe
    b secondary_core
    
primary_core:
    // Stack setup (her exception level için ayrı)
    ldr x0, =_stack_top_el1
    msr sp_el1, x0
    
    // EL2'den EL1'e düş (eğer EL2'deysek)
    mrs x0, currentel
    cmp x0, #(2 << 2)  // EL2
    b.eq drop_to_el1
    cmp x0, #(3 << 2)  // EL3
    b.eq drop_to_el3
    b el1_setup
    
drop_to_el3:
    // SCR_EL3: RW=1 (AArch64 at EL2), NS=1 (Non-secure)
    mov x0, #(1 << 10) | (1 << 0)
    msr scr_el3, x0
    
    // SPSR_EL3: M[3:0] = 0b1001 (EL2h), interrupts masked
    mov x0, #(0b1111 << 6) | 0b1001
    msr spsr_el3, x0
    
    adr x0, drop_to_el2
    msr elr_el3, x0
    eret
    
drop_to_el2:
    // HCR_EL2: RW=1 (AArch64 at EL1), SWIO=1
    mov x0, #(1 << 31) | (1 << 1)
    msr hcr_el2, x0
    
    // SPSR_EL2: M[3:0] = 0b0101 (EL1h)
    mov x0, #(0b1111 << 6) | 0b0101
    msr spsr_el2, x0
    
    adr x0, el1_setup
    msr elr_el2, x0
    eret
    
el1_setup:
    // MMU off
    mrs x0, sctlr_el1
    bic x0, x0, #1
    msr sctlr_el1, x0
    isb
    
    // TLB invalidate
    tlbi vmalle1
    dsb sy
    isb
    
    // MAIR_EL1: memory attributes
    ldr x0, =0xFF440C0400FF0000  // Attr0=WB, Attr1=WT, Attr2=NC, Attr3=Device-nGnRnE, Attr4=Device-nGnRE
    msr mair_el1, x0
    
    // TCR_EL1: 4KB granule, 48-bit VA, inner/outer shareable WB
    ldr x0, =0x00000010B5183518
    msr tcr_el1, x0
    
    // TTBR0/TTBR1: page table base (fiziksel adres)
    ldr x0, =page_table_l0
    msr ttbr0_el1, x0
    msr ttbr1_el1, x0
    isb
    
    // Enable MMU, I-cache, D-cache
    mrs x0, sctlr_el1
    orr x0, x0, #(1 << 0)   // M (MMU)
    orr x0, x0, #(1 << 2)   // C (D-cache)
    orr x0, x0, #(1 << 12)  // I (I-cache)
    msr sctlr_el1, x0
    isb
    
    // Jump to C code
    ldr x0, =kernel_main
    br x0

.section .bss
.align 16
_stack_bottom_el1:
    .space 65536
_stack_top_el1:

.align 4096
page_table_l0:
    .space 4096
page_table_l1:
    .space 4096
page_table_l2:
    .space 4096
```

### 9.3 RISC-V64 Boot (QEMU virt / VisionFive 2)

```asm
; BOOT/riscv64_bootloader.S
; Entry: M-mode (hart 0)
; Hedef: S-mode with Sv48 MMU

.section .text.boot
.global _start

_start:
    // Sadece hart 0 init yapsın
    csrr a0, mhartid
    bnez a0, secondary_hart
    
    // Stack setup
    la sp, _stack_top
    
    // PMP (Physical Memory Protection): tüm belleğe R/W/X izni ver
    // pmpaddr0 = 0x3FFFFFFFFFFFFF (tüm 56-bit adres alanı, NAPOT)
    li t0, 0x3FFFFFFFFFFFFF
    csrw pmpaddr0, t0
    // pmpcfg0: L=1 (locked), A=3 (NAPOT), X=1, W=1, R=1 = 0x9F
    li t0, 0x000000000000009F
    csrw pmpcfg0, t0
    
    // SATP: Sv48 modu (MODE=9)
    la t0, page_table_root
    srli t0, t0, 12        // PPN shift
    li t1, (9 << 60)       // MODE = Sv48
    or t0, t0, t1
    csrw satp, t0
    
    // MSTATUS: MPP = 01 (S-mode), MPIE = 1
    li t0, (1 << 11) | (1 << 7) | (1 << 3)
    csrw mstatus, t0
    
    // MEDELEG/MIDELEG: exception/interrupt delegation to S-mode
    li t0, 0xFFFF
    csrw medeleg, t0
    csrw mideleg, t0
    
    // MEPC: return address (S-mode entry)
    la t0, s_mode_entry
    csrw mepc, t0
    
    // MIE: external, timer, software interrupts enabled
    li t0, (1 << 11) | (1 << 7) | (1 << 3)
    csrw mie, t0
    
    // MRET: M-mode -> S-mode
    mret
    
s_mode_entry:
    // S-mode C code entry
    la t0, kernel_main
    jr t0
    
secondary_hart:
    wfi
    j secondary_hart

.section .bss
.align 16
_stack_bottom:
    .space 65536
_stack_top:

.align 4096
page_table_root:
    .space 4096
```

---

## 10. GERÇEKLEŞTİRİLECEK İŞLEM SIRASI (Öncelik Sırası)

### Aşama 1: Acil Düzeltmeler (Hafta 1)
1. **`highres_renderer.rs`** `#![no_std]` derleme hatasını düzelt
2. **`ozkan_os_gpu.rs`** sanal adres → fiziksel adres çevrimini ekle
3. **`gpgpu.rs`** fake MMIO adresini kaldır, PCI probe kullan
4. **`nvidia.rs`** decoder mask `0x0F` → `0x50` olarak düzelt
5. **`main.rs`** FS wrapper'ları (1300 satır) ilgili crate'lere taşı

### Aşama 2: Çoklu Mimari İskelet (Hafta 2-3)
1. `kernel/architecture_hal.rs` — ortak trait'ler
2. `kernel/x86_64_hal.rs` — x86_64 implementasyonu (mevcut kod refactor)
3. `kernel/aarch64_hal.rs` — ARM64 iskelet (QEMU virt ile test)
4. `kernel/riscv64_hal.rs` — RISC-V64 iskelet (QEMU virt ile test)
5. `kernel/mips64_hal.rs` — MIPS64 iskelet
6. `BOOT/aarch64_bootloader.S` — ARM64 boot
7. `BOOT/riscv64_bootloader.S` — RISC-V64 boot

### Aşama 3: Temel Sürücüleri Tamamla (Hafta 4-6)
1. `drivers/storage/ahci_driver.rs` — read/write implementasyonu
2. `drivers/storage/nvme_driver.rs` — I/O queue oluşturma
3. `drivers/net/rtl8139_driver.rs` — RX/TX ring tamamla
4. `kernel/tcpip_stack.rs` — ARP + ICMP + UDP + TCP handshake
5. `drivers/audio/hda_audio.rs` — HDA codec init

### Aşama 4: GPU Düzeltmeleri (Hafta 7-8)
1. `drivers/gpu/i915_gpu.rs` — modeset tamamla, GEM init
2. `drivers/gpu/amdgpu_gpu.rs` — power management, display
3. `drivers/gpu/nouveau_gpu.rs` — NVxx decoder düzelt
4. `kernel/dma_buffer.rs` — universal DMA pool

### Aşama 5: Çoklu Mimari Boot (Hafta 9-12)
1. Her mimari için: boot → kernel init → HAL → timer → interrupts
2. QEMU test matrisi: `qemu-system-x86_64`, `qemu-system-aarch64`, `qemu-system-riscv64`
3. Gerçek donanım testi: Raspberry Pi 4, VisionFive 2

---

## 11. SONUÇ

| Kategori | Toplam | Gerçek Kod | Stub | Roadmap | Tamamlanma |
|----------|--------|-----------|------|---------|-----------|
| **Boot** | 13 mimari | 1 | 0 | 12 | %7.7 |
| **HAL** | 13 mimari | 1 | 0 | 12 | %7.7 |
| **Memory** | 13 mimari | 1 (x86_64) | 0 | 12 | %7.7 |
| **Interrupts** | 13 mimari | 1 | 0 | 12 | %7.7 |
| **Timer** | 13 mimari | 1 | 0 | 12 | %7.7 |
| **Storage** | 3 driver (AHCI/NVMe/SD) | 0.5 (AHCI probe) | 2.5 | 0 | %16.7 |
| **Network** | 4 driver + stack | 0.2 (RTL8139 probe) | 3.8 | 0 | %5.0 |
| **GPU** | 3 driver + core | 0.5 (modeset stub) | 2.5 | 0 | %16.7 |
| **Audio** | 1 driver (HDA) | 0.1 (probe) | 0.9 | 0 | %10.0 |
| **Input** | 2 driver (PS/2, USB) | 1.5 | 0.5 | 0 | %75.0 |
| **Display** | 1 (VBE) | 1 | 0 | 0 | %100 |
| **Sandbox** | 25 modül | 23.8 | 1.2 | 0 | %95.0 |
| **Compatibility** | 2 (DOS, Win32) | 1.6 | 0.4 | 0 | %80.0 |
| **Userland** | 4 (init, wm, desktop, apps) | 0 | 0 | 4 | %0 |
| **TOTAL** | - | - | - | - | **~35-40%** |

**Özet:** ÖZKAN-OS, x86_64 üzerinde boot-to-desktop çalışan bir çekirdeğe sahiptir. Ancak çoklu mimari desteğin tamamı (12/13) henüz başlamamıştır. Ağ yığını, NVMe, GPU sürücüleri ve userland zinciri kritik eksikliklerdir. Mevcut kodda 5 kritik donanım hatası gerçek PC'de çalışmayı engellemektedir. Öncelikli olarak bu hataların düzeltilmesi ve x86_64 dışındaki mimariler için boot iskeletinin oluşturulması gerekmektedir.
