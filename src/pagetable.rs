// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Code to set up an initial pagetable.

use core::arch::global_asm;

const MAIR_DEV_NGNRE: u64 = 0x04;
const MAIR_MEM_WBWA: u64 = 0xff;
const MAIR_VALUE: u64 = MAIR_DEV_NGNRE | MAIR_MEM_WBWA << 8;

/// 4 KiB granule size for TTBR0_ELx.
const TCR_TG0_4KB: u64 = 0x0 << 14;
/// 4 KiB granule size for TTBR1_ELx.
#[cfg(not(feature = "el3"))]
const TCR_TG1_4KB: u64 = 0x2 << 30;
/// Disable translation table walk for TTBR1_ELx, generating a translation fault instead.
#[cfg(not(feature = "el3"))]
const TCR_EPD1: u64 = 0x1 << 23;
/// Translation table walks for TTBR0_ELx are inner sharable.
const TCR_SH_INNER: u64 = 0x3 << 12;
/// Translation table walks for TTBR0_ELx are outer write-back read-allocate write-allocate
/// cacheable.
const TCR_RGN_OWB: u64 = 0x1 << 10;
/// Translation table walks for TTBR0_ELx are inner write-back read-allocate write-allocate
/// cacheable.
const TCR_RGN_IWB: u64 = 0x1 << 8;
/// Size offset for TTBR0_ELx is 2**39 bytes (512 GiB).
const TCR_T0SZ_512: u64 = 64 - 39;
#[cfg(not(feature = "el3"))]
const TCR_VALUE: u64 =
    TCR_TG0_4KB | TCR_TG1_4KB | TCR_EPD1 | TCR_RGN_OWB | TCR_RGN_IWB | TCR_SH_INNER | TCR_T0SZ_512;
#[cfg(feature = "el3")]
const TCR_VALUE_EL3: u64 = TCR_TG0_4KB | TCR_RGN_OWB | TCR_RGN_IWB | TCR_SH_INNER | TCR_T0SZ_512;

/// Stage 1 instruction access cacheability is unaffected.
const SCTLR_ELX_I: u64 = 0x1 << 12;
/// SP alignment fault if SP is not aligned to a 16 byte boundary.
const SCTLR_ELX_SA: u64 = 0x1 << 3;
/// Stage 1 data access cacheability is unaffected.
const SCTLR_ELX_C: u64 = 0x1 << 2;
/// EL0 and EL1 stage 1 MMU enabled.
const SCTLR_ELX_M: u64 = 0x1 << 0;
/// Privileged Access Never is unchanged on taking an exception to ELx.
const SCTLR_ELX_SPAN: u64 = 0x1 << 23;
/// SETEND instruction disabled at EL0 in aarch32 mode.
const SCTLR_ELX_SED: u64 = 0x1 << 8;
/// Various IT instructions are disabled at EL0 in aarch32 mode.
const SCTLR_ELX_ITD: u64 = 0x1 << 7;
const SCTLR_ELX_RES1: u64 = (0x1 << 11) | (0x1 << 20) | (0x1 << 22) | (0x1 << 28) | (0x1 << 29);
const SCTLR_VALUE: u64 = SCTLR_ELX_M
    | SCTLR_ELX_C
    | SCTLR_ELX_SA
    | SCTLR_ELX_ITD
    | SCTLR_ELX_SED
    | SCTLR_ELX_I
    | SCTLR_ELX_SPAN
    | SCTLR_ELX_RES1;

#[cfg(feature = "el1")]
global_asm!(
    include_str!("el1_enable_mmu.S"),
    MAIR_VALUE = const MAIR_VALUE,
    TCR_VALUE = const TCR_VALUE,
    SCTLR_VALUE = const SCTLR_VALUE,
);
#[cfg(feature = "el2")]
global_asm!(
    include_str!("el2_enable_mmu.S"),
    MAIR_VALUE = const MAIR_VALUE,
    TCR_VALUE = const TCR_VALUE,
    SCTLR_VALUE = const SCTLR_VALUE,
);
#[cfg(feature = "el3")]
global_asm!(
    include_str!("el3_enable_mmu.S"),
    MAIR_VALUE = const MAIR_VALUE,
    TCR_VALUE = const TCR_VALUE_EL3,
    SCTLR_VALUE = const SCTLR_VALUE,
);

/// Provides an initial pagetable which can be used before any Rust code is run.
///
/// The `initial-pagetable` feature must be enabled for this to be used.
#[macro_export]
macro_rules! initial_pagetable {
    ($value:expr) => {
        #[unsafe(export_name = "initial_pagetable")]
        #[unsafe(link_section = ".rodata.initial_pagetable")]
        static INITIAL_PAGETABLE: $crate::InitialPagetable = $value;
    };
}

/// A hardcoded pagetable.
#[repr(C, align(4096))]
pub struct InitialPagetable(pub [usize; 512]);
