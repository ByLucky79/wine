# ÖZKAN-OS — Geliştirme Kuralları (Güncel: 2026.05)

> Bu dosya AI asistanın ve geliştiricinin uyması gereken bağlayıcı kurallardır.
> Kod yazmaya başlamadan önce mutlaka okunacak. Hiçbir kural atlanamaz.

---

## ═══════════════════════════════════════════════
## BÖLÜM A — KESİN KURALLAR (İhlal edilemez)
## ═══════════════════════════════════════════════

### KURAL 1 — DOSYA BAŞLIĞI (ZORUNLU)
Her üretilen Rust, C, C++ dosyasının en başına şu header **eksiksiz** yazılacak:

```rust
// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : [MODÜL ADI — tek cümle açıklama]
// Dosya Yolu         : [projeye göre tam göreceli yol, örn: kernel/system/core/mm.rs]
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   [Modülün ne yaptığının detaylı açıklaması — en az 2 satır]
//
// Bağımlı Dosyalar:
//   1-) [gerçek bağımlılık — tam yol veya crate adı]
//   2-) [gerçek bağımlılık]
//   (bağımlılık yoksa: "Yok")
//
//              Dosyaya Müdahaleler
// 2026-05-13      Dosya oluşturuldu
// *******************************************************************
```

**Header Kuralları:**
- `Dosya Yolu` alanı gerçek dosya yolunu gösterir — kopyala-yapıştır hata olmamalı
- `Bağımlı Dosyalar` alanı gerçek `use`/`mod`/`extern crate` bağımlılıklarını listeler — "Yok" veya gerçek yol
- `Dosya Görev Tanımı` tek bakışta modülün ne yaptığını anlatan kısa açıklama
- Assembly dosyaları için (`;` yorum), C/C++ için (`//` veya `/* */`) aynı format
- Dosya taşınır veya yeniden adlandırılırsa header **hemen güncellenir**

---

### KURAL 29 — MODÜLER DOSYA MİMARİSİ (KRİTİK)

**Kernel'e yük bindirmemek için her dosya tek bir sorumluluğa sahip olmalıdır.**

#### Dosya Boyutu Sınırları
| Konum | Maksimum Satır | Aşılırsa |
|-------|---------------|----------|
| `kernel/system/core/` | **600 satır** | Mutlaka bölünür |
| `kernel/hardware/` | **800 satır** | Mutlaka bölünür |
| `kernel/FS/` | **800 satır** | Mutlaka bölünür |
| `kernel/graphics/` | **700 satır** | Mutlaka bölünür |
| `boot/` | **400 satır** | Mutlaka bölünür |
| `apps/` | **1000 satır** | Mutlaka bölünür |
| Diğer | **800 satır** | Mutlaka bölünür |

#### Bölme Kuralları
- Bir dosya **tek bir sorumluluk** taşır (Single Responsibility Principle)
- İlgili fonksiyon grubu büyüdüğünde → yeni alt modül dosyası oluştur
- Örnek: `network_ids.rs` büyüdüğünde:
  ```
  network_ids/
  ├── mod.rs          ← sadece pub use + genel tanımlar
  ├── signatures.rs   ← imza tablosu
  ├── detection.rs    ← tespit motoru
  ├── actions.rs      ← aksiyon işleme
  └── stats.rs        ← istatistikler
  ```
- `mod.rs` dosyası yalnızca `pub use` ve struct tanımları içerir — işlevsel kod **alt modüllere** dağılır
- Büyük enum/struct koleksiyonları ayrı `_types.rs` dosyasına çıkarılır
- Test kodu ayrı `_test.rs` dosyasına çıkarılır

#### Kesinlikle Yasak
- Tek dosyaya birden fazla bağımsız sistem sıkıştırmak
- `main.rs` veya `bare.rs` içine doğrudan fonksiyon eklemek
- 600+ satırlık dosya içinde "şimdilik burada kalsın" mantığı
- Dosya büyüdükçe aynı dosyaya eklemeye devam etmek

#### Yeni Özellik Eklerken
1. Özellik mevcut hangi modüle ait? → o modül dosyasına ekle
2. Yeni bir alt sistem mi? → yeni dizin + görev odaklı isimli giriş dosyası + alt dosyalar oluştur
3. Tek satır bile olsa yanlış dosyaya ekleme — doğru modülü bul

