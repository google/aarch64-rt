/*
 * Copyright 2025 The aarch64-rt Authors.
 *
 * This project is dual-licensed under Apache 2.0 and MIT terms.
 * See LICENSE-APACHE and LICENSE-MIT for details.
 */

/**
 * Saves the volatile registers onto the stack. This currently takes 14
 * instructions, so it can be used in exception handlers with 18 instructions
 * left.
 *
 * On return, x0 and x1 are initialised to elr_elX and spsr_elX respectively,
 * which can be used as the first and second arguments of a subsequent call.
 */
.macro save_volatile_to_stack el:req
	/* Reserve stack space and save registers x0-x18, x29 & x30. */
	stp x0, x1, [sp, #-(8 * 24)]!
	stp x2, x3, [sp, #8 * 2]
	stp x4, x5, [sp, #8 * 4]
	stp x6, x7, [sp, #8 * 6]
	stp x8, x9, [sp, #8 * 8]
	stp x10, x11, [sp, #8 * 10]
	stp x12, x13, [sp, #8 * 12]
	stp x14, x15, [sp, #8 * 14]
	stp x16, x17, [sp, #8 * 16]
	str x18, [sp, #8 * 18]
	stp x29, x30, [sp, #8 * 20]

	/*
	 * Save elr_elX & spsr_elX. This such that we can take nested exception
	 * and still be able to unwind.
	 */
	mrs x0, elr_\el
	mrs x1, spsr_\el
	stp x0, x1, [sp, #8 * 22]
.endm

/**
 * Restores the volatile registers from the stack. This currently takes 14
 * instructions, so it can be used in exception handlers while still leaving 18
 * instructions left; if paired with save_volatile_to_stack, there are 4
 * instructions to spare.
 */
.macro restore_volatile_from_stack el:req
	/* Restore registers x2-x18, x29 & x30. */
	ldp x2, x3, [sp, #8 * 2]
	ldp x4, x5, [sp, #8 * 4]
	ldp x6, x7, [sp, #8 * 6]
	ldp x8, x9, [sp, #8 * 8]
	ldp x10, x11, [sp, #8 * 10]
	ldp x12, x13, [sp, #8 * 12]
	ldp x14, x15, [sp, #8 * 14]
	ldp x16, x17, [sp, #8 * 16]
	ldr x18, [sp, #8 * 18]
	ldp x29, x30, [sp, #8 * 20]

	/* Restore registers elr_elX & spsr_elX, using x0 & x1 as scratch. */
	ldp x0, x1, [sp, #8 * 22]
	msr elr_\el, x0
	msr spsr_\el, x1

	/* Restore x0 & x1, and release stack space. */
	ldp x0, x1, [sp], #8 * 24
.endm

/**
 * This is a generic handler for exceptions taken at the current EL while using
 * SP0. It behaves similarly to the SPx case by first switching to SPx, doing
 * the work, then switching back to SP0 before returning.
 *
 * Switching to SPx and calling the Rust handler takes 16 instructions. To
 * restore and return we need an additional 16 instructions, so we can implement
 * the whole handler within the allotted 32 instructions.
 */
.macro current_exception_sp0 handler:req el:req
	msr spsel, #1
	save_volatile_to_stack \el
	bl \handler
	restore_volatile_from_stack \el
	msr spsel, #0
	eret
.endm

/**
 * This is a generic handler for exceptions taken at the current EL while using
 * SPx. It saves volatile registers, calls the Rust handler, restores volatile
 * registers, then returns.
 *
 * This also works for exceptions taken from EL0, if we don't care about
 * non-volatile registers.
 *
 * Saving state and jumping to the Rust handler takes 15 instructions, and
 * restoring and returning also takes 15 instructions, so we can fit the whole
 * handler in 30 instructions, under the limit of 32.
 */
.macro current_exception_spx handler:req el:req
	save_volatile_to_stack \el
	bl \handler
	restore_volatile_from_stack \el
	eret
.endm


.macro vector_table el:req
.section .text.vector_table_\el, "ax"
.global vector_table_\el
.balign 0x800
vector_table_\el:
sync_cur_sp0_\el:
	current_exception_sp0 sync_exception_current \el

.balign 0x80
irq_cur_sp0_\el:
	current_exception_sp0 irq_current \el

.balign 0x80
fiq_cur_sp0_\el:
	current_exception_sp0 fiq_current \el

.balign 0x80
serr_cur_sp0_\el:
	current_exception_sp0 serr_current \el

.balign 0x80
sync_cur_spx_\el:
	current_exception_spx sync_exception_current \el

.balign 0x80
irq_cur_spx_\el:
	current_exception_spx irq_current \el

.balign 0x80
fiq_cur_spx_\el:
	current_exception_spx fiq_current \el

.balign 0x80
serr_cur_spx_\el:
	current_exception_spx serr_current \el

.balign 0x80
sync_lower_64_\el:
	current_exception_spx sync_lower \el

.balign 0x80
irq_lower_64_\el:
	current_exception_spx irq_lower \el

.balign 0x80
fiq_lower_64_\el:
	current_exception_spx fiq_lower \el

.balign 0x80
serr_lower_64_\el:
	current_exception_spx serr_lower \el

.balign 0x80
sync_lower_32_\el:
	current_exception_spx sync_lower \el

.balign 0x80
irq_lower_32_\el:
	current_exception_spx irq_lower \el

.balign 0x80
fiq_lower_32_\el:
	current_exception_spx fiq_lower \el

.balign 0x80
serr_lower_32_\el:
	current_exception_spx serr_lower \el

.endm

vector_table el1
vector_table el2
vector_table el3
