// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Code to set up an initial pagetable.

use core::arch::naked_asm;

const MAIR_DEV_NGNRE: u64 = 0x04;
const MAIR_MEM_WBWA: u64 = 0xff;
/// The default value used for MAIR_ELx.
pub const DEFAULT_MAIR: u64 = MAIR_DEV_NGNRE | MAIR_MEM_WBWA << 8;

/// 4 KiB granule size for TTBR1_ELx.
const TCR_TG1_4KB: u64 = 0x2 << 30;
/// Disable translation table walk for TTBR1_ELx, generating a translation fault instead.
const TCR_EPD1: u64 = 0x1 << 23;
/// 40 bits, 1 TiB.
const TCR_EL1_IPS_1TB: u64 = 0x2 << 32;
/// 40 bits, 1 TiB.
const TCR_EL2_PS_1TB: u64 = 0x2 << 16;
/// 4 KiB granule size for TTBR0_ELx.
const TCR_TG0_4KB: u64 = 0x0 << 14;
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
/// The default value used for TCR_EL1.
pub const DEFAULT_TCR_EL1: u64 = TCR_EL1_IPS_1TB
    | TCR_TG1_4KB
    | TCR_EPD1
    | TCR_TG0_4KB
    | TCR_SH_INNER
    | TCR_RGN_OWB
    | TCR_RGN_IWB
    | TCR_T0SZ_512;
/// The default value used for TCR_EL2.
pub const DEFAULT_TCR_EL2: u64 =
    TCR_EL2_PS_1TB | TCR_TG0_4KB | TCR_SH_INNER | TCR_RGN_OWB | TCR_RGN_IWB | TCR_T0SZ_512;
/// The default value used for TCR_EL3.
pub const DEFAULT_TCR_EL3: u64 =
    TCR_TG0_4KB | TCR_RGN_OWB | TCR_RGN_IWB | TCR_SH_INNER | TCR_T0SZ_512;

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
#[cfg(any(feature = "el1", feature = "el2", feature = "el3"))]
#[macro_export]
macro_rules! initial_pagetable {
    ($value:expr, $mair:expr, $sctlr:expr, $tcr:expr) => {
        static INITIAL_PAGETABLE: $crate::InitialPagetable = $value;

        $crate::enable_mmu!(INITIAL_PAGETABLE, $mair, $tcr, $sctlr);
    };
    ($value:expr, $mair:expr) => {
        $crate::initial_pagetable!($value, $mair, $crate::DEFAULT_SCTLR, $crate::DEFAULT_TCR);
    };
    ($value:expr) => {
        $crate::initial_pagetable!(
            $value,
            $crate::DEFAULT_MAIR,
            $crate::DEFAULT_SCTLR,
            $crate::DEFAULT_TCR
        );
    };
}

