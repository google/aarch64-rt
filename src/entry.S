/*
 * Copyright 2025 The aarch64-rt Authors.
 *
 * This project is dual-licensed under Apache 2.0 and MIT terms.
 * See LICENSE-APACHE and LICENSE-MIT for details.
 */

.macro adr_l, reg:req, sym:req
    adrp \reg, \sym
    add \reg, \reg, :lo12:\sym
.endm

/**
 * This is a generic entry point for an image. It carries out the operations required to prepare the
 * loaded image to be run. Specifically, it zeroes the bss section using registers x25 and above,
 * prepares the stack, enables floating point, and sets up the exception vector. It preserves x0-x3
 * for the Rust entry point, as these may contain boot parameters.
 */
.section .init.entry, "ax"
.global entry
entry:
    bl enable_mmu

    /* Disable trapping floating point access in EL1. */
    mrs x30, cpacr_el1
    orr x30, x30, #(0x3 << 20)
    msr cpacr_el1, x30
    isb

    /* Zero out the bss section. */
    adr_l x29, bss_begin
    adr_l x30, bss_end
0:  cmp x29, x30
    b.hs 1f
    stp xzr, xzr, [x29], #16
    b 0b

1:  /* Prepare the stack. */
    adr_l x30, boot_stack_end
    mov sp, x30

    /* Call into Rust code. */
    b rust_entry
