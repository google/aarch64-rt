// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Code to set up an initial pagetable.

use core::arch::global_asm;

#[cfg(feature = "el1")]
global_asm!(include_str!("el1_enable_mmu.S"));
#[cfg(feature = "el2")]
global_asm!(include_str!("el2_enable_mmu.S"));
#[cfg(feature = "el3")]
global_asm!(include_str!("el3_enable_mmu.S"));

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
