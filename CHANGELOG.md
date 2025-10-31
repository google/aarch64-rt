# Changelog

## Unreleased

### Improvements

- If the `initial-pagetable` feature is specified without any of the `elX` features, then the
  exception level will be checked at runtime and the appropriate registers for the current EL will
  be used.

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