---

### KURAL 30 — DOSYA İSİMLENDİRME (KESİN YASAK)

**Aşağıdaki genel amaçlı dosya isimleri KESİNLİKLE KULLANILMAZ:**

| Yasak İsim | Yerine Kullan |
|------------|---------------|
| `mod.rs` | `<modül_adı>_mod.rs` veya dizin giriş dosyası modülün adını taşır |
| `lib.rs` | `<crate_adı>_core.rs` veya `<crate_adı>_api.rs` |
| `main.rs` | `<uygulama_adı>_main.rs` veya `<uygulama_adı>_entry.rs` |
| `types.rs` | `<modül_adı>_types.rs` |
| `utils.rs` | `<modül_adı>_utils.rs` veya işleve göre özel isim |
| `helpers.rs` | işleve göre özel isim (örn: `format_helpers.rs`) |
| `common.rs` | `<alan>_common.rs` (örn: `fs_common.rs`) |
| `core.rs` | `<modül_adı>_core.rs` (örn: `scheduler_core.rs`) |

**Doğru isimlendirme örnekleri:**
```
YANLIŞ             →  DOĞRU
mod.rs             →  scheduler_cfs.rs / ozfs_mod.rs
lib.rs             →  kernel_core_api.rs
main.rs            →  ozsh_entry.rs / desktop_main.rs
types.rs           →  mm_types.rs / ipc_types.rs
utils.rs           →  string_utils.rs / path_utils.rs
common.rs          →  fs_common.rs / arch_common.rs
```

**Neden:** Dosya ismi tek bakışta ne yaptığını söylemeli. "mod", "lib", "main" hiçbir anlam taşımaz.  
Rust'ın `mod.rs` kuralı teknik olarak geçerli olsa da ÖZKAN-OS'ta yasaktır — dizin modülleri için  
`<dizin_adı>.rs` yöntemi veya `<modül_adı>_mod.rs` kullanılır.

---

### KURAL 31 — MEVCUT mod.rs / lib.rs / main.rs DOSYALARININ DÖNÜŞTÜRÜLMESİ
- Projede tespit edilen `mod.rs`, `lib.rs`, `main.rs` dosyaları bulunduğu sistemin adını alarak yeniden isimlendirilir
- Yeniden adlandırma sırasında Cargo.toml'daki `path` alanı güncellenir
- Derleme bozulmadan geçiş yapılır (sıfır hata/uyarı kontrolü)
- Periyodik tarama ile yeni oluşan yasak isimli dosyalar tespit edilip dönüştürülür

---

### KURAL 27 — UTF-8 KODLAMA (ZORUNLU)
- **Tüm kaynak dosyalar UTF-8 kodlamasında kaydedilir** — BOM (Byte Order Mark) olmadan
- Editör/araç fark etmeksizin: `.rs`, `.c`, `.cpp`, `.h`, `.asm`, `.s`, `.toml`, `.md` → hepsi UTF-8
- Türkçe karakterler (ğ, ü, ş, ı, ö, ç, Ğ, Ü, Ş, İ, Ö, Ç) doğru kodlanmalı
- UTF-8 olmayan dosya tespit edildiğinde **hemen dönüştürülür**
- Yeni dosya yazıldığında UTF-8 olduğu varsayılan olarak doğrulanır

---

### KURAL 28 — HEADER DOĞRULAMA VE GÜNCELLEME
- Mevcut dosyalarda `Dosya Yolu` yanlış veya eksikse **düzeltilir**
- `Bağımlı Dosyalar` alanı gerçek `use` ifadeleriyle uyumlu tutulur
- Dosya büyük değişiklik geçirdiğinde `Dosyaya Müdahaleler` bölümüne tarih + kısa açıklama eklenir
- Header yoksa **eklenir**; hatalıysa **düzeltilir**
- Periyodik tarama: her büyük oturumda tüm dosyalar header açısından kontrol edilir

---

