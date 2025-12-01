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

mod entry;
#[cfg(feature = "exceptions")]
mod exceptions;
#[cfg(feature = "initial-pagetable")]
mod pagetable;

#[cfg(feature = "initial-pagetable")]
#[doc(hidden)]
pub mod __private {
    pub use crate::pagetable::{__enable_mmu_el1, __enable_mmu_el2, __enable_mmu_el3};
}

#[cfg(any(feature = "exceptions", feature = "psci"))]
use core::arch::asm;
#[cfg(not(feature = "initial-pagetable"))]
use core::arch::naked_asm;
pub use entry::secondary_entry;
#[cfg(feature = "exceptions")]
pub use exceptions::{ExceptionHandlers, RegisterState, RegisterStateRef};
#[cfg(all(feature = "initial-pagetable", feature = "el1"))]
pub use pagetable::DEFAULT_TCR_EL1 as DEFAULT_TCR;
#[cfg(all(feature = "initial-pagetable", feature = "el2"))]
pub use pagetable::DEFAULT_TCR_EL2 as DEFAULT_TCR;
#[cfg(all(feature = "initial-pagetable", feature = "el3"))]
pub use pagetable::DEFAULT_TCR_EL3 as DEFAULT_TCR;
#[cfg(feature = "initial-pagetable")]
pub use pagetable::{
    DEFAULT_MAIR, DEFAULT_SCTLR, DEFAULT_TCR_EL1, DEFAULT_TCR_EL2, DEFAULT_TCR_EL3,
    InitialPagetable,
};

#[cfg(feature = "psci")]
use core::mem::ManuallyDrop;

#[cfg(not(feature = "initial-pagetable"))]
#[unsafe(naked)]
#[unsafe(link_section = ".init")]
#[unsafe(export_name = "enable_mmu")]
extern "C" fn enable_mmu() {
    naked_asm!("ret")
}

/// Sets the appropriate vbar to point to our `vector_table`, if the `exceptions` feature is
/// enabled.
extern "C" fn set_exception_vector() {
    // SAFETY: We provide a valid vector table.
    #[cfg(all(feature = "el1", feature = "exceptions"))]
    unsafe {
        asm!(
            "adr x9, vector_table_el1",
            "msr vbar_el1, x9",
            options(nomem, nostack),
            out("x9") _,
        );
    }
    // SAFETY: We provide a valid vector table.
    #[cfg(all(feature = "el2", feature = "exceptions"))]
    unsafe {
        asm!(
            "adr x9, vector_table_el2",
            "msr vbar_el2, x9",
            options(nomem, nostack),
            out("x9") _,
        );
    }
    // SAFETY: We provide a valid vector table.
    #[cfg(all(feature = "el3", feature = "exceptions"))]
    unsafe {
        asm!(
            "adr x9, vector_table_el3",
            "msr vbar_el3, x9",
            options(nomem, nostack),
            out("x9") _,
        );
    }
    #[cfg(all(
        feature = "exceptions",
        not(any(feature = "el1", feature = "el2", feature = "el3"))
    ))]
    {
        let current_el: u64;
        // SAFETY: Reading CurrentEL is always safe.
        unsafe {
            asm!(
                "mrs {current_el}, CurrentEL",
                options(nomem, nostack, preserves_flags),
                current_el = out(reg) current_el,
            );
        }
        match (current_el >> 2) & 0b11 {
            // SAFETY: We provide a valid vector table.
            1 => unsafe {
                asm!(
                    "adr x9, vector_table_el1",
                    "msr vbar_el1, x9",
                    options(nomem, nostack, preserves_flags),
                    out("x9") _,
                );
            },
            // SAFETY: We provide a valid vector table.
            2 => unsafe {
                asm!(
                    "adr x9, vector_table_el2",
                    "msr vbar_el2, x9",
                    options(nomem, nostack, preserves_flags),
                    out("x9") _,
                );
            },
            // SAFETY: We provide a valid vector table.
            3 => unsafe {
                asm!(
                    "adr x9, vector_table_el3",
                    "msr vbar_el3, x9",
                    options(nomem, nostack, preserves_flags),
                    out("x9") _,
                );
            },
            _ => {
                panic!("Unexpected EL");
            }
        }
    }
}

extern "C" fn rust_entry(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
    set_exception_vector();
    __main(arg0, arg1, arg2, arg3)
}

