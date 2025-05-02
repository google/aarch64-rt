# Changelog

## Unreleased

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
