# ÖZKAN-OS Entegrasyon Planı — Modüllerin Mevcut Kod Tabanına Bağlanması

**Tarih:** 2026-04-27  
**Hazırlayan:** AI Agent + Kullanıcı Onayı  
**Kapsam:** 5 yeni modül (~1170 satır) → mevcut kernel init akışına entegrasyon

---

## 1. Tamamlanan Modüller (Silah Deposu Hazır)

| # | Modül | Dosya | Satır | Durum |
|---|-------|-------|-------|-------|
| 1 | `driver_heartbeat` | `kernel/system/core/driver_heartbeat.rs` | ~200 | ✅ Derleniyor, testleri var |
| 2 | `driver_quota` | `kernel/system/core/driver_quota.rs` | ~180 | ✅ Derleniyor, testleri var |
| 3 | `boot_timer` | `kernel/system/core/boot_timer.rs` | ~150 | ✅ Derleniyor, testleri var |
| 4 | `panic_recovery` | `kernel/system/core/panic_recovery.rs` | ~220 | ✅ Derleniyor, testleri var |
| 5 | `memory_budget` | `kernel/system/core/memory_budget.rs` | ~120 | ✅ Derleniyor, testleri var |

**Ortak özellik:** Tüm modüller `no_std`, `static mut` kullanmıyor, Atomic/Mutex tabanlı, unit test içeriyor.

---

## 2. Entegrasyon Noktaları

### 2.1 Boot Timer Entegrasyonu (Risk: SIFIR)

**Dosya:** `kernel/system/core/main.rs`

| Konum | Eklenecek Kod | Amaç |
|-------|---------------|------|
| `kernel_main_bare()` başlangıcı, `serial::init()` öncesi | `crate::boot_timer::mark_boot_start();` + `mark(BootPhase::KernelEntry);` | Boot süresi ölçümünü başlat |
| VBE/framebuffer init sonrası | `mark(BootPhase::DriversInit);` | Grafik init süresi |
| `init_drivers()` sonu, "Lazy driver registry ready" sonrası | `mark(BootPhase::PciEnum);` | PCI tarama süresi |
| `start_native_service_chain()` başlangıcı | `mark(BootPhase::DesktopReady);` | Desktop init süresi |

**Not:** Sadece atomik `u64` store'lar. Build kırılma riski yok.

---

### 2.2 Panic Recovery Entegrasyonu (Risk: SIFIR)

**Dosya:** `kernel/system/core/panic.rs`

| Konum | Eklenecek Kod | Amaç |
|-------|---------------|------|
| `PANIC_COUNT.fetch_add(1)` sonrası, satır ~149 | `crate::panic_recovery::record(PanicSource::KernelCore, 0, line, 0);` | Panic'i kategorize et, audit log'a yaz |
| Sonsuz döngü öncesi, `count >= MAX_PANIC_COUNT` | Mevcut `emergency_halt()` korunur, sadece kayıt eklenir | Davranış değişmez |

**Strateji dönüş değeri:** `panic_recovery::record()` `RecoveryStrategy` döner (Reboot, SafeMode, Halt). Şu an sadece log amaçlı kullanılabilir; ileride stratejiye göre davranış değişebilir.

---

### 2.3 Driver Heartbeat Entegrasyonu (Risk: ORTA)

**Dosyalar:** `kernel/hardware/drivers/input/mouse.rs` (PS/2) + `kernel/drivers/storage/ahci.rs` (AHCI)

**PS/2 Mouse Init:**
```rust
pub fn ps2_mouse_init() -> Result<(), Error> {
    // ... mevcut init kodu ...
    
    // EKLE: heartbeat kaydı + kota
    let _drv_id = crate::driver_heartbeat::register("ps2_mouse").unwrap_or(0);
    let _ = crate::driver_quota::register("ps2_mouse", crate::memory_budget::QUOTA_PS2);
    
    Ok(())
}
```

**PS/2 IRQ Handler:**
```rust
pub fn ps2_mouse_irq() {
    // EKLE: her IRQ'da ping
    crate::driver_heartbeat::ping_by_name("ps2_mouse");
    // ... mevcut IRQ kodu ...
}
```

