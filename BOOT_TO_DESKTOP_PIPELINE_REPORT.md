# ÖZKAN-OS Boot → Masaüstü Pipeline Raporu (11 Mimari)

**Tarih**: 2026-05-15
**Kapsam**: x86_64, x86 (i686), AArch64, ARM32, RISC-V 64, RISC-V 32, LoongArch64, MIPS32, PowerPC 32, PowerPC 64
**Derinlik**: CPU init → RAM → MMU → GPU/VRAM → Storage (HDD/NVMe/SSD) → Input (K/F/D/K) → Masaüstü

---

## 1. Mimari Envanteri

| # | Mimari | Boot Dizini | Kernel Girişi | HAL Dizini |
|---|--------|-------------|---------------|------------|
| 1 | **x86_64** | `boot/x86_64/` | `boot/x86_64/entry.rs` | `kernel/hardware/hal/x86_64/` |
| 2 | **x86 (i686)** | `boot/x86/` | `boot/x86/entry.rs` | `kernel/hardware/hal/x86/` |
| 3 | **AArch64** | `boot/aarch64/` | `boot/aarch64/entry.rs` | `kernel/hardware/hal/aarch64/` |
| 4 | **ARM32** | `boot/arm32/` | `boot/arm32/entry.rs` | `kernel/hardware/hal/arm/` |
| 5 | **RISC-V 64** | `boot/riscv64/` | `boot/riscv64/entry.rs` | `kernel/hardware/hal/riscv/` |
| 6 | **RISC-V 32** | `boot/riscv32/` | `boot/riscv32/entry.rs` | `kernel/hardware/hal/riscv/` |
| 7 | **LoongArch64** | `boot/loongarch64/` | `boot/loongarch64/entry.rs` | `kernel/hardware/hal/loongarch64/` |
| 8 | **MIPS32** | `boot/mips32/` | `boot/mips32/entry.rs` | `kernel/hardware/hal/mips/` |
| 9 | **MIPS (ortak)** | `boot/mips/` | `boot/mips/entry.rs` | `kernel/hardware/hal/mips/` |
| 10 | **PowerPC 32** | `boot/ppc32/` | `boot/ppc32/entry.rs` | `kernel/hardware/hal/ppc32/` |
| 11 | **PowerPC 64** | `boot/ppc64/` | `boot/ppc64/entry.rs` | `kernel/hardware/hal/ppc64/` |

---

## 2. Çift Boot-Yol Tasarımı

ÖZKAN-OS'te **iki temel farklı boot yolu** vardır:

### 2.1 x86_64 "Zengin" Yol (Rich Path)

| Aşama | Dosya | Ana Eylem |
|-------|-------|-----------|
| Boot entry | `boot/x86_64/entry.rs` | CPU tespiti, E820, ACPI RSDP, entropi → `OzkanBootHeader` oluştur |
| Kernel dispatch | `kernel/system/core/kernel_entry.rs` | Header → `BootInfo` dönüşümü → `boot_sequence::kernel_main()` çağrısı |
| Tam init | `kernel/system/core/boot_sequence.rs` | HAL, güvenli önyükleme, bellek, KASLR, RNG, IRQ, SMP, scheduler, video, sürücüler, kullanıcı alanı |
| **Masaüstü** | — | `scheduler::idle_loop()`'da biter; masaüstü **kullanıcı alanı init**'inden beklenir |

### 2.2 Evrensel Yol (Tüm non-x86_64 + x86)

| Aşama | Dosya | Ana Eylem |
|-------|-------|-----------|
| Boot entry | `boot/<mimari>/entry.rs` | Framebuffer & bellek sabit kodlu → 104 baytlık `BootInfo` oluştur |
| Kernel dispatch | `kernel/system/core/oz_arch_kernel_entry.rs` | `kernel_main(info)` → `oz_arch_kernel_entry::kernel_main` |
| Temel init | Aynı dosya | Heap, framebuffer global, kernel state, input init, desktop loop |
| **Masaüstü** | Satır 261 | Eğer `bi.framebuffer_addr != 0` → `desktop::bare::run_desktop_loop()` |

