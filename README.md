# Startup code for bare-metal aarch64

[![crates.io page](https://img.shields.io/crates/v/aarch64-rt.svg)](https://crates.io/crates/aarch64-rt)
[![docs.rs page](https://docs.rs/aarch64-rt/badge.svg)](https://docs.rs/aarch64-rt)

This crate provides entry point and exception handling for bare-metal Rust binaries on aarch64
Cortex-A processors.

This is not an officially supported Google product.

## Usage

Use the `entry!` macro to mark your main function:

```rust
use aarch64_rt::entry;

entry!(main);
fn main(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> ! {
    // ...
}
```

`arg0` through `arg3` will contain the initial values of registers `x0`–`x3`. These are often used
to pass arguments from the previous-stage bootloader, such as the address of the device tree.

## Features

`el1`, `exceptions` and `initial-pagetable` are enabled by default.

### `el1`

If the `exceptions` feature is also enabled then uses `vbar_el1` for the exception vector. If
`initial-pagetable` is also enabled then uses `ttbr0_el1` for the page table, and other EL1 MMU
configuration registers.

### `el2`

If the `exceptions` feature is also enabled then uses `vbar_el2` for the exception vector. If
`initial-pagetable` is also enabled then uses `ttbr0_el2` for the page table, and other EL2 MMU
configuration registers.

### `el3`

If the `exceptions` feature is also enabled then uses `vbar_el3` for the exception vector. If
`initial-pagetable` is also enabled then uses `ttbr0_el3` for the page table, and other EL3 MMU
configuration registers.

### `exceptions`

Provides an exception vector table, and sets it in the appropriate `vbar` system register for the
selected exception level. You must provide handlers for each exception like so:

```rust
#[unsafe(no_mangle)]
extern "C" fn sync_exception_current(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn irq_current(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn fiq_current(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn serr_current(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn sync_lower(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn irq_lower(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn fiq_lower(_elr: u64, _spsr: u64) {
}

#[unsafe(no_mangle)]
extern "C" fn serr_lower(_elr: u64, _spsr: u64) {
}
```

### `initial-pagetable`

Sets an initial pagetable in the appropriate TTBR and enables the MMU and cache before running any
Rust code or writing to any memory.

This is especially important if running at EL1 in a VM, as accessing memory with the cache disabled
while the hypervisor or host has cacheable aliases to the same memory can lead to cache coherency
issues. Even if the host doesn't explicitly access the memory, speculative accesses can lead to
cache fills.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

If you want to contribute to the project, see details of
[how we accept contributions](CONTRIBUTING.md).