### KURAL 2 — RUST ZORUNLU ÖZELLİKLERİ
- `#![no_std]` **yalnızca crate root dosyasında** kullanılır — alt modül `.rs` dosyalarına yazılmaz
- `static mut` **KESİNLİKLE KULLANILMAZ** → `AtomicXxx`, `UnsafeCell<T>` + `unsafe impl Sync`, veya `Mutex` kullan
- Port I/O ve donanım erişimi için `core::arch::asm!()` makrosu kullan
- Hata yönetimi için `Result<T, E>` kullan — `unwrap()` kernel kodunda yasak
- `#[must_use]` attribute'unu tüm `Result` ve `Option` döndüren public fonksiyonlara ekle
- Inline assembly'de `rbx`/`ebx` **doğrudan operand olamaz** (LLVM x86_64'te dahili) → push/pop rbx veya `{reg:e}` modifier kullan
- `lib.rs` ismi kullanılmaz → modülün işlevine uygun isim (örn: `boot_entry.rs`, `mm_core.rs`)

---

### KURAL 3 — DİL / MESAJ SİSTEMİ
- Kaynak kodda **doğrudan Türkçe veya başka dil string yazılmaz**
- Kullanıcıya gösterilen tüm metinler LangId sistemi üzerinden gelir:
  ```rust
  Lang::get(LangId::MSG_DISK_ERROR)
  ```
- Varsayılan dil: **TÜRKÇE** — kullanıcı değiştirmedikçe değişmez
- Dil değişimi hedefi: < 100 ms
- Yeni bir kullanıcı mesajı eklendiğinde mutlaka dil dosyalarına da eklenir:
  ```
  kernel/graphics/ui/lang_tr.rs   ← varsayılan
  kernel/graphics/ui/lang_en.rs
  kernel/graphics/ui/lang_de.rs
  kernel/graphics/ui/lang_ru.rs
  kernel/graphics/ui/lang_jp.rs
  kernel/graphics/ui/lang_ch.rs
  ```

---

### KURAL 4 — DOSYA ADLANDIRMA VE YOL
- Tüm dizin adları **küçük harf** olmalı (workspace kökü hariç mevcut `kernel/` yapısına dokunulmaz)
- Yeni dosyalar oluşturulurken modülün işlevini açıklayan isim verilir
- `lib.rs` ismi yasak — örnek: `ozfs_core.rs`, `scheduler_cfs.rs`, `boot_entry_x86_64.rs`
- Mimari özel kod: `#[cfg(target_arch = "x86_64")]` bloğu içine alınır

---

### KURAL 5 — DİZİN AĞACI GÜNCELLEMESİ
Her yeni klasör veya dosya oluşturulduğunda `Prompt.txt` dosyasındaki dizin ağacı güncellenir.

---

### KURAL 6 — AI_STATUS.md KAYDI
`E:\OZKAN-OS\DOCS\AI_STATUS.md` dosyası her oturumda okunur ve yapılan tüm işlemler buraya kaydedilir. Bu dosyayı okumadan iş başlamaz, bitirmeden iş bitmez.

---

### KURAL 7 — MODÜLERLİK (KRİTİK)
- **`kernel/system/core/main.rs` ve `bare.rs` dosyalarına doğrudan kod eklenmez**
- Her özellik kendi modül dosyasında geliştirilir, sonra ilgili crate'e entegre edilir
- Sürücüler `kernel/hardware/drivers/` altında bağımsız crate olarak kalır — kernel core'a gömülmez
- Grafik, ağ, depolama sürücüleri boot sonrası **lazy load** edilir
- Kernel core yalnızca şunları içerir: ARCH init, HAL soyutlama, scheduler, memory manager, IPC

---

### KURAL 8 — PERFORMANS HEDEFİ
| Metrik | Hedef | Referans |
|--------|-------|---------|
| Boot süresi | < 1.8 sn | Windows 11: ~9.2 sn |
| Uygulama açılışı | < 90 ms | Windows 11: ~310 ms |
| Dosya kopyalama (1 GB) | < 450 ms | Windows 11: ~1.1 sn |
| Ağ throughput | > 9.4 Gbps | — |
| Boşta RAM | < 512 MB | Windows 11: ~2.1 GB |
| Dil değişimi | < 100 ms | — |
| Kurtarma başlatma | < 10 sn | Windows 11: yok (BSOD) |

---

### KURAL 9 — SANDBOX — YABANCI KOD DAIMA İZOLE
- Tüm harici uygulamalar sandbox içinde çalışır: `namespace + cgroup + seccomp + landlock`
- Sandbox ihlali = **anında SIGKILL + audit log**
- Sandbox bypass kesinlikle mümkün olmayacak şekilde tasarlanır

