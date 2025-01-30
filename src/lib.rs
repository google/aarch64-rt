// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Startup code for aarch64 Cortex-A processors.

#![no_std]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(any(feature = "el1", feature = "exceptions"))]
use core::arch::global_asm;

#[cfg(feature = "el1")]
global_asm!(include_str!("el1_entry.S"));

#[cfg(feature = "exceptions")]
global_asm!(include_str!("exceptions.S"));

#[unsafe(no_mangle)]
extern "C" fn rust_entry(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
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
