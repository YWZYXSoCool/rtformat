# Contributing to rtformat

Thanks for your interest! This document explains how the project works and
how to get a change merged.

## Scope

`rtformat` aims to be a runtime string formatter whose output matches
`std::fmt` for the features it supports. As a rule of thumb:

- **In scope:** behavior that brings output closer to `std::fmt`, bug fixes,
  performance improvements, and documentation.
- **Likely out of scope:** syntax or features that diverge from `std::fmt`
  (e.g. named arguments). Open an issue to discuss before starting large
  work.

## Development workflow

1. Fork the repository and create a branch from `main`.
2. Make your change. Add tests for new behavior or bug fixes.
   - Differential tests against `std::format!` live in `src/std_compat.rs`
     and are the preferred way to lock down formatting behavior.
3. Check locally:

   ```sh
   cargo test --workspace
   cargo fmt --all
   cargo clippy --all-targets --workspace -- -D warnings
   ```

4. If your change affects user-visible behavior, add an entry under
   `[Unreleased]` in [`CHANGELOG.md`](CHANGELOG.md).
5. Open a pull request. CI runs the same checks plus a multi-platform test
   matrix and a `no_std` build, so keep it green.

Small fixes (typos, docs) can skip the ceremony — just open a PR.

## Style

- `cargo fmt` is authoritative for formatting; CI enforces it.
- Clippy runs with `-D warnings`, so new lints must be clean.
- The crate supports `#![no_std]` (with `alloc`) and an MSRV of
  `rust-version` declared in `Cargo.toml`. Don't use APIs newer than the
  MSRV without discussion, and avoid pulling in `std`-only APIs into
  library code.

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **patch** — bug fixes, no API changes
- **minor** — new, backwards-compatible functionality
- **major** — breaking changes

CI runs `cargo-semver-checks` against the last published release to catch
accidental breaking changes. If it fails on your PR, your change is likely
breaking and should either be reworked or flagged for a major release.

## Releasing (maintainers)

The workspace contains two crates with a dependency order:
`rtformat` depends on `rtformat-derive`, so publish in that order.

1. Move the `[Unreleased]` section in `CHANGELOG.md` to a new
   `[x.y.z] - YYYY-MM-DD` heading and update the comparison links.
2. Bump the `version` in the relevant `Cargo.toml`. If `rtformat-derive`
   changed, also update the `rtformat-derive` dependency version in the
   root `Cargo.toml`.
3. Run `cargo semver-checks` (or rely on CI) and confirm the bump matches
   the actual change.
4. Dry-run the packaging:

   ```sh
   cargo package -p rtformat-derive
   cargo package -p rtformat
   ```

5. Publish derive first, then wait for the crates.io index to update:

   ```sh
   cargo publish -p rtformat-derive
   # wait a few minutes
   cargo publish -p rtformat
   ```

6. Tag the release so the CHANGELOG comparison links work:

   ```sh
   git tag v<x.y.z>
   git push origin v<x.y.z>
   ```

## Conduct

This project follows the [Rust Code of Conduct](CODE_OF_CONDUCT.md).