---

### KURAL 10 — VARSAYILAN DİL
- Sistem varsayılan dili **TÜRKÇE** — kullanıcı değiştirmedikçe değişmez
- Tüm UI, menü, hata ekranları Türkçe görünür (LangId sistemi üzerinden)

---

### KURAL 11 — ESKİ DONANIM UYUMU
- 486DX4 100MHz + 8 MB RAM + IDE disk üzerinde boot edebilmeli
- Her modül CPUID/donanım probe ile özellikleri tespit eder
- Özellik yoksa **graceful degradation** (düzgün gerileme) zorunludur:
  - 486DX4 + GPU yok → scalar fallback, metin modu
  - 486DX4 + eski GPU → OpenCL 1.1 offload
  - Modern donanım → tam özellik seti

---

### KURAL 12 — MAVİ EKRAN YOK
- Kernel panic ekranı gösterilmez; hata izole edilir, sistem devam eder
- Sürücü hatası → unload + yeniden yükle → sistem devam
- Gerçek kernel panic → ÖZKAN-OS hata ekranı (kendi tasarımımız) → 30 sn geri sayım
- Hiçbir hata "sistem çöktü, yeniden başlatın" mesajıyla sonuçlanmaz

---

### KURAL 13 — YENİDEN BAŞLATMA ZORUNLU DEĞİL
- Sürücü güncellemesi → **livepatch** — yeniden başlatma yok
- Kernel modülü güncellemesi → **atomik hot-swap** — yeniden başlatma yok
- Uygulama güncellemesi → **delta güncelleme** — yeniden başlatma yok
- Büyük sürüm → kullanıcı zaman seçer, hibernate ile tüm pencereler açık döner

---

### KURAL 14 — SÜRÜM NUMARALAMA
- Format: **CalVer** `2026.05` (YYYY.MM)
- `Cargo.toml` workspace'te: `version = "2026.05"`
- Her yeni ay başında minor güncelleme yapılır

---

### KURAL 15 — DOSYA YAZMA HIZI
- Aynı anda **en fazla 2 dosya** kodlanır, kaydedilir, sonra devam edilir
- Token verimliliği için büyük dosyaları parçalara böl
- Yarım/stub/TODO bırakma — her dosya tamamlanmış olarak kaydedilir

---

### KURAL 16 — MESAJLAR DİL DOSYASINA
- Yeni kullanıcı mesajı eklendiğinde her zaman önce dil dosyasına eklenir
- Kod içine asla ham string yazmadan LangId üzerinden çağrılır (Kural 3 ile birlikte)

---

### KURAL 17 — AI_STATUS.md OKUMA ZORUNLULUĞU
Her oturum başında `E:\OZKAN-OS\DOCS\AI_STATUS.md` okunur. Yapılan tüm işlemler oturumun sonunda bu dosyaya kaydedilir. Atlamak yasak.

---

### KURAL 18 — KOD KALİTESİ VE HIZI
- Acele kod yazılmaz — rahat ve sakin çalışılır
- **TODO, STUB, yarım kod, bölüm kod kesinlikle teslim edilmez**
- Her dosya **tam implementasyon** olarak tamamlanır
- VMware ve QEMU'da ve belirtilen 11 mimaride çalışacak profesyonel düzeyde kod

---

### KURAL 19 — 11 mimari DESTEĞİ (TAM LİSTE)
Desteklenen mimariler — hepsi eş zamanlı desteklenir:

| # | Mimari | Hedef Rust Triple | Hedef Donanım |
|---|--------|-------------------|---------------|
| 1 | x86_64 | `x86_64-unknown-none` | Modern PC, sunucu |
| 2 | x86 (486DX4+) | `i686-unknown-linux-musl` | Eski PC, 486DX4 ve sonrası |
| 3 | AArch64 | `aarch64-unknown-none` | Telefon, RPi 4/5, Apple Silicon |
| 4 | ARM32 | `armv7a-none-eabi` | ARMv7, RPi 2/3, eski telefonlar |
| 5 | RISC-V 64 | `riscv64gc-unknown-none-elf` | VisionFive, yeni SBC |
| 6 | RISC-V 32 | `riscv32imac-unknown-none-elf` | Gömülü RISC-V |
| 7 | MIPS 32 | `mips-unknown-linux-musl` | Router, gömülü |
| 8 | MIPS (ortak) | `mips-unknown-linux-musl` | SGI Indy, PS2 IOP |
| 9 | PowerPC 32 | `powerpc-unknown-linux-gnu` | PowerMac G3/G4, iMac G3 |
| 10 | PowerPC 64 | `powerpc64-unknown-linux-gnu` | PowerMac G5, IBM POWER |
| 11 | LoongArch64 | `loongarch64-unknown-none` | Çin Loongson PC |
> **NOT:** m68k, SPARC, MIPS64, Alpha, VAX, HPPA, SH-4, IA-64 — **kalıcı olarak kaldırıldı**.
> Sebep: LLVM/Rust bu mimarileri desteklemiyor; QEMU bile çalıştıramıyor.
> GNU cross-binutils bağımlılığı "tamamen yerli" prensibine aykırı.

