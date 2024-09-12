# Changelog

This document contains the changes made to the crate since version 0.9.5.
This crate combines the `common` and `macro` crates into one, adds some tests,
and defines the test executable that is built with the "binary" and/or "gui" features.
See [common/CHANGELOG.md](../common/CHANGELOG.md) for the changes made to the
non-macro parts of the crate, and [macro/CHANGELOG.md](../macro/CHANGELOG.md)
for the changes made to the macros.

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