---

## 3. Mimari Bazında Aşama Detayları

### 3.1 x86_64

#### Boot Entry (`boot/x86_64/entry.rs`)
- CPU özellikleri tespiti (SSE2/AVX/AVX-512)
- E820 bellek haritası okunur (`0x8000`)
- ACPI RSDP taranır
- Entropi toplanır
- `OzkanBootHeader` oluşturulur
- `kernel_main_ozkan()` çağrılır

#### CPU Init
- **HAL**: `kernel/hardware/hal/x86_64/hal_x86_64.rs`
- GDT/IDT init, APIC/IO-APIC init, PIC/PIT fallback
- LAPIC timer 100 Hz
- SMP: ACPI MADT parse + AP trampoline

#### RAM / MMU
- `kernel/system/core/boot_sequence.rs` içinde `memory::init()`
- Page allocator, slab allocator, heap allocator
- KASLR desteği
- KPTI (Kernel Page Table Isolation)

#### GPU / Framebuffer / VRAM
- `kernel/hardware/hal/x86_64/gpu.rs`
- Çalışma zamanı tespiti (öncelik sırası):
  1. GOP scratch (`0x7F00`)
  2. Bochs BGA (`0xFD00_0000`)
  3. VirtIO-GPU
  4. Intel i915
  5. AMD GCN
  6. NVIDIA nouveau