---

### KURAL 20 — ONAY GEREKMEYECEK İŞLEMLER
Kodlama, derleme, dosya oluşturma, düzenleme işlemleri için kullanıcıdan onay istenmez.
Yalnızca şu durumlarda onay istenecek:
- Tüm bir dizini silmek
- Dış sisteme push/yayın yapmak
- Geri alınamaz disk/veri değişikliği

---

### KURAL 21 — TOKEN VERİMLİLİĞİ
- Gereksiz tekrar, şablon metin, uzun açıklama yazılmaz
- Hatalı kodlama yapılmaz — ilk seferinde doğru yazılır
- Büyük dosyalar parçalı yazılır (Kural 15)

---

### KURAL 22 — SIFIR HATA, SIFIR UYARI (EN KRİTİK KURAL)
Dosya tamamlandıktan sonra mutlaka derleme yapılır:
- `cargo build --release --target <hedef> -p <paket>` ile ilgili mimari için kontrol
- Hiçbir `error` kabul edilmez
- Hiçbir `warning` kabul edilmez — `#[allow(...)]` ile uyarı **kapatılmaz**, **gerçekten düzeltilir**
- 11 mimari için ayrı ayrı derleme yapılır (Kural 23)

---

### KURAL 23 — DERLEME SIRASI (11 mimari)
Derleme yapılırken her mimari **ayrı ayrı** derlenir, her birinde sıfır hata/uyarı sağlanır:

```
1.  cargo build --release --target x86_64-unknown-none           -p kernel-core
2.  cargo build --release --target i686-unknown-linux-musl       -p ozkan-boot-x86
3.  cargo build --release --target aarch64-unknown-none          -p ozkan-boot-arm64
4.  cargo build --release --target armv7a-none-eabi              -p ozkan-boot-arm32
5.  cargo build --release --target riscv64gc-unknown-none-elf    -p ozkan-boot-riscv64
6.  cargo build --release --target riscv32imac-unknown-none-elf  -p ozkan-boot-riscv32
7.  cargo build --release --target mips-unknown-linux-musl       -p ozkan-boot-mips32
8.  cargo build --release --target mips-unknown-linux-musl       -p ozkan-boot-mips
9.  cargo build --release --target powerpc-unknown-linux-gnu     -p ozkan-boot-ppc32
10. cargo build --release --target powerpc64-unknown-linux-gnu   -p ozkan-boot-ppc64
11. cargo build --release --target loongarch64-unknown-none      -p ozkan-boot-loongarch64
# Alpha, VAX, HPPA, SH-4, IA-64 kalıcı olarak kaldırıldı (QEMU desteklemiyor)
```

---

### KURAL 24 — MİMARİYE GÖRE KOD AYIRMA
- Mimari özel kod `#[cfg(target_arch = "...")]` bloğu içinde yazılır
- Ortak kod `cfg` bloğu dışında kalır
- Her mimari için graceful degradation sağlanır

---

### KURAL 25 — MODÜLER MİMARİ YAPISI
Her mimari kendi dizininde bağımsız crate olarak bulunur:
```
BOOT/x86_64/       boot/x86_64/
BOOT/arm64/        boot/arm64/
kernel/hardware/hal/x86_64/
kernel/hardware/hal/aarch64/
...
```
Tüm mimariler workspace `Cargo.toml`'daki `members` listesinde yer alır.

---