unsafe extern "Rust" {
    /// Main function provided by the application using the `main!` macro.
    safe fn __main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> !;
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
        #[unsafe(export_name = "__main")]
        fn __main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
            // Ensure that the main function provided by the application has the correct type.
            $name(arg0, arg1, arg2, arg3)
        }
    };
}

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

#[cfg(feature = "psci")]
/// Issues a PSCI CPU_ON call to start the CPU core with the given MPIDR.
///
/// This starts the core with an assembly entry point which will enable the MMU, disable trapping of
/// floating point instructions, initialise the stack pointer to the given value, and then jump to
/// the given Rust entry point function, passing it the given argument value.
///
/// The closure passed as `rust_entry` **should never return**. Because the
/// [never type has not been stabilized](https://github.com/rust-lang/rust/issues/35121)), this
/// cannot be enforced by the type system yet.
///
/// # Safety
///
/// `stack` must point to a region of memory which is reserved for this core's stack. It must remain
/// valid as long as the core is running, and there must not be any other access to it during that
/// time. It must be mapped both for the current core to write to it (to pass initial parameters)
/// and in the initial page table which the core being started will used, with the same memory
/// attributes for both.
// TODO: change `F` generic bounds to `FnOnce() -> !` when the never type is stabilized:
// https://github.com/rust-lang/rust/issues/35121
pub unsafe fn start_core<C: smccc::Call, F: FnOnce() + Send + 'static, const N: usize>(
    mpidr: u64,
    stack: *mut Stack<N>,
    rust_entry: F,
) -> Result<(), smccc::psci::Error> {
    #[expect(
        unused,
        reason = "the fields are read in the `secondary_entry` assembly code"
    )]
    struct StartCoreStack<F> {
        trampoline_ptr: unsafe extern "C" fn(&mut ManuallyDrop<F>) -> !,
        entry_ptr: *mut ManuallyDrop<F>,
    }

    const {
        assert!(
            core::mem::size_of::<StartCoreStack<F>>()
                + 2 * core::mem::size_of::<F>()
                + 2 * core::mem::align_of::<StartCoreStack<F>>()
                + 1024 // trampoline stack frame overhead
                <= core::mem::size_of::<Stack<N>>(),
            "the `rust_entry` closure is too big to fit in the core stack"
        );
    }

    let rust_entry = ManuallyDrop::new(rust_entry);

    let stack_bottom = stack.cast::<u8>();
    let align_offfset = stack.align_offset(core::mem::align_of::<F>());
    let entry_ptr = stack_bottom
        .wrapping_add(align_offfset)
        .cast::<ManuallyDrop<F>>();

    assert!(stack.is_aligned());
    let stack_end = stack.wrapping_add(1) as *mut StartCoreStack<F>;

    // Write the trampoline and entry closure, so the assembly entry point can jump to it.
    // SAFETY: Our caller promised that the stack is valid and nothing else will access it.
    unsafe {
        entry_ptr.write(rust_entry);
        *stack_end.wrapping_sub(1) = StartCoreStack {
            trampoline_ptr: trampoline::<F>,
            entry_ptr,
        };
    };

    // Wait for the stores above to complete before starting the secondary CPU core.
    dsb_st();

    smccc::psci::cpu_on::<C>(
        mpidr,
        secondary_entry as usize as _,
        stack_end as usize as _,
    )
}

#[cfg(feature = "psci")]
/// Used by [`start_core`] as an entry point for the secondary CPU core.
///
/// # Safety
///
/// This calls [`ManuallyDrop::take`] on the provided argument, so this function must be
/// called at most once for a given instance of `F`.
// TODO: change `F` generic bounds to `FnOnce() -> !` when the never type is stabilized:
// https://github.com/rust-lang/rust/issues/35121
unsafe extern "C" fn trampoline<F: FnOnce() + Send + 'static>(entry: &mut ManuallyDrop<F>) -> ! {
    // SAFETY: the trampoline function is only ever called once after creating ManuallyDrop
    // instance, so we won't call ManuallyDrop::take more than once.
    let entry = unsafe { ManuallyDrop::take(entry) };
    entry();

    panic!("rust_entry function passed to start_core should never return");
}

/// Data synchronisation barrier that waits for stores to complete, for the full system.
#[cfg(feature = "psci")]
fn dsb_st() {
    // SAFETY: A synchronisation barrier is always safe.
    unsafe {
        asm!("dsb st", options(nostack));
    }
}