- VRAM yönetimi: DRM alt sistemi (modüler, `ozkan-os-drm-*` crate'leri)

#### Storage (HDD / NVMe / SSD)
- PCI tarama: `kernel/hardware/drivers/pci/pci.rs`
- NVMe: `kernel/hardware/hal/storage_nvme.rs` (BAR0 runtime okunur)
- AHCI: SATA controller desteği
- ATA/IDE: Legacy destek
- Block device abstraction → VFS registry

#### Input
- **PS/2**: IRQ1 (klavye), IRQ12 (fare) — `driver_init.rs`
- **USB HID**: xHCI PCI keşfi + timer fallback 1000 Hz — `hid_pump.rs`
- **I²C-HID**: DesignWare + Synaptics/ELAN touchpad — `i2c_driver.rs` (timer poll)
- **VMware vmmouse**: Absolute positioning PS/2 uzantısı — `mouse.rs`

#### Masaüstü Geçişi
- `boot_sequence.rs` masaüstüyü **doğrudan çağırmaz**
- Masaüstü `init_userspace()` tarafından başlatılması beklenir
- Eğer userspace init yoksa → `scheduler::idle_loop()`

---

### 3.2 x86 (i686 / 486DX4+)

#### Boot Entry (`boot/x86/entry.rs`)
- x86_64'e benzer ama 32-bit
- `OzkanBootHeader` oluşturulur
- `kernel_entry.rs` non-x86_64 yolu → `UniversalBootInfo`

#### CPU Init
- `kernel/hardware/hal/x86/hal_x86.rs`
- GDT/IDT (32-bit), PIC init
- APIC init (varsa)

#### RAM / MMU
- Identity paging varsayılır
- Heap seed: statik BSS 8 MB

#### GPU / Framebuffer
- **Sabit kodlu**: `framebuffer_addr: 0xFD00_0000` (BGA adresi)
- Runtime GPU keşfi **yok**
- Bu adres x86 QEMU'da BGA için geçerlidir

#### Storage
- ATA/IDE legacy destek
- NVMe BAR0 sabit kodlu (`0xF700_0000`)
- PCI tarama x86 için mevcut

#### Input
- PS/2 (i8042) destekli
- USB HID xHCI (varsa)

#### Masaüstü Geçişi
- `oz_arch_kernel_entry.rs` → `desktop::bare::run_desktop_loop()`
- Eğer framebuffer geçerliyse masaüstü başlar

---

### 3.3 AArch64

#### Boot Entry (`boot/aarch64/entry.rs`)
- PL011 UART @ `0x0900_0000`
- CPU özellikleri: `ID_AA64PFR0_EL1`
- Entropi
- Bellek keşfi: `discover_memory()` → `(0x4000_0000, 256MB, fdt=0x4000_0000)`
- **Sabit framebuffer**: `0xFD00_0000` (x86 BGA adresi — **GEÇERSİZ**)

#### CPU Init
- `kernel/hardware/hal/aarch64/hal_aarch64.rs`
- GIC-400 init (`gic::init()`)
- Generic Timer init
- `msr daifclr, #0xF` → IRQ enable

#### RAM / MMU
- `mmu::configure_tcr()` → TCR_EL1
- `mmu::enable()` → SCTLR.M biti
- **Ama**: Page allocator entegrasyonu eksik

#### GPU / Framebuffer
- `kernel/hardware/hal/aarch64/gpu.rs`
- DTB framebuffer @ `0x4000_0000`
- RPi mailbox → VirtIO-GPU → fallback `0x3C00_0000`
- **Ama**: Boot entry `0xFD00_0000` geçersiz

#### Storage
- NVMe BAR0 sabit kodlu: `0xFE36_0000`
- Runtime PCI enumeration **yok** (ECAM desteği var ama kullanılmıyor)

#### Input
- **VirtIO-Input**: ✅ Çalışır (FDT probe + MMIO)
- **PL050 KMI**: ✅ FDT tabanlı init + poll
- **I²C-HID**: ✅ `ozkan_os_i2c::i2c_hid_pump()` timer'dan çağrılır
- USB HID: Stub (dönüş `false`)

#### Kritik Bloklayıcılar
- RNG `init_global_rng()` takılma riski
- Framebuffer adresi geçersiz
- MMU init minimal

---

### 3.4 ARM32

#### Boot Entry (`boot/arm32/entry.rs`)
- AArch64'e benzer
- Sabit framebuffer: `0xFD00_0000`

#### CPU Init
- `kernel/hardware/hal/arm/hal_arm.rs`
- GIC stub
- MPIDR okuma

#### RAM / MMU
- MMU init stub
- Identity paging varsayılır

#### GPU / Framebuffer
- `kernel/hardware/hal/arm/gpu.rs`
- DTB framebuffer
- **Ama**: Boot entry sabit adres geçersiz

#### Storage
- NVMe BAR0 sabit kodlu
- PCI enum yok

#### Input
- VirtIO-Input ✅
- PL050 KMI ✅
- I²C-HID ✅

#### Kritik Bloklayıcılar
- Framebuffer adresi geçersiz
- MMU init stub

---

### 3.5 RISC-V 64

#### Boot Entry (`boot/riscv64/entry.rs`)
- NS16550 UART @ `0x1000_0000`
- MISA detection
- Entropi: `rdtime`
- Bellek: `(0x8000_0000, 256MB, fdt=0x8200_0000)`
- Sabit framebuffer: `0xFD00_0000`

#### CPU Init
- `kernel/hardware/hal/riscv/hal_riscv.rs`
- PLIC init (stub)
- `csrsi sstatus, 0x2` → IRQ enable
- `mhartid` okuma

#### RAM / MMU
- Identity mapping varsayılır
- Page allocator entegrasyonu yok

#### GPU / Framebuffer
- Boot entry sabit adres geçersiz
- VirtIO-GPU desteği kodda var ama init edilmiyor

#### Storage
- NVMe BAR0 sabit kodlu: `0x1003_0000`
- PCI enum yok

#### Input
- VirtIO-Input ✅ (FDT probe + MMIO)
- I²C-HID ✅ (timer poll)

#### Kritik Bloklayıcılar
- Framebuffer adresi geçersiz
- PLIC stub → IRQ-driven input yok
- MMU minimal

---

### 3.6 RISC-V 32

#### Boot Entry (`boot/riscv32/entry.rs`)
- RISC-V 64'ün aynısı ama 32-bit pointer
- Sabit framebuffer: `0xFD00_0000`

#### CPU Init
- RISC-V 64 ile aynı HAL kullanır
- 32-bit pointer farkı

#### RAM / MMU
- Identity mapping varsayılır

#### GPU / Framebuffer
- Boot entry sabit adres geçersiz

#### Storage
- NVMe BAR0 sabit kodlu

#### Input
- VirtIO-Input ✅
- I²C-HID ✅

#### Kritik Bloklayıcılar
- Framebuffer adresi geçersiz
- PLIC stub

---

### 3.7 LoongArch64

#### Boot Entry (`boot/loongarch64/entry.rs`)
- UART @ `0x1FE0_01E0`
- `cpucfg` detection
- Bellek: `(0x9000_0000_0000_0000, 256MB, fdt=0)`
- **Sabit framebuffer**: `0xFD00_0000`
- **FDT adresi 0** → donanım keşfi yapılamaz

#### CPU Init
- `kernel/hardware/hal/loongarch64/hal_loongarch64.rs`
- CSR init (boot.S'te yapılır)
- IOCSR IPI stub

#### RAM / MMU
- PGDL/PGDH = 0
- PGD + CRMD.PG
- Page allocator entegrasyonu yok

#### GPU / Framebuffer
- Boot entry sabit adres geçersiz
- FDT yok → VirtIO-GPU keşfi yapılamaz

#### Storage
- NVMe BAR0 sabit kodlu

#### Input
- VirtIO-Input: FDT yok → probe başarısız
- I²C-HID: timer poll var ama cihaz keşfi yok

#### Kritik Bloklayıcılar
- FDT adresi 0 → donanım keşfi yok
- Framebuffer adresi geçersiz
- MMU stub

---

### 3.8 MIPS32

#### Boot Entry (`boot/mips32/entry.rs`)
- UART @ `0xBF00_0900`
- CP0 config detection
- Bellek: `(0x0000_0000, 256MB, fdt=0)`
- Sabit framebuffer: `0xFD00_0000`

#### CPU Init
- `kernel/hardware/hal/mips/hal_mips.rs`
- CP0 status init
- CP0 EBase init
- AP stub

#### RAM / MMU
- TLB init
- `tlb::flush_all()`
- Page allocator entegrasyonu yok

#### GPU / Framebuffer
- Boot entry sabit adres geçersiz

#### Storage
- NVMe BAR0 sabit kodlu: `0xB070_0000`

#### Input
- VirtIO-Input: FDT yok → probe başarısız
- I²C-HID: timer poll var

#### Kritik Bloklayıcılar
- FDT yok → donanım keşfi yok
- Framebuffer adresi geçersiz
- `register_handler()` stub → IRQ kurulumu yok

---

### 3.9 PowerPC 32

#### Boot Entry (`boot/ppc32/entry.rs`)
- UART @ `0x8000_03F8`
- PVR-based CPU detection (G3/G4/e500)
- Sabit framebuffer: `0xFD00_0000`

#### CPU Init
- `kernel/hardware/hal/ppc32/hal_ppc32.rs`
- OpenPIC init stub
- SLB/DBAT init
- Spin-table stub

#### RAM / MMU
- SLB/DBAT clear
- MSR |= 0x30
- Page allocator entegrasyonu yok

#### GPU / Framebuffer
- Boot entry sabit adres geçersiz

#### Storage
- NVMe BAR0 sabit kodlu

#### Input
- VirtIO-Input: FDT yok → probe başarısız
- OHCI partial stub
- ADB stub

#### Kritik Bloklayıcılar
- Framebuffer adresi geçersiz
- IRQ handler registration stub
- MMU minimal

---

### 3.10 PowerPC 64

#### Boot Entry (`boot/ppc64/entry.rs`)
- UART @ `0x8000_03F8`
- PVR/PIC detection (POWER8/9/10/Cell)
- Sabit framebuffer: `0xFD00_0000`

#### CPU Init
- `kernel/hardware/hal/ppc64/hal_ppc64.rs`
- XICS/XIVE init stub
- Radix page table init stub
- RTAS stub
- AP stub

#### RAM / MMU
- `slbia`
- MSR |= 0x30
- Radix page table minimal

#### GPU / Framebuffer
- Boot entry sabit adres geçersiz

#### Storage
- NVMe BAR0 sabit kodlu

#### Input
- VirtIO-Input: FDT yok → probe başarısız
- OHCI partial stub

#### Kritik Bloklayıcılar
- Framebuffer adresi geçersiz
- IRQ handler registration stub
- MMU minimal

---

## 4. Masaüstü Döngüsü (Desktop Loop)

**Dosya**: `kernel/graphics/ui/gui/desktop/desktop_loop.rs`

### Çalışma Akışı
1. `ozkan_os_gpu::drm::manager()` sorgusu → GPU framebuffer override kontrolü
2. Back buffer alloc
3. Duvar kağıdı çizimi
4. Görev çubuğu, simgeler, pencereler çizimi
5. İmleç çizimi (mouse absolute pozisyon)
6. Input poll: `syscall_poll_input_into()`
7. Frame present: `present_frame_filtered()`
8. `syscall_yield()`

### x86_64'teki Problem
- `boot_sequence.rs` masaüstü döngüsünü **çağırmaz**
- Kullanıcı alanı init beklenir
- Bare-metal masaüstü ulaşılamaz

### Non-x86_64'teki Problem
- `oz_arch_kernel_entry.rs` masaüstüyü çağırır
- Ama framebuffer adresi geçersizse → `serial-idle loop`'a düşer

---

## 5. Desktop Erişilebilirlik Matrisi (Gerçek Durum)

| Mimari | Boot | Framebuffer | HAL | MMU | Input | Storage | Masaüstü Ulaşılabilir? | Birincil Bloklayıcılar |
|--------|------|-------------|-----|-----|-------|---------|----------------------|----------------------|
| **x86_64** | ✅ | ✅ Runtime PCI/GOP/BGA | Tam | Tam | ✅ PS/2 / USB HID / I²C-HID / VMware | ✅ PCI + AHCI/NVMe | **Kısmen** | Bare-metal masaüstü çağrısı yok |
| **x86 (i686)** | ✅ | ❌ Sabit `0xFD00_0000` | Kısmen | Identity | ✅ PS/2 (i8042) | ATA/IDE | **Evet** (QEMU BGA varsa) | Yanlış FB adresi |
| **AArch64** | ✅ | ❌ Sabit `0xFD00_0000` | GIC stub | TCR stub | ✅ VirtIO / PL050 / I²C-HID | NVMe tahmini | **Hayır** | Geçersiz FB, RNG takılma, MMU |
| **ARM32** | ✅ | ❌ Sabit `0xFD00_0000` | GIC stub | Stub | ✅ VirtIO / PL050 / I²C-HID | NVMe tahmini | **Hayır** | Geçersiz FB, MMU stub |
| **RISC-V 64** | ✅ | ❌ Sabit `0xFD00_0000` | PLIC stub | Identity | ✅ VirtIO / I²C-HID | NVMe tahmini | **Hayır** | Geçersiz FB, PLIC stub |
| **RISC-V 32** | ✅ | ❌ Sabit `0xFD00_0000` | PLIC stub | Identity | ✅ VirtIO / I²C-HID | NVMe tahmini | **Hayır** | Geçersiz FB, PLIC stub |
| **LoongArch64** | ✅ | ❌ Sabit `0xFD00_0000` | CSR stub | PGD stub | ⚠️ FDT yok | NVMe tahmini | **Hayır** | FDT=0, geçersiz FB |
| **MIPS32** | ✅ | ❌ Sabit `0xFD00_0000` | CP0 stub | TLB stub | ⚠️ FDT yok | NVMe tahmini | **Hayır** | FDT=0, geçersiz FB |
| **PowerPC 32** | ✅ | ❌ Sabit `0xFD00_0000` | OpenPIC stub | SLB stub | ⚠️ FDT yok | NVMe tahmini | **Hayır** | Geçersiz FB, IRQ stub |
| **PowerPC 64** | ✅ | ❌ Sabit `0xFD00_0000` | XICS stub | Radix stub | ⚠️ FDT yok | NVMe tahmini | **Hayır** | Geçersiz FB, IRQ stub |

---

## 6. Input Alt Sistemi Detayı (Güncel — 2026-05-15)

### 6.1 x86_64 Input Pipeline

```
PS/2 Keyboard (IRQ1) ──→ keyboard_handler ──→ keyboard::push_scancode ──→ input_driver::INPUT_QUEUE
PS/2 Mouse (IRQ12) ────→ mouse_handler ────→ ps2_mouse_irq ────────────→ input_driver::INPUT_QUEUE
USB HID Keyboard ──────→ xHCI Event Ring ──→ hid_pump (timer 1000Hz) ─→ input_driver::INPUT_QUEUE
USB HID Mouse ─────────→ xHCI Event Ring ──→ hid_pump (timer 1000Hz) ─→ input_driver::INPUT_QUEUE
USB HID Pen ───────────→ xHCI Event Ring ──→ hid_pump (timer 1000Hz) ─→ parse_usb_hid_pen / parse_wacom_pen
I²C-HID Touchpad ──────→ i2c_hid_pump ─────→ timer_handler (1000Hz) ──→ input_driver::INPUT_QUEUE
I²C-HID Pen ───────────→ i2c_hid_pump ─────→ timer_handler (1000Hz) ──→ input_driver::INPUT_QUEUE
VMware vmmouse ────────→ PS/2 absolute ─────→ mouse_handler ───────────→ input_driver::INPUT_QUEUE (absolute)
```

### 6.2 AArch64 / ARM32 Input Pipeline

```
FDT Probe ──→ virtio,mmio ──→ VirtIO-Input MMIO ──→ virtio_input_poll() ──→ input_driver::INPUT_QUEUE
FDT Probe ──→ arm,pl050 ────→ PL050 KMI ──────────→ poll_pl050_kbd() ─────→ input_driver::INPUT_QUEUE
I²C-HID ─────→ i2c_hid_pump() ─────────────────────→ oz_arch_drivers::poll_input()
```

### 6.3 RISC-V / LoongArch / MIPS / PPC Input Pipeline

```
FDT Probe ──→ virtio,mmio ──→ VirtIO-Input MMIO ──→ virtio_input_poll() ──→ input_driver::INPUT_QUEUE
I²C-HID ─────→ i2c_hid_pump() ─────────────────────→ oz_arch_drivers::poll_input()
```

> **Not**: LoongArch64, MIPS, PPC'de FDT adresi 0 olduğunda VirtIO-Input keşfi başarısız olur.

---

## 7. Kritik Boşluklar ve Bloklayıcılar

| # | Boşluk | Etki | Etkilenen Mimari(ler) |
|---|--------|------|----------------------|
| 1 | **Framebuffer sabit kodlu `0xFD00_0000`** (x86 BGA) | Tüm non-x86 mimarilerde geçersiz MMIO → çökme veya siyah ekran | Tüm non-x86_64 |
| 2 | **Bellek haritası sabit kodlu** (`discover_memory()` QEMU tahmini) | Yanlış RAM bölgeleri → allocator bozulması | Tüm non-x86_64 |
| 3 | **MMU init minimal/boş** (`mmu_init()` stub) | TLB/MMU istisnası geçersiz erişimlerde | Tüm non-x86_64 |
| 4 | **Input stub'ları** (USB HID `false` döndürür, IRQ handler yok) | Masaüstü çizilir ama tepkisiz | Tüm non-x86_64 (input kodu var ama init zinciri eksik) |
| 5 | **RNG takılma riski** (`init_global_rng()` aarch64'te) | Boot donmadan önce masaüstüne ulaşılamaz | AArch64 |
| 6 | **Storage sabit kodlu BAR0** | Blok cihazı yok → dosya yöneticisi başarısız | Tüm non-x86_64 |
| 7 | **x86_64 masaüstü çağrısı eksik** | Rich path bare-metal masaüstüne ulaşamaz | x86_64 |
| 8 | **FDT adresi 0** (LoongArch, MIPS, PPC) | Donanım keşfi tamamen yok | LoongArch64, MIPS32, PPC |
| 9 | **IRQ handler registration stub** | Runtime IRQ kaydı imkansız | MIPS, LoongArch, PPC |
| 10 | **SMP AP bring-up placeholder** | Sadece BSP çalışır; kritik değil ama eksik | Tüm non-x86_64 |

---

## 8. Önerilen Öncelikli Düzeltmeler (Masaüstüne Ulaşmak İçin)

| Öncelik | Düzeltme | Hedef Dosyalar | Etki |
|---------|----------|----------------|------|
| **P0** | **Framebuffer adresini mimari-özel keşifle değiştir** (VirtIO-GPU, DTB, veya headless fallback) | `boot/<mimari>/entry.rs` | Tüm non-x86_64'te görüntü |
| **P0** | **Gerçek bellek haritasını firmware'den parse et** (DTB / ACPI) | `boot/<mimari>/entry.rs` | Allocator stabilitesi |
| **P0** | **Identity paging veya geçerli root page table oluştur** | `kernel/hardware/hal/<mimari>/hal_impl.rs` | MMU hatalarını önle |
| **P1** | `init_global_rng()`'i aarch64'te güvenli hale getir veya atla | `kernel/system/core/boot_sequence_universal.rs` | Boot takılmasını önle |
| **P1** | **Input init zincirini tamamla** — `oz_arch_drivers::init_input_subsystem()`'i boot'tan çağır | `boot/<mimari>/entry.rs` | Tepkili masaüstü |
| **P1** | **x86_64 rich path'e `launch_desktop()` çağrısı ekle** (feature-gated) | `kernel/system/core/boot_sequence.rs` | x86_64 bare-metal masaüstü |
| **P2** | NVMe BAR0 sabit kodunu kaldır; MMIO/PCI probe ekle veya depolama init'ini graceful skip yap | `kernel/hardware/hal/storage_nvme.rs` | Dosya yöneticisi stabilitesi |
| **P2** | IRQ handler registration'ı tüm mimarilerde implemente et | `kernel/hardware/hal/<mimari>/hal_impl.rs` | IRQ-driven input & storage |

---

## 9. Özet

### Çalışan Durumda Olanlar
- **x86_64**: En gelişmiş yol. PS/2, USB HID, I²C-HID, VMware vmmouse hepsi çalışır. Ama masaüstü çağrısı eksik (userspace init bekleniyor).
- **Input altyapısı (kod)**: Tüm mimarilerde VirtIO-Input, PL050, I²C-HID sürücü kodları yazıldı ve derleniyor.

### Çalışmayanlar / Bloklayıcılar
- **Görüntü (Framebuffer)**: Non-x86_64'te sabit kodlu `0xFD00_0000` adresi neredeyse her zaman geçersiz.
- **Bellek**: Hardcoded RAM bölgeleri gerçek donanımda yanlış.
- **MMU**: Çoğu mimaride stub, identity paging veya geçerli page table yok.
- **Masaüstü erişimi**: Sadece x86 (i686) QEMU'da BGA varsa ve x86_64'te userspace init varsa ulaşılabilir.

### Sonuç
**Input sistemi** (klavye, fare, dokunmatik, kalem) **teknik olarak tüm 11 mimaride yazıldı ve derleniyor**. Ancak **masaüstüne ulaşmak** için öncelikle **geçerli bir framebuffer**, **doğru bellek haritası**, ve **çalışan MMU** gerekiyor.