### KURAL 26 — KALICI HAFIZA
- Bu 26 kural AI'ın kalıcı hafızasına (`MEMORY.md`) kaydedilir
- Her yeni oturumda rules.md okunmadan kodlamaya başlanmaz
- Kurallar değiştiğinde hafıza da güncellenir

---

## ═══════════════════════════════════════════════
## BÖLÜM B — PROJE DURUMU (2026.05 itibarıyla)
## ═══════════════════════════════════════════════

### Tamamlanan Sistemler

| Sistem | Durum | Konum |
|--------|-------|-------|
| Boot zinciri (x86_64) | ✅ Çalışıyor | `kernel/boot/ASM/` |
| 11 mimari boot stub | ✅ Derleniyor | `boot/*/` |
| HAL soyutlama katmanı | ✅ Tamamlandı | `kernel/hardware/hal/` |
| Kernel core (x86_64) | ✅ Çalışıyor | `kernel/system/core/` |
| VFS + 12 dosya sistemi | ✅ Tamamlandı | `kernel/FS/` |
| ÖzFS (gizli FS) | ✅ Tamamlandı | `kernel/FS/ozfs/` |
| GUI — Desktop | ✅ Çalışıyor | `kernel/graphics/ui/` |
| GUI — Boot splash | ✅ Çalışıyor | `kernel/graphics/ui/boot_splash.rs` |
| Güvenlik modülleri | ✅ Tamamlandı | `kernel/system/core/oz_*.rs` |
| Device Manager GUI | ✅ Tamamlandı | — |
| Disk Bridge (Bu Bilgisayar) | ✅ Tamamlandı | — |
| Mount Manager | ✅ Tamamlandı | — |
| MaxPerf modülleri | ✅ Tamamlandı | `kernel/system/core/maxperf/` |
| CPU özellik tespiti | ✅ Tamamlandı | `oz_anti_hack.rs`, `detect_cpu_features()` |

### Aktif Geliştirme

| Sistem | Durum | Öncelik |
|--------|-------|---------|
| Boot ↔ Kernel köprüsü (11 mimari) | 🔄 Devam | Yüksek |
| Sürücü altyapısı | 🔄 Devam | Yüksek |
| Ağ yığını (TCP/IP + XDP) | 📋 Planlandı | Orta |
| OzLivePatch | 📋 Planlandı | Orta |
| OzRecovery | 📋 Planlandı | Orta |
| Paket yöneticisi (OzPkg) | 📋 Planlandı | Düşük |

---

## ═══════════════════════════════════════════════
## BÖLÜM C — MİMARİ REFERANS
## ═══════════════════════════════════════════════

### Boot Zinciri (x86_64)
```
BIOS → Stage1 MBR (0x7C00) → Stage2 (0x7E00) → PM32 → 0x100000 (kernel)
```

### Bellek Haritası (x86_64)
```
0x0500-0x0508  Scratch (KERN_LBA, KERNEL_SECTORS)
0x7C00         Stage1 MBR
0x7E00         Stage2 Loader
0x10000        Geçici staging buffer
0x100000       Kernel flat binary (PVH note)
0x101000       pvh_start (kernel entry)
0x1000-0x7000  Page tables (pml4/pdpt/pd)
0xFD000000     LFB (Linear Framebuffer, BGA 1360x768x32)
```

### Başarılı Boot Serial Çıktısı
```
RLS........DXC!PTG6SBKOZKAN-OS kernel booted successfully!
```

### Derleme (Hızlı)
```powershell
build.bat     # Çift tıkla veya CMD'den
run.bat       # QEMU ile başlat
```

### GRUB YASAK
GRUB, UEFI bootloader, multiboot, syslinux **KESİNLİKLE KULLANILMAZ**.
Boot zinciri tamamen ÖZKAN-OS'a aittir.

---

## ═══════════════════════════════════════════════
## BÖLÜM D — DOSYA SİSTEMİ VE GÜVENLİK
## ═══════════════════════════════════════════════

### Desteklenen Dosya Sistemleri
| FS | Erişim | Notlar |
|----|--------|--------|
| FAT12/16/32 | R+W | Tam uyumlu |
| exFAT | R+W | Flash optimizeli |
| NTFS | R+W | MFT tam parse, journal |
| EXT3/EXT4 | R+W | Journal destekli |
| Btrfs | R+W | CoW + snapshot |
| ZFS | R+W | Pool yönetimi |
| HFS+/APFS | R+W | Journaled |
| ISO9660/UDF | R | Optik medya |
| HPFS | R | OS/2 legacy |
| **ÖzFS** | R+W | **Yalnızca ÖZKAN-OS okuyabilir** |