/// Provides an initial pagetable which can be used before any Rust code is run.
///
/// The `initial-pagetable` feature must be enabled for this to be used.
#[cfg(not(any(feature = "el1", feature = "el2", feature = "el3")))]
#[macro_export]
macro_rules! initial_pagetable {
    ($value:expr, $mair:expr, $sctlr:expr, $tcr_el1:expr, $tcr_el2:expr, $tcr_el3:expr) => {
        static INITIAL_PAGETABLE: $crate::InitialPagetable = $value;

        $crate::enable_mmu!(
            INITIAL_PAGETABLE,
            $mair,
            $sctlr,
            $tcr_el1,
            $tcr_el2,
            $tcr_el3
        );
    };
    ($value:expr, $mair:expr) => {
        initial_pagetable!(
            $value,
            $mair,
            $crate::DEFAULT_SCTLR,
            $crate::DEFAULT_TCR_EL1,
            $crate::DEFAULT_TCR_EL2,
            $crate::DEFAULT_TCR_EL3
        );
    };
    ($value:expr) => {
        initial_pagetable!(
            $value,
            $crate::DEFAULT_MAIR,
            $crate::DEFAULT_SCTLR,
            $crate::DEFAULT_TCR_EL1,
            $crate::DEFAULT_TCR_EL2,
            $crate::DEFAULT_TCR_EL3
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
/// Expects the MAIR value in x8, the SCTLR value in x9, the TCR value in x10 and the root pagetable
/// address in x11.
///
/// Clobbers x8-x9.
#[doc(hidden)]
#[unsafe(naked)]
pub unsafe extern "C" fn __enable_mmu_el1() {
    naked_asm!(
        // Load and apply the memory management configuration, ready to enable MMU and
        // caches.
        "msr mair_el1, x8",
        "msr ttbr0_el1, x11",
        // Copy the supported PA range into TCR_EL1.IPS.
        "mrs x8, id_aa64mmfr0_el1",
        "bfi x10, x8, #32, #4",
        "msr tcr_el1, x10",
        // Ensure everything before this point has completed, then invalidate any
        // potentially stale local TLB entries before they start being used.
        "isb",
        "tlbi vmalle1",
        "ic iallu",
        "dsb nsh",
        "isb",
        // Configure SCTLR_EL1 to enable MMU and cache and don't proceed until this has
        // completed.
        "msr sctlr_el1, x9",
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
/// Expects the MAIR value in x8, the SCTLR value in x9, the TCR value in x10 and the root pagetable
/// address in x11.
///
/// Clobbers x8-x9.
#[doc(hidden)]
#[unsafe(naked)]
pub unsafe extern "C" fn __enable_mmu_el2() {
    naked_asm!(
        // Load and apply the memory management configuration, ready to enable MMU and
        // caches.
        "msr mair_el2, x8",
        "msr ttbr0_el2, x11",
        // Copy the supported PA range into TCR_EL2.IPS.
        "mrs x8, id_aa64mmfr0_el1",
        "bfi x10, x8, #32, #4",
        "msr tcr_el2, x10",
        // Ensure everything before this point has completed, then invalidate any
        // potentially stale local TLB entries before they start being used.
        "isb",
        "tlbi vmalle1",
        "ic iallu",
        "dsb nsh",
        "isb",
        // Configure SCTLR_EL2 to enable MMU and cache and don't proceed until this has
        // completed.
        "msr sctlr_el2, x9",
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
/// Expects the MAIR value in x8, the SCTLR value in x9, the TCR value in x10 and the root pagetable
/// address in x11.
///
/// Clobbers x8-x9.
#[doc(hidden)]
#[unsafe(naked)]
pub unsafe extern "C" fn __enable_mmu_el3() {
    naked_asm!(
        // Load and apply the memory management configuration, ready to enable MMU and
        // caches.
        "msr mair_el3, x8",
        "msr ttbr0_el3, x11",
        // Copy the supported PA range into TCR_EL3.IPS.
        "mrs x8, id_aa64mmfr0_el1",
        "bfi x10, x8, #32, #4",
        "msr tcr_el3, x10",
        // Ensure everything before this point has completed, then invalidate any
        // potentially stale local TLB entries before they start being used.
        "isb",
        "tlbi vmalle1",
        "ic iallu",
        "dsb nsh",
        "isb",
        // Configure SCTLR_EL3 to enable MMU and cache and don't proceed until this has
        // completed.
        "msr sctlr_el3, x9",
        "isb",
        "ret"
    );
}

/// Generates assembly code to enable the MMU and caches with the given initial pagetable before any
/// Rust code is run.
///
/// This may be used indirectly via the [`initial_pagetable!`] macro.
#[cfg(feature = "el1")]
#[macro_export]
macro_rules! enable_mmu {
    ($pagetable:path, $mair:expr, $sctlr:expr, $tcr:expr) => {
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
                "mov_i x9 {SCTLR_VALUE}",
                "mov_i x10, {TCR_VALUE}",
                "adrp x11, {pagetable}",

                "b {enable_mmu_el1}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            SCTLR_VALUE = const $sctlr,
            TCR_VALUE = const $tcr,
            pagetable = sym $pagetable,
            enable_mmu_el1 = sym $crate::__private::__enable_mmu_el1,
        );
    };
    ($pagetable:path) => {
        $crate::enable_mmu!($pagetable, $crate::DEFAULT_MAIR, $crate::DEFAULT_SCTLR, $crate::DEFAULT_TCR_EL1);
    };
}

/// Generates assembly code to enable the MMU and caches with the given initial pagetable before any
/// Rust code is run.
///
/// This may be used indirectly via the [`initial_pagetable!`] macro.
#[cfg(feature = "el2")]
#[macro_export]
macro_rules! enable_mmu {
    ($pagetable:path, $mair:expr, $sctlr:expr, $tcr:expr) => {
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
                "mov_i x9, {SCTLR_VALUE}",
                "mov_i x10, {TCR_VALUE}",
                "adrp x11, {pagetable}",

                "b {enable_mmu_el2}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            SCTLR_VALUE = const $sctlr,
            TCR_VALUE = const $tcr,
            pagetable = sym $pagetable,
            enable_mmu_el2 = sym $crate::__private::__enable_mmu_el2,
        );
    };
    ($pagetable:path) => {
        $crate::enable_mmu!($pagetable, $crate::DEFAULT_MAIR, $crate::DEFAULT_SCTLR, $crate::DEFAULT_TCR_EL2);
    };
}

/// Generates assembly code to enable the MMU and caches with the given initial pagetable before any
/// Rust code is run.
///
/// This may be used indirectly via the [`initial_pagetable!`] macro.
#[cfg(feature = "el3")]
#[macro_export]
macro_rules! enable_mmu {
    ($pagetable:path, $mair:expr, $sctlr:expr, $tcr:expr) => {
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
                "mov_i x9, {SCTLR_VALUE}",
                "mov_i x10, {TCR_VALUE}",
                "adrp x11, {pagetable}",

                "b {enable_mmu_el3}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            SCTLR_VALUE = const $sctlr,
            TCR_VALUE = const $tcr,
            pagetable = sym $pagetable,
            enable_mmu_el3 = sym $crate::__private::__enable_mmu_el3,
        );
    };
    ($pagetable:path) => {
        $crate::enable_mmu!($pagetable, $crate::DEFAULT_MAIR, $crate::DEFAULT_SCTLR, $crate::DEFAULT_TCR_EL3);
    };
}

/// Generates assembly code to enable the MMU and caches with the given initial pagetable before any
/// Rust code is run.
///
/// This may be used indirectly via the [`initial_pagetable!`] macro.
#[cfg(not(any(feature = "el1", feature = "el2", feature = "el3")))]
#[macro_export]
macro_rules! enable_mmu {
    ($pagetable:path, $mair:expr, $sctlr:expr, $tcr_el1:expr, $tcr_el2:expr, $tcr_el3:expr) => {
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
                "mov_i x9, {SCTLR_VALUE}",
                "adrp x11, {pagetable}",

                "mrs x12, CurrentEL",
                "ubfx x12, x12, #2, #2",

                "cmp x12, #3",
                "b.ne 0f",
                "mov_i x10, {TCR_EL3_VALUE}",
                "b {enable_mmu_el3}",
            "0:",
                "cmp x12, #2",
                "b.ne 1f",
                "mov_i x10, {TCR_EL2_VALUE}",
                "b {enable_mmu_el2}",
            "1:",
                "mov_i x10, {TCR_EL1_VALUE}",
                "b {enable_mmu_el1}",

            ".purgem mov_i",
            MAIR_VALUE = const $mair,
            SCTLR_VALUE = const $sctlr,
            TCR_EL1_VALUE = const $tcr_el1,
            TCR_EL2_VALUE = const $tcr_el2,
            TCR_EL3_VALUE = const $tcr_el3,
            pagetable = sym $pagetable,
            enable_mmu_el1 = sym $crate::__private::__enable_mmu_el1,
            enable_mmu_el2 = sym $crate::__private::__enable_mmu_el2,
            enable_mmu_el3 = sym $crate::__private::__enable_mmu_el3,
        );
    };
    ($pagetable:path) => {
        $crate::enable_mmu!(
            $pagetable,
            $crate::DEFAULT_MAIR,
            $crate::DEFAULT_SCTLR,
            $crate::DEFAULT_TCR_EL1,
            $crate::DEFAULT_TCR_EL2,
            $crate::DEFAULT_TCR_EL3
        );
    };
}

/// A hardcoded pagetable.
#[repr(C, align(4096))]
pub struct InitialPagetable(pub [usize; 512]);
