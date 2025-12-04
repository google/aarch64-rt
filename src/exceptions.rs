// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use core::{borrow::Borrow, ops::Deref};

/// The register state saved before calling the exception handler.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct RegisterState {
    /// Registers x0-x18.
    pub registers: [u64; 19],
    padding: u64,
    /// Register x29, the Frame Pointer.
    pub fp: u64,
    /// Register x30, the Stack Pointer.
    pub sp: u64,
    pub elr: usize,
    pub spsr: u64,
}

const _: () = assert!(size_of::<RegisterState>() == 8 * 24);

/// A reference to the register state saved when an exception happened.
#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct RegisterStateRef<'a>(&'a mut RegisterState);

impl RegisterStateRef<'_> {
    /// Returns a mutable reference to the register state.
    ///
    /// # Safety
    ///
    /// Any changes made to the saved register state made via this reference must not cause
    /// undefined behaviour when returning from the exception.
    ///
    /// Exactly what this means depends on the exception and the context in which it happened, but
    /// for example changing the ELR to point to an invalid or unexpected location, or changing some
    /// general-purpose register value which the code doesn't expect to change could cause undefined
    /// behaviour.
    pub unsafe fn get_mut(&mut self) -> &mut RegisterState {
        self.0
    }
}

impl AsRef<RegisterState> for RegisterStateRef<'_> {
    fn as_ref(&self) -> &RegisterState {
        self.0
    }
}

impl Borrow<RegisterState> for RegisterStateRef<'_> {
    fn borrow(&self) -> &RegisterState {
        self.0
    }
}

impl Deref for RegisterStateRef<'_> {
    type Target = RegisterState;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Functions to handle aarch64 exceptions.
///
/// Each method has a default implementation which will panic.
pub trait ExceptionHandlers {
    /// Handles synchronous exceptions from the current exception level.
    extern "C" fn sync_current(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected synchronous exception from current EL");
    }

    /// Handles IRQs from the current exception level.
    extern "C" fn irq_current(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected IRQ from current EL");
    }

    /// Handles FIQs from the current exception level.
    extern "C" fn fiq_current(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected FIQ from current EL");
    }

    /// Handles SErrors from the current exception level.
    extern "C" fn serror_current(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected SError from current EL");
    }

    /// Handles synchronous exceptions from a lower exception level.
    extern "C" fn sync_lower(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected synchronous exception from lower EL");
    }

    /// Handles IRQs from the a lower exception level.
    extern "C" fn irq_lower(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected IRQ from lower EL");
    }

    /// Handles FIQs from the a lower exception level.
    extern "C" fn fiq_lower(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected FIQ from lower EL");
    }

    /// Handles SErrors from a lower exception level.
    extern "C" fn serror_lower(register_state: RegisterStateRef) {
        _ = register_state;
        panic!("Unexpected SError from lower EL");
    }
}

/// Registers an implementation of the [`ExceptionHandlers`] trait to handle exceptions.
#[macro_export]
macro_rules! exception_handlers {
    ($handlers:ty) => {
        core::arch::global_asm!(
            r#"
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
 * This is a generic handler for exceptions taken at the current EL. It saves
 * volatile registers to the stack, calls the Rust handler, restores volatile
 * registers, then returns.
 *
 * This also works for exceptions taken from lower ELs, if we don't care about
 * non-volatile registers.
 *
 * Saving state and jumping to the Rust handler takes 16 instructions, and
 * restoring and returning also takes 15 instructions, so we can fit the whole
 * handler in 31 instructions, under the limit of 32.
 */
.macro current_exception handler:req el:req
	save_volatile_to_stack \el
	mov x0, sp
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
	current_exception {sync_current} \el

.balign 0x80
irq_cur_sp0_\el:
	current_exception {irq_current} \el

.balign 0x80
fiq_cur_sp0_\el:
	current_exception {fiq_current} \el

.balign 0x80
serr_cur_sp0_\el:
	current_exception {serror_current} \el

.balign 0x80
sync_cur_spx_\el:
	current_exception {sync_current} \el

.balign 0x80
irq_cur_spx_\el:
	current_exception {irq_current} \el

.balign 0x80
fiq_cur_spx_\el:
	current_exception {fiq_current} \el

.balign 0x80
serr_cur_spx_\el:
	current_exception {serror_current} \el

.balign 0x80
sync_lower_64_\el:
	current_exception {sync_lower} \el

.balign 0x80
irq_lower_64_\el:
	current_exception {irq_lower} \el

.balign 0x80
fiq_lower_64_\el:
	current_exception {fiq_lower} \el

.balign 0x80
serr_lower_64_\el:
	current_exception {serror_lower} \el

.balign 0x80
sync_lower_32_\el:
	current_exception {sync_lower} \el

.balign 0x80
irq_lower_32_\el:
	current_exception {irq_lower} \el

.balign 0x80
fiq_lower_32_\el:
	current_exception {fiq_lower} \el

.balign 0x80
serr_lower_32_\el:
	current_exception {serror_lower} \el

.endm

vector_table el1
vector_table el2
vector_table el3
            "#,
            sync_current = sym <$handlers as $crate::ExceptionHandlers>::sync_current,
            irq_current = sym <$handlers as $crate::ExceptionHandlers>::irq_current,
            fiq_current = sym <$handlers as $crate::ExceptionHandlers>::fiq_current,
            serror_current = sym <$handlers as $crate::ExceptionHandlers>::serror_current,
            sync_lower = sym <$handlers as $crate::ExceptionHandlers>::sync_lower,
            irq_lower = sym <$handlers as $crate::ExceptionHandlers>::irq_lower,
            fiq_lower = sym <$handlers as $crate::ExceptionHandlers>::fiq_lower,
            serror_lower = sym <$handlers as $crate::ExceptionHandlers>::serror_lower,
        );
    };
}