### ÖzFS Gizleme
- Magic: `0x4F5A4B414E4F5A46` ("OZKANOZF")
- Şifreleme: AES-256-XTS + XChaCha20 hibrit
- GPT GUID: özel (fdisk/diskpart tanıyamaz)
- Kernel ACL: root dahi doğrudan yazamaz

### Güvenlik Katmanları
1. Capability-Based Security (64 granüler yetki biti)
2. Full Namespace İzolasyonu (PID+NET+MNT+IPC+UTS+USER+TIME)
3. Seccomp-BPF + Landlock (syscall minimize)
4. Kernel Lockdown (varsayılan açık)
5. Sıfır Telemetri (hiç veri toplanmaz)
6. CFI — Control Flow Integrity (CET IBT + SHSTK, ARM BTI + PAC)
7. Shadow Stack (backward-edge CFI)
8. Memory Encryption (AMD SME/SEV, Intel TME/MKTME)
9. Verified Boot (ED25519 + TPM 2.0 PCR)
10. Post-Quantum Kriptografi (CRYSTALS-Kyber + Dilithium)

---

## ═══════════════════════════════════════════════
## BÖLÜM E — KULLANICI DENEYİMİ
## ═══════════════════════════════════════════════

### Tasarım İlkesi
Windows kullanıcısı ÖZKAN-OS'u kolayca kullanabilmeli.
Ancak Windows telif hakkı ve patent ihlali **kesinlikle yapılmaz**.
Özgün tasarım dili — piksel kopyası yok, marka öğesi yok.

### Yasak Fontlar
Arial, Times New Roman, Segoe UI, Calibri (ticari lisans)

### İzinli Fontlar
Inter, Noto Sans, Liberation Sans, Fira Sans, Fira Code,
Source Sans Pro, Source Code Pro, Arimo (OFL/Apache-2.0)

### Grafik Sistemi
- BGA (Bochs Graphics Adapter): 1360x768@32bpp
- LFB: `0xFD000000` (QEMU PCI bar0)
- Boot splash: `kernel/graphics/ui/boot_splash.rs`
- Desktop: `kernel/graphics/ui/gui/desktop/bare.rs`
- Tasarım referansı: `gui/icons/generators/ozkanos.html`

### Başlat Butonu
- Kalkan ikonu + "Başlat" yazısı
- **Dokunulmaz** — değiştirilmez

---

## ═══════════════════════════════════════════════
## BÖLÜM F — WORKSPACE YAPISI
## ═══════════════════════════════════════════════

```
E:\OZKAN-OS\
├── Cargo.toml              ← Workspace root (11 mimari member)
├── .cargo/config.toml      ← Hedef + rustflags
├── build.bat               ← Hızlı derleme
├── run.bat                 ← QEMU başlatma
├── rules.md                ← BU DOSYA
├── CLAUDE.md               ← AI rehberi
├── docs/AI_STATUS.md       ← İşlem kaydı
│
├── boot/                   ← 11 mimari boot stub (Rust)
│   ├── x86_64/entry.rs
│   ├── arm64/entry.rs
│   ├── riscv64/entry.rs
│   └── ...
│
├── kernel/
│   ├── boot/ASM/           ← Stage1 + Stage2 (Assembly — DOKUNMA)
│   ├── SYSTEM/core/        ← Kernel core (253+ Rust dosyası)
│   │   ├── maxperf/        ← Performans alt sistemi
│   │   └── oz_*.rs         ← Güvenlik modülleri
│   ├── HARDWARE/
│   │   ├── hal/            ← 11 mimari HAL
│   │   ├── ARCH/           ← Mimari özel kod
│   │   └── drivers/        ← Modüler sürücüler
│   ├── FS/                 ← 12 dosya sistemi + ÖzFS
│   └── GRAPHICS/           ← GUI, framebuffer, font
│
└── gui/icons/generators/   ← Tasarım referans HTML'leri
```

---

*Son güncelleme: 2026-05-13 — Oturum 58 sonrası*
*Proje: 1514+ Rust dosyası, 11 mimari, sıfır hata/uyarı*
