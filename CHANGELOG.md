# Changelog

## Unreleased

### Improvements

- Made `set_exception_vector` public.

## 0.4.0

### Breaking changes

- Added `ExceptionHandlers` trait and `exception_handlers!` macro to define exception handlers,
  rather than relying on global symbols.
- Added `RegisterStateRef` parameter to exception handlers to access the saved register state, and
  removed ELR and SPSR parameters.
- `start_core` now takes an `FnOnce` rather than an `fn` and an argument. This allows arbitrary
  context to be captured in the closure.

## 0.3.1

### Bugfixes

- Fixed bug with four-argument version of `initial_pagetable!` swapping TCR and SCTLR values.

## 0.3.0

### Breaking changes

- Changed order of parameters to `initial-pagetable` macro, to make `TCR` last.

### Improvements

- If the `initial-pagetable` or `exceptions` features are specified without any of the `elX`
  features, then the exception level will be checked at runtime and the appropriate registers for
  the current EL will be used. The `el1` feature is no longer enabled by default, as this runtime
  detection should work instead. Note that different ELs have different TCR registers which aren't
  laid out entirely the same, so different values must be specified for TCR for each EL.
- Exposed `enable_mmu!` macro to allow the MMU and caches to be enbled with an arbitrary initial
  pagetable, rather than using `initial_pagetable!` to declare the static.

### Bugfixes

- Stopped exposing unmangled symbols for `set_exception_vector` and `rust_entry`.

## 0.2.2

### Improvements

- Added optional parameters to `initial_pagetable!` to allow initial MAIR, TCR and SCTLR values to
  be specified. The default values are exposed as constants.

## 0.2.1

### Bugfixes

- Fixed build failure when `psci` feature was enabled without `exceptions` feature.

## 0.2.0

### Breaking changes

- `vector_table` renamed to `vector_table_el1`.
- `start_core` now takes a type parameter to choose whether to use an HVC or SMC PSCI call.

### Bugfixes

- Save and restore correct ELR and SPSR registers when handling exceptions in EL2 or EL3. New vector
  tables `vector_table_el2` and `vector_table_el3` are provided for these.

## 0.1.3

### Improvements

- Set exception vector on secondary cores too.

## 0.1.2

### Bugfixes

- Renamed internal `main` symbol to `__main` to avoid clashes with symbols from the application.

### Improvements

- Added secondary core entry point.
- Added `start_core` function to wrap a `PSCI_CPU_ON` call to start a secondary core, with the
  secondary core entry point. This is behind the new `psci` feature, which is enabled by default.

## 0.1.1

### Improvements

- Made boot stack size configurable.

## 0.1.0

Initial release.
