# Changelog

This document contains the changes made to the crate since version 0.9.5.
This crate combines the `common` and `macro` crates into one, adds some tests,
and defines the test executable that is built with the "binary" and/or "gui" features.
See [common/CHANGELOG.md](../common/CHANGELOG.md) for the changes made to the
non-macro parts of the crate, and [macro/CHANGELOG.md](../macro/CHANGELOG.md)
for the changes made to the macros.

## 0.13.5

- Add the `documentation` field to the Cargo.toml.

## 0.13.4

- Fix a broken doclink to the `Backtrace` type.
- Improvements to CI.

## 0.13.3

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies.

## 0.13.2

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies.

## 0.13.1

- Set `rust-version` to 1.81.0 since if we don't crates.io sets it to 1.56.0 by default,
 which is too old to compile the crate.
- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies.

## 0.13.0

- Fixed a bug where attempting to decode an empty string would
 result in a crash because the library would attempt to allocate `usize::MAX/2` bytes.
- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies.

### Breaking changes

- `std` is no longer enabled by default.

## 0.12.3

- Re-enabled the help text for the CLI program.

## 0.12.2

- Updated the `rfd` and `iced` dependencies that are used for the optional binary.
- Updated the `zalgo-codec-common` and `zalgo-codec-macro` dependencies.

## 0.12.1

- Added the `rkyv` feature that derives the serialization traits from the
 [`rkyv`](https://crates.io/crates/rkyv) crate for `ZalgoString`.
- Derived the `Error` trait from `core` instead of `std`.
- Disabled unused features in dependencies,
 notbly diabled the `std` feature in several dependencies.
- Updated the `zalgo-codec-common` and `zalgo-codec-macro` dependencies.

## 0.12.0

- Updated `zalgo-codec-common` dependency. For more information about breaking
 changes see its changelog.

## 0.11.1

- Added links to local versions of licenses.  
- Added docs.rs badge.

## 0.11.0

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies
- Updated `iced` dependency used for the optional GUI to version 0.12
- For more information about breaking changes see the changelog of `zalgo-codec-common`.

## 0.10.4

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies

## 0.10.3

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies

## 0.10.2

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies

## 0.10.1

- Changed the links to licenses in the readme to be compatible with crates.io.
- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies

## 0.10.0

- Made `anyhow` an optional dependency that is only enabled when the binary is built.
- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies

## 0.9.6

- Updated `zalgo-codec-common` and `zalgo-codec-macro` dependencies
