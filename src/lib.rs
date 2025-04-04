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

/// Marks the main function of the binary and reserves space for the boot stack.
///
/// Example:
///
/// ```rust
/// use aarch64_rt::entry;
///
/// entry!(main);
/// fn main() -> ! {
///     info!("Hello world");
/// }
/// ```
///
/// 40 pages (160 KiB) is reserved for the boot stack by default; a different size may be configured
/// by passing the number of pages as a second argument to the macro, e.g. `entry!(main, 10);` to
/// reserve only 10 pages.
#[macro_export]
macro_rules! entry {
    ($name:path) => {
        entry!($name, 40);
    };
    ($name:path, $boot_stack_pages:expr) => {
        #[unsafe(export_name = "boot_stack")]
        #[unsafe(link_section = ".stack.boot_stack")]
        static mut __BOOT_STACK: $crate::Stack<$boot_stack_pages> = $crate::Stack::new();

        // Export a symbol with a name matching the extern declaration above.
        #[export_name = "main"]
        fn __main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
            // Ensure that the main function provided by the application has the correct type.
            $name(arg0, arg1, arg2, arg3)
        }
    };
}

/// Provides an initial pagetable which can be used before any Rust code is run.
///
/// The `initial-pagetable` feature must be enabled for this to be used.
#[cfg(feature = "initial-pagetable")]
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

/// A stack for some CPU core.
///
/// This is used by the [`entry!`] macro to reserve space for the boot stack.
#[repr(C, align(4096))]
pub struct Stack<const NUM_PAGES: usize>([StackPage; NUM_PAGES]);

impl<const NUM_PAGES: usize> Stack<NUM_PAGES> {
    /// Creates a new zero-initialised stack.
    pub const fn new() -> Self {
        Self([const { StackPage::new() }; NUM_PAGES])
    }
}

impl<const NUM_PAGES: usize> Default for Stack<NUM_PAGES> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C, align(4096))]
struct StackPage([u8; 4096]);

impl StackPage {
    const fn new() -> Self {
        Self([0; 4096])
    }
}
