// Copyright 2025 The aarch64-rt Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

//! Example to run at EL1 on QEMU's virt board.

#![no_std]
#![no_main]

use aarch64_paging::{
    mair::{Mair, MairAttribute, NormalMemory},
    paging::Attributes,
};
use aarch64_rt::{InitialPagetable, entry, initial_pagetable};
use arm_pl011_uart::{PL011Registers, Uart, UniqueMmioPointer};
use core::{fmt::Write, panic::PanicInfo, ptr::NonNull};
use smccc::{
    Hvc,
    psci::{system_off, system_reset},
};

/// Base address of the first PL011 UART.
const PL011_BASE_ADDRESS: *mut PL011Registers = 0x900_0000 as _;

/// Attributes to use for device memory in the initial identity map.
const DEVICE_ATTRIBUTES: Attributes = Attributes::VALID
    .union(Attributes::ATTRIBUTE_INDEX_0)
    .union(Attributes::ACCESSED)
    .union(Attributes::UXN);

/// Attributes to use for normal memory in the initial identity map.
const MEMORY_ATTRIBUTES: Attributes = Attributes::VALID
    .union(Attributes::ATTRIBUTE_INDEX_1)
    .union(Attributes::INNER_SHAREABLE)
    .union(Attributes::ACCESSED)
    .union(Attributes::NON_GLOBAL);

/// Indirect memory attributes to use.
///
/// These are used for `ATTRIBUTE_INDEX_0` and `ATTRIBUTE_INDEX_1` in `DEVICE_ATTRIBUTES` and
/// `MEMORY_ATTRIBUTES` respectively.
const MAIR: Mair = Mair::EMPTY
    .with_attribute(0, MairAttribute::DEVICE_NGNRE)
    .with_attribute(
        1,
        MairAttribute::normal(
            NormalMemory::WriteBackNonTransientReadWriteAllocate,
            NormalMemory::WriteBackNonTransientReadWriteAllocate,
        ),
    );

initial_pagetable!(
    {
        let mut idmap = [0; 512];
        // 1 GiB of device memory.
        idmap[0] = DEVICE_ATTRIBUTES.bits();
        // 1 GiB of normal memory.
        idmap[1] = MEMORY_ATTRIBUTES.bits() | 0x40000000;
        // Another 1 GiB of device memory starting at 256 GiB.
        idmap[256] = DEVICE_ATTRIBUTES.bits() | 0x4000000000;
        InitialPagetable(idmap)
    },
    MAIR.0
);

entry!(main);
fn main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
    // SAFETY: The PL011 base address is mapped by the initial identity mapping, and this is the
    // only place we create something referring to it.
    let mut uart =
        Uart::new(unsafe { UniqueMmioPointer::new(NonNull::new(PL011_BASE_ADDRESS).unwrap()) });

    writeln!(
        uart,
        "main({:#x}, {:#x}, {:#x}, {:#x})",
        arg0, arg1, arg2, arg3
    )
    .unwrap();

    system_off::<Hvc>().unwrap();
    panic!("system_off returned");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    system_reset::<Hvc>().unwrap();
    #[allow(clippy::empty_loop)]
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn sync_exception_current(_elr: u64, _spsr: u64) {
    panic!("Unexpected sync_exception_current");
}

#[unsafe(no_mangle)]
extern "C" fn irq_current(_elr: u64, _spsr: u64) {
    panic!("Unexpected irq_current");
}

#[unsafe(no_mangle)]
extern "C" fn fiq_current(_elr: u64, _spsr: u64) {
    panic!("Unexpected fiq_current");
}

#[unsafe(no_mangle)]
extern "C" fn serr_current(_elr: u64, _spsr: u64) {
    panic!("Unexpected serr_current");
}

#[unsafe(no_mangle)]
extern "C" fn sync_lower(_elr: u64, _spsr: u64) {
    panic!("Unexpected sync_lower");
}

#[unsafe(no_mangle)]
extern "C" fn irq_lower(_elr: u64, _spsr: u64) {
    panic!("Unexpected irq_lower");
}

#[unsafe(no_mangle)]
extern "C" fn fiq_lower(_elr: u64, _spsr: u64) {
    panic!("Unexpected fiq_lower");
}

#[unsafe(no_mangle)]
extern "C" fn serr_lower(_elr: u64, _spsr: u64) {
    panic!("Unexpected serr_lower");
}
