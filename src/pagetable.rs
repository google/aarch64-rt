// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Code to set up an initial pagetable.

use core::arch::naked_asm;

const MAIR_DEV_NGNRE: u64 = 0x04;
const MAIR_MEM_WBWA: u64 = 0xff;
/// The default value used for MAIR_ELx.
pub const DEFAULT_MAIR: u64 = MAIR_DEV_NGNRE | MAIR_MEM_WBWA << 8;

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
/// The default value used for TCR_EL1 or TCR_EL2.
#[cfg(not(feature = "el3"))]
pub const DEFAULT_TCR: u64 =
    TCR_TG0_4KB | TCR_TG1_4KB | TCR_EPD1 | TCR_RGN_OWB | TCR_RGN_IWB | TCR_SH_INNER | TCR_T0SZ_512;
/// The default value used for TCR_EL3.
#[cfg(feature = "el3")]
pub const DEFAULT_TCR: u64 = TCR_TG0_4KB | TCR_RGN_OWB | TCR_RGN_IWB | TCR_SH_INNER | TCR_T0SZ_512;

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
/// The default value used for SCTLR_ELx.
pub const DEFAULT_SCTLR: u64 = SCTLR_ELX_M
    | SCTLR_ELX_C
    | SCTLR_ELX_SA
    | SCTLR_ELX_ITD
    | SCTLR_ELX_SED
    | SCTLR_ELX_I
    | SCTLR_ELX_SPAN
    | SCTLR_ELX_RES1;

/// Provides an initial pagetable which can be used before any Rust code is run.
///
/// The `initial-pagetable` feature must be enabled for this to be used.
#[macro_export]
macro_rules! initial_pagetable {
    ($value:expr, $mair:expr, $tcr:expr, $sctlr:expr) => {
        #[unsafe(export_name = "initial_pagetable")]
        #[unsafe(link_section = ".rodata.initial_pagetable")]
        static INITIAL_PAGETABLE: $crate::InitialPagetable = $value;

        $crate::__enable_mmu!($mair, $tcr, $sctlr);
    };
    ($value:expr, $mair:expr) => {
        initial_pagetable!($value, $mair, $crate::DEFAULT_TCR, $crate::DEFAULT_SCTLR);
    };
    ($value:expr) => {
        initial_pagetable!(
            $value,
            $crate::DEFAULT_MAIR,
            $crate::DEFAULT_TCR,
            $crate::DEFAULT_SCTLR
        );
    };
}

/// Enables the MMU and caches, assuming that we are running at EL1.
///
/// # Safety
///
/// This function doesn't follow the standard aarch64 calling convention. It must only be called
/// from assembly code, early in the boot process.
///
/// Expects the MAIR value in x8, the TCR value in x9, and the SCTLR value in x10.
///
/// Clobbers x8-x9.
#[unsafe(naked)]
pub unsafe extern "C" fn enable_mmu_el1() {
    naked_asm!(
        // Load and apply the memory management configuration, ready to enable MMU and
        // caches.
        "msr mair_el1, x8",
        "adrp x8, initial_pagetable",
        "msr ttbr0_el1, x8",
        // Copy the supported PA range into TCR_EL1.IPS.
        "mrs x8, id_aa64mmfr0_el1",
        "bfi x9, x8, #32, #4",
        "msr tcr_el1, x9",
        // Ensure everything before this point has completed, then invalidate any
        // potentially stale local TLB entries before they start being used.
        "isb",
        "tlbi vmalle1",
        "ic iallu",
        "dsb nsh",
        "isb",
        // Configure SCTLR_EL1 to enable MMU and cache and don't proceed until this has
        // completed.
        "msr sctlr_el1, x10",
        "isb",
        "ret"
    );
}

/// Enables the MMU and caches, assuming that we are running at EL2.
///
/// # Safety
///
/// This function doesn't follow the standard aarch64 calling convention. It must only be called
/// from assembly code, early in the boot process.
///
/// Expects the MAIR value in x8, the TCR value in x9, and the SCTLR value in x10.
///
/// Clobbers x8-x9.
#[unsafe(naked)]
pub unsafe extern "C" fn enable_mmu_el2() {
    naked_asm!(
        // Load and apply the memory management configuration, ready to enable MMU and
        // caches.
        "msr mair_el2, x8",
        "adrp x8, initial_pagetable",
        "msr ttbr0_el2, x8",
        // Copy the supported PA range into TCR_EL2.IPS.
        "mrs x8, id_aa64mmfr0_el1",
        "bfi x9, x8, #32, #4",
        "msr tcr_el2, x9",
        // Ensure everything before this point has completed, then invalidate any
        // potentially stale local TLB entries before they start being used.
        "isb",
        "tlbi vmalle1",
        "ic iallu",
        "dsb nsh",
        "isb",
        // Configure SCTLR_EL2 to enable MMU and cache and don't proceed until this has
        // completed.
        "msr sctlr_el2, x10",
        "isb",
        "ret"
    );
}

