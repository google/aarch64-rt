// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Entrypoint code

use core::arch::naked_asm;

use crate::rust_entry;

/// This is a generic entry point for an image that calls [`entry_early_prepare`].
///
/// # Safety
///
/// This function is marked unsafe because it should never be called by anyone. The linker is
/// responsible for setting it as the entry function.
#[cfg(not(feature = "relocate"))]
#[unsafe(naked)]
#[unsafe(link_section = ".init.entry")]
#[unsafe(export_name = "entry")]
unsafe extern "C" fn entry() -> ! {
    naked_asm!(
        "b {entry_early_prepare}",
        entry_early_prepare = sym entry_early_prepare
    )
}

/// This is a generic entry point for an image prefixed with an [AArch64 Linux kernel boot
/// header](https://docs.kernel.org/arch/arm64/booting.html) that calls [`entry_early_prepare`].
///
/// # Safety
///
/// This function is marked unsafe because it should never be called by anyone. The linker is
/// responsible for setting it as the entry function.
#[cfg(feature = "relocate")]
#[unsafe(naked)]
#[unsafe(link_section = ".init.entry")]
#[unsafe(export_name = "entry")]
unsafe extern "C" fn entry() -> ! {
    const HEADER_FLAG_ENDIANNESS: u64 = cfg!(target_endian = "big") as u64;
    // 0 - Unspecified, 1 - 4K, 2 - 16K, 3 - 64K
    const HEADER_FLAG_PAGE_SIZE: u64 = 1;
    const HEADER_FLAG_PHYSICAL_PLACEMENT: u64 = 1;
    const HEADER_FLAGS: u64 = HEADER_FLAG_ENDIANNESS
        | (HEADER_FLAG_PAGE_SIZE << 1)
        | (HEADER_FLAG_PHYSICAL_PLACEMENT << 3);

    naked_asm!(
        // code0
        "b {entry_early_prepare}",
        // code1
        "nop",

        // text_offset
        ".quad 0x0",
        // image_size
        ".quad bin_end - entry",
        // flags
        ".quad {HEADER_FLAGS}",
        // res2
        ".quad 0",
        // res3
        ".quad 0",
        // res4
        ".quad 0",

        // "ARM\x64" magic number
        ".long 0x644d5241",
        // res5
        ".long 0",
        ".align 3",
        entry_early_prepare = sym entry_early_prepare,
        HEADER_FLAGS = const HEADER_FLAGS.to_le(),
    )
}

/// Early entry point preparations.
///
/// It carries out the operations required to prepare the loaded image to be run. Specifically, it
/// zeroes the bss section using registers x25 and above, prepares the stack, enables floating
/// point, and sets up the exception vector. It preserves x0-x3 for the Rust entry point, as these
/// may contain boot parameters.
#[unsafe(naked)]
unsafe extern "C" fn entry_early_prepare() -> ! {
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
        // Perform final Rust entrypoint setup
        "b {entry_prepare_image}",
        ".purgem adr_l",
        entry_prepare_image = sym entry_prepare_image
    )
}

#[cfg(not(feature = "relocate"))]
#[unsafe(naked)]
unsafe extern "C" fn entry_prepare_image() -> ! {
    naked_asm!(
        // Call into Rust code.
        "b {rust_entry}",
        rust_entry = sym rust_entry
    )
}

#[cfg(feature = "relocate")]
#[unsafe(naked)]
unsafe extern "C" fn entry_prepare_image() -> ! {
    naked_asm!(
        ".macro adr_l, reg:req, sym:req",
        r"adrp \reg, \sym",
        r"add \reg, \reg, :lo12:\sym",
        ".endm",
        // Where the image was loaded
        "adr_l x7, text_begin",
        // Start relocating.
        // let mut rela: *mut Elf64Rela = __rela_start;
        "adr_l x9, __rela_start",
        // let rela_end: *mut Elf64Rela = __rela_end;
        "adr_l x8, __rela_end",
        "b 1f",
        "2:",
        // rela = rela.wrapping_offset(1);
        "add x9, x9, #24",
        "1:",
        // while rela < rela_end {
        "cmp x9, x8",
        "b.hs 3f",
        // let r_type = unsafe { *rela }.r_info & 0xffff_ffff;
        "ldr w10, [x9, #8]",
        // if r_type != R_AARCH64_RELATIVE { continue };
        "cmp w10, #1027",
        "b.ne 2b",
        // let mut new_ptr = unsafe { *rela }.r_addend;
        "ldr x10, [x9, #16]",
        // let r_offset = unsafe { *rela }.r_offset;
        "ldr x11, [x9]",
        //  new_ptr += offset;
        "add x10, x10, x7",
        // unsafe { *(offset + r_offset) } = new_ptr;
        "str x10, [x7, x11]",
        "b 2b",
        "3:",
        // End of relocations.
        // Call into Rust code.
        "b {rust_entry}",
        ".purgem adr_l",
        rust_entry = sym rust_entry,
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
        // Load Rust entry point address and argument from the bottom of the stack into
        // callee-saved registers.
        "ldp x19, x20, [sp, #-16]",
        // Set the exception vector.
        "bl {set_exception_vector}",
        // Pass argument to Rust entry point.
        "mov x0, x19",
        // Call into Rust code.
        "br x20",
        set_exception_vector = sym crate::set_exception_vector,
    )
}
