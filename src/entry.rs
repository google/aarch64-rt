// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Entrypoint code

use core::arch::naked_asm;

/// This is a generic entry point for an image. It carries out the operations required to prepare the
/// loaded image to be run. Specifically, it zeroes the bss section using registers x25 and above,
/// prepares the stack, enables floating point, and sets up the exception vector. It preserves x0-x3
/// for the Rust entry point, as these may contain boot parameters.
///
/// # Safety
///
/// This function is marked unsafe because it should never be called by anyone. The linker is
/// responsible for setting it as the entry function.
#[unsafe(naked)]
#[unsafe(link_section = ".init.entry")]
#[unsafe(export_name = "entry")]
unsafe extern "C" fn entry() -> ! {
    naked_asm!(
        ".macro adr_l, reg:req, sym:req",
        r"adrp \reg, \sym",
        r"add \reg, \reg, :lo12:\sym",
        ".endm",
        "bl enable_mmu",
        // Disable trapping floating point access in EL1.
        "mrs x30, cpacr_el1",
        "orr x30, x30, #(0x3 << 20)",
        "msr cpacr_el1, x30",
        "isb",
        // Zero out the bss section.
        "adr_l x29, bss_begin",
        "adr_l x30, bss_end",
        "0:",
        "cmp x29, x30",
        "b.hs 1f",
        "stp xzr, xzr, [x29], #16",
        "b 0b",
        "1:",
        // Prepare the stack.
        "adr_l x30, boot_stack_end",
        "mov sp, x30",
        // Call into Rust code.
        "b {rust_entry}",
        rust_entry = sym crate::rust_entry,
    )
}

/// An assembly entry point for secondary cores.
///
/// It will enable the MMU, disable trapping of floating point instructions, initialise the
/// stack pointer to `stack_end` and then jump to the function pointer at the bottom of the
/// stack with the u64 value second on the stack as a parameter.
///
/// # Safety
///
/// This requires that an initial stack pointer value be passed in `x0`, and the stack must contain
/// the address of a Rust entry point to jump to and a parameter value to pass to it.
#[unsafe(naked)]
pub unsafe extern "C" fn secondary_entry(stack_end: *mut u64) -> ! {
    naked_asm!(
        "bl enable_mmu",
        // Disable trapping floating point access in EL1.
        "mrs x30, cpacr_el1",
        "orr x30, x30, #(0x3 << 20)",
        "msr cpacr_el1, x30",
        "isb",
        // Set the stack pointer which was passed.
        "mov sp, x0",
        // Load trampoline address.
        "ldr x20, [sp]",
        // Set the exception vector.
        "bl {set_exception_vector}",
        // Call into Rust code. x0 already contains the argument.
        "br x20",
        set_exception_vector = sym crate::set_exception_vector,
    )
}