/// Enables the MMU and caches, assuming that we are running at EL3.
///
/// # Safety
///
/// This function doesn't follow the standard aarch64 calling convention. It must only be called
/// from assembly code, early in the boot process.
///
/// Expects the MAIR value in x8, the TCR value in x9, and the SCTLR value in x10.
///
/// Clobbers x8-x9.
#[unsafe(naked)]
pub unsafe extern "C" fn enable_mmu_el3() {
    naked_asm!(
        // Load and apply the memory management configuration, ready to enable MMU and
        // caches.
        "msr mair_el3, x8",
        "adrp x8, initial_pagetable",
        "msr ttbr0_el3, x8",
        // Copy the supported PA range into TCR_EL3.IPS.
        "mrs x8, id_aa64mmfr0_el1",
        "bfi x9, x8, #32, #4",
        "msr tcr_el3, x9",
        // Ensure everything before this point has completed, then invalidate any
        // potentially stale local TLB entries before they start being used.
        "isb",
        "tlbi vmalle1",
        "ic iallu",
        "dsb nsh",
        "isb",
        // Configure SCTLR_EL3 to enable MMU and cache and don't proceed until this has
        // completed.
        "msr sctlr_el3, x10",
        "isb",
        "ret"
    );
}

/// Macro used internally by [`initial_pagetable!`]. Shouldn't be used directly.
#[cfg(feature = "el1")]
#[doc(hidden)]
#[macro_export]
macro_rules! __enable_mmu {
    ($mair:expr, $tcr:expr, $sctlr:expr) => {
        core::arch::global_asm!(
            r".macro mov_i, reg:req, imm:req",
                r"movz \reg, :abs_g3:\imm",
                r"movk \reg, :abs_g2_nc:\imm",
                r"movk \reg, :abs_g1_nc:\imm",
                r"movk \reg, :abs_g0_nc:\imm",
            r".endm",

            ".section .init, \"ax\"",
            ".global enable_mmu",
            "enable_mmu:",
                "mov_i x8, {MAIR_VALUE}",
                "mov_i x9, {TCR_VALUE}",
                "mov_i x10, {SCTLR_VALUE}",

                "b {enable_mmu_el1}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            TCR_VALUE = const $tcr,
            SCTLR_VALUE = const $sctlr,
            enable_mmu_el1 = sym $crate::enable_mmu_el1,
        );
    };
}

/// Macro used internally by [`initial_pagetable!`]. Shouldn't be used directly.
#[cfg(feature = "el2")]
#[doc(hidden)]
#[macro_export]
macro_rules! __enable_mmu {
    ($mair:expr, $tcr:expr, $sctlr:expr) => {
        core::arch::global_asm!(
            r".macro mov_i, reg:req, imm:req",
                r"movz \reg, :abs_g3:\imm",
                r"movk \reg, :abs_g2_nc:\imm",
                r"movk \reg, :abs_g1_nc:\imm",
                r"movk \reg, :abs_g0_nc:\imm",
            r".endm",

            ".section .init, \"ax\"",
            ".global enable_mmu",
            "enable_mmu:",
                "mov_i x8, {MAIR_VALUE}",
                "mov_i x9, {TCR_VALUE}",
                "mov_i x10, {SCTLR_VALUE}",

                "b {enable_mmu_el2}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            TCR_VALUE = const $tcr,
            SCTLR_VALUE = const $sctlr,
            enable_mmu_el2 = sym $crate::enable_mmu_el2,
        );
    };
}

/// Macro used internally by [`initial_pagetable!`]. Shouldn't be used directly.
#[cfg(feature = "el3")]
#[doc(hidden)]
#[macro_export]
macro_rules! __enable_mmu {
    ($mair:expr, $tcr:expr, $sctlr:expr) => {
        core::arch::global_asm!(
            r".macro mov_i, reg:req, imm:req",
                r"movz \reg, :abs_g3:\imm",
                r"movk \reg, :abs_g2_nc:\imm",
                r"movk \reg, :abs_g1_nc:\imm",
                r"movk \reg, :abs_g0_nc:\imm",
            r".endm",

            ".section .init, \"ax\"",
            ".global enable_mmu",
            "enable_mmu:",
                "mov_i x8, {MAIR_VALUE}",
                "mov_i x9, {TCR_VALUE}",
                "mov_i x10, {SCTLR_VALUE}",

                "b {enable_mmu_el3}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            TCR_VALUE = const $tcr,
            SCTLR_VALUE = const $sctlr,
            enable_mmu_el3 = sym $crate::enable_mmu_el3,
        );
    };
}

/// A hardcoded pagetable.
#[repr(C, align(4096))]
pub struct InitialPagetable(pub [usize; 512]);