**AHCI Init:**
```rust
pub unsafe fn ahci_hba_init(hba_base: usize) -> Result<(), DriverError> {
    // ... mevcut init kodu ...
    
    let _drv_id = crate::driver_heartbeat::register("ahci").unwrap_or(0);
    let _ = crate::driver_quota::register("ahci", crate::memory_budget::QUOTA_STORAGE);
    
    Ok(())
}
```

**Önemli Not:** `ping_by_name()` kullanımı, sürücü crate'lerinin `kernel-core`'e bağımlı olması (circular dependency) riskini ortadan kaldırır. `main.rs` ve `interrupt_handlers.rs` üzerinden entegrasyon yapılmalıdır.

---

### 2.4 Driver Quota Entegrasyonu (Risk: ORTA)

**Dosyalar:** AHCI DMA tahsis noktaları, USB DMA tahsis noktaları

**Örnek: DMA Buffer Alloc**
```rust
fn alloc_dma_buffer(size: u64) -> Result<DmaBuffer, Error> {
    // EKLE: kota kontrolü
    crate::driver_quota::alloc(DRV_ID, size)
        .map_err(|_| Error::OutOfMemory)?;
    
    let buf = page_allocator::alloc_pages(...)?;
    Ok(DmaBuffer { ptr: buf, size })
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        crate::driver_quota::free(DRV_ID, self.size);  // EKLE
        page_allocator::free_pages(self.ptr);
    }
}
```

**Zorluk:** Her sürücüde `DRV_ID` global/sabit tanımlanmalı. AHCI için `0x01`, USB için `0x02`, vb.

---

## 3. Önerilen Uygulama Sırası

| Sıra | Entegrasyon | Risk | Değer | Süre | Öncelik |
|------|-------------|------|-------|------|---------|
| 1 | Boot Timer (4 satır) | Sıfır | Yüksek (Kural 8 ölçüm) | 2 dk | 🔴 İlk |
| 2 | Panic Recovery (8 satır) | Sıfır | Yüksek (Kural 12) | 3 dk | 🔴 İkinci |
| 3 | Heartbeat (PS/2 + AHCI) | Orta | Orta | 15 dk | 🟠 Üçüncü |
| 4 | Quota (AHCI + USB DMA) | Orta | Orta | 20 dk | 🟠 Son |

---

## 4. Önerilen Seçenekler

### Seçenek A — Sadece Sıfır Riskli (#1 + #2)
Boot Timer + Panic Recovery hemen entegre edilir. Sürücü tarafına dokunulmaz. Build kırılma riski yok.

### Seçenek B — Tam Entegrasyon (4/4)
Tüm entegrasyonlar tek seferde yapılır. Sürücü dosya yolları bulunup eklenir. 30-40 dk.

### Seçenek C — Diff Dokümanı (Mevcut)
Bu doküman referans alınarak entegrasyon elle yapılır.

---

## 5. Ek Öneriler (Bu Dokümana Eklendi)

### 5.1 Heartbeat Ping Frequency
`driver_heartbeat::ping_by_name()` her IRQ'da çağrılması overhead yaratmaz (sadece atomic store). Ancak timer-based sürücülerde (örn. PIT tick) her tick'te ping atmak gereksizdir. Öneri: IRQ-based sürücülerde (PS/2, AHCI completion IRQ) her IRQ'da ping; polling-based sürücülerde her 1000 iterasyonda bir ping.

### 5.2 Quota ID Yönetimi
Sabit ID'ler yerine `driver_quota::register()` dönüş değeri (`u64`) kullanılmalı. Bu, çakışma riskini elimine eder.

### 5.3 Boot Timer Output
`boot_timer::report()` fonksiyonu seri port üzerinden JSON formatında çıktı verebilir. Örnek:
```
{"phase":"KernelEntry","us":1250}
{"phase":"DriversInit","us":8900}
{"phase":"PciEnum","us":12400}
{"phase":"DesktopReady","us":18500}
```
Bu, harici benchmark araçları tarafından parse edilebilir.

### 5.4 Kural 22 Uyumu
Her entegrasyon sonrası `cargo check -p kernel-core` ile derleme kontrolü yapılmalı. Sıfır hata, sıfır warning hedefi korunmalı.
