// *******************************************************************
//                    ÖZKAN-OS İşletim Sistemi
//
// Dosya Görev Tanımı : Win32 sanal bellek bölge yöneticisi.
// Dosya Yolu         : compat/win32/win32_virtual_memory.rs
// Yazar              : Özkan Yıldırım
// Lisans             : GPLv3
//
// Desteklediği İşlemciler:
//   486DX4, x86, x86_64, ARM 32, ARM64,
//   RISC-V 32, RISC-V 64, MIPS 32,
//   PowerPC 32, PowerPC 64, LoongArch64
//
// Açıklama:
//   Bu dosya Win32 tarzı reserve/commit/protect/release işlemleri için
//   süreç sanal bellek bölgelerini izleyen ortak yöneticiyi sağlar.
//
// Bağımlı Dosyalar:
//   1-) compat/win32/win32_base_types.rs
//   2-) compat/win32/process_mgr.rs
//   3-) compat/win32/pe_loader.rs
//
//              Dosyaya Müdahaleler
// 2026-05-16      Ortak Win32 sanal bellek yöneticisi dosyası oluşturuldu
// *******************************************************************

#![allow(dead_code)]

use alloc::vec::Vec;

use crate::win32::win32_base_types::{
    WinDword, WinProtection, WinSizeT, WIN_ERROR_INVALID_PARAMETER, WIN_ERROR_NOT_ENOUGH_MEMORY,
    WIN_MEM_COMMIT, WIN_MEM_DECOMMIT, WIN_MEM_RELEASE, WIN_MEM_RESERVE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinMemoryState {
    Free,
    Reserved,
    Committed,
}

#[derive(Debug, Clone, Copy)]
pub struct WinMemoryRegion {
    pub base: u64,
    pub size: WinSizeT,
    pub state: WinMemoryState,
    pub protection: WinProtection,
    pub allocation_type: WinDword,
}

impl WinMemoryRegion {
    pub const fn new(
        base: u64,
        size: WinSizeT,
        state: WinMemoryState,
        protection: WinProtection,
        allocation_type: WinDword,
    ) -> Self {
        Self {
            base,
            size,
            state,
            protection,
            allocation_type,
        }
    }

    pub fn contains(&self, address: u64) -> bool {
        let end = self.base.saturating_add(self.size as u64);
        address >= self.base && address < end
    }
}

#[derive(Debug, Default)]
pub struct WinVirtualMemoryManager {
    next_base: u64,
    regions: Vec<WinMemoryRegion>,
}

impl WinVirtualMemoryManager {
    pub fn new() -> Self {
        Self {
            next_base: 0x0000_0001_0000_0000,
            regions: Vec::new(),
        }
    }

    pub fn reserve(
        &mut self,
        size: WinSizeT,
        protection: WinProtection,
    ) -> Result<u64, WinDword> {
        if size == 0 {
            return Err(WIN_ERROR_INVALID_PARAMETER);
        }

        let aligned_size = Self::align_up(size as u64, 0x1000) as WinSizeT;
        let base = self.next_base;
        self.next_base = self.next_base.saturating_add(aligned_size as u64).saturating_add(0x1000);

        self.regions.push(WinMemoryRegion::new(
            base,
            aligned_size,
            WinMemoryState::Reserved,
            protection,
            WIN_MEM_RESERVE,
        ));
        Ok(base)
    }

    pub fn commit(&mut self, base: u64, size: WinSizeT, protection: WinProtection) -> Result<(), WinDword> {
        if size == 0 {
            return Err(WIN_ERROR_INVALID_PARAMETER);
        }

        let region = self
            .regions
            .iter_mut()
            .find(|region| region.base == base)
            .ok_or(WIN_ERROR_INVALID_PARAMETER)?;

        if size > region.size {
            return Err(WIN_ERROR_NOT_ENOUGH_MEMORY);
        }

        region.state = WinMemoryState::Committed;
        region.protection = protection;
        region.allocation_type |= WIN_MEM_COMMIT;
        Ok(())
    }

    pub fn reserve_and_commit(
        &mut self,
        size: WinSizeT,
        protection: WinProtection,
    ) -> Result<u64, WinDword> {
        let base = self.reserve(size, protection)?;
        self.commit(base, size, protection)?;
        Ok(base)
    }

    pub fn protect(&mut self, base: u64, protection: WinProtection) -> Result<WinProtection, WinDword> {
        let region = self
            .regions
            .iter_mut()
            .find(|region| region.base == base)
            .ok_or(WIN_ERROR_INVALID_PARAMETER)?;
        let previous = region.protection;
        region.protection = protection;
        Ok(previous)
    }

    pub fn decommit(&mut self, base: u64) -> Result<(), WinDword> {
        let region = self
            .regions
            .iter_mut()
            .find(|region| region.base == base)
            .ok_or(WIN_ERROR_INVALID_PARAMETER)?;
        region.state = WinMemoryState::Reserved;
        region.allocation_type |= WIN_MEM_DECOMMIT;
        Ok(())
    }

    pub fn release(&mut self, base: u64) -> Result<(), WinDword> {
        let index = self
            .regions
            .iter()
            .position(|region| region.base == base)
            .ok_or(WIN_ERROR_INVALID_PARAMETER)?;
        let mut region = self.regions.remove(index);
        region.state = WinMemoryState::Free;
        region.allocation_type |= WIN_MEM_RELEASE;
        Ok(())
    }

    pub fn query(&self, address: u64) -> Option<WinMemoryRegion> {
        self.regions.iter().copied().find(|region| region.contains(address))
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    pub fn regions(&self) -> &[WinMemoryRegion] {
        &self.regions
    }

    const fn align_up(value: u64, alignment: u64) -> u64 {
        if alignment == 0 {
            value
        } else {
            (value + alignment - 1) & !(alignment - 1)
        }
    }
}
