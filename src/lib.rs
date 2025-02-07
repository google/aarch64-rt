// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Startup code for aarch64 Cortex-A processors.

#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(any(
    all(feature = "el1", feature = "el2"),
    all(feature = "el1", feature = "el3"),
    all(feature = "el2", feature = "el3"),
))]
compile_error!("Only one `el` feature may be enabled at once.");

#[cfg(feature = "exceptions")]
use core::arch::asm;
use core::arch::global_asm;

global_asm!(include_str!("entry.S"));

#[cfg(not(feature = "initial-pagetable"))]
global_asm!(include_str!("dummy_enable_mmu.S"),);
#[cfg(all(feature = "el1", feature = "initial-pagetable"))]
global_asm!(include_str!("el1_enable_mmu.S"),);
#[cfg(all(feature = "el2", feature = "initial-pagetable"))]
global_asm!(include_str!("el2_enable_mmu.S"));
#[cfg(all(feature = "el3", feature = "initial-pagetable"))]
global_asm!(include_str!("el3_enable_mmu.S"));

#[cfg(feature = "exceptions")]
global_asm!(include_str!("exceptions.S"));

#[unsafe(no_mangle)]
extern "C" fn rust_entry(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
    // SAFETY: We provide a valid vector table.
    #[cfg(all(feature = "el1", feature = "exceptions"))]
    unsafe {
        asm!(
            "adr x30, vector_table",
            "msr vbar_el1, x30",
            options(nomem, nostack),
            out("x30") _,
        );
    }
    // SAFETY: We provide a valid vector table.
    #[cfg(all(feature = "el2", feature = "exceptions"))]
    unsafe {
        asm!(
            "adr x30, vector_table",
            "msr vbar_el2, x30",
            options(nomem, nostack),
            out("x30") _,
        );
    }
    // SAFETY: We provide a valid vector table.
    #[cfg(all(feature = "el3", feature = "exceptions"))]
    unsafe {
        asm!(
            "adr x30, vector_table",
            "msr vbar_el3, x30",
            options(nomem, nostack),
            out("x30") _,
        );
    }
    main(arg0, arg1, arg2, arg3)
}

unsafe extern "Rust" {
    /// Main function provided by the application using the `main!` macro.
    safe fn main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> !;
}

/// Marks the main function of the binary.
///
/// Example:
///
/// ```rust
/// use aarch64_rt::entry;
///
/// entry!(main);
/// fn main() -> ! {
///     log::set_max_level(LevelFilter::Info);
///     info!("Hello world");
/// }
/// ```
#[macro_export]
macro_rules! entry {
    ($name:path) => {
        // Export a symbol with a name matching the extern declaration above.
        #[export_name = "main"]
        fn __main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
            // Ensure that the main function provided by the application has the correct type.
            $name(arg0, arg1, arg2, arg3)
        }
    };
}

#[macro_export]
macro_rules! initial_idmap {
    ($value:expr) => {
        #[unsafe(export_name = "idmap")]
        #[unsafe(link_section = ".rodata.idmap")]
        static INITIAL_IDMAP: $crate::InitialIdmap = $value;
    };
}

/// A hardcoded pagetable.
#[repr(align(4096))]
#[allow(dead_code)]
pub struct InitialIdmap(pub [usize; 512]);
