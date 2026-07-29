# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-07-29

### Added

- Precompiled templates: `Template::parse` parses once and formats many
  times. Literal segments borrow from the template (zero-copy), and
  `arity()` reports the minimum argument count — counting both
  placeholder indices and `n$` width/precision references.
- Formatting into any `core::fmt::Write` sink (appending, not
  overwriting): `Format::try_write_to` / `write_to`,
  `Template::try_write_to` / `write_to`, and
  `FormatBuilder::try_build_to` / `build_to`.
- `FormatError::WriteFailed`, returned when a sink rejects a write.
- GitHub Actions CI: test matrix (Ubuntu/Windows/macOS × MSRV/stable/beta),
  a bare-metal `thumbv6m-none-eabi` build proving `no_std` support, and
  clippy/rustfmt lints.
- `rust-version = "1.88"` (MSRV) declared in both crates.

### Changed

- The rendering layer now writes to a sink and reuses a single scratch
  buffer across all placeholders of one call; padding no longer builds
  intermediate strings.
- All formatting paths (`rformat!`, `Format`, builder) share the
  `Template` parse + render pipeline.
- `FormatError` is now `#[non_exhaustive]`. Exhaustive matches on it
  need a wildcard arm.

## [0.1.2] - 2026-07-29

### Fixed

- Output divergences from `std::fmt`, found and now guarded by
  differential tests against `std::format!`:
  - the `0` flag always sign-aware zero-fills numerics, overriding
    fill/align (`{:<05}` on `42` renders `00042`, not `42···`);
  - integer precision is ignored, matching current std
    (`{:.5}` on `42` renders `42`);
  - `{:#X}` uses the lowercase `0x` prefix (`0xFF`, not `0XFF`);
  - `+` is never attached to NaN (`{:+}` on NaN renders `NaN`).
- `format()` / `build()` panic messages now report the actual
  `FormatError` instead of always saying "Insufficient parameters".

### Added

- `#![no_std]` support (requires `alloc`); `FormatError` implements
  `core::error::Error`.
- `FormatParam` implementations for tuples up to 16 elements (was 13).
- ~200 differential test assertions comparing `rformat!` against
  `std::format!` (`src/std_compat.rs`).

## [0.1.1] - 2026-07-29

Housekeeping release: removed `.vscode` from version control.

## [0.1.0] - 2026-07-29

Initial release: runtime string formatting with `std::fmt`-compatible
placeholder syntax — implicit/explicit positional arguments, format
types (`{:?}` `{:#?}` `{:x}` `{:X}` `{:o}` `{:b}` `{:e}` `{:E}`),
alignment and fill, sign/`#`/zero flags, width and precision including
`n$` references, `{{`/`}}` escapes, `#[derive(FormatArg)]`, fallible
formatting (`try_format`), and a builder API.

[Unreleased]: https://github.com/YWZYXSoCool/rtformat/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/YWZYXSoCool/rtformat/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/YWZYXSoCool/rtformat/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/YWZYXSoCool/rtformat/compare/a205b52...v0.1.1
[0.1.0]: https://github.com/YWZYXSoCool/rtformat/tree/143f041
