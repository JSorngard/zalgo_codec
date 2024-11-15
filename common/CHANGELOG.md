# Changelog

This document contains all changes to the crate since version 0.9.4.

## 0.13.0

- Added the `DecodeError` error type.
- Fixed bug in `zalgo_decode` that made it attempt to allocate
 `usize::MAX/2` bytes if the input string was empty.

### Breaking changes

- Renamed `Error` to `EncodeError`.
- Made `zalgo_decode` return a `DecodeError` instead of a `FromUtf8Error`.
- `std` is no longer enabled by default.

## 0.12.2

- Documentation improvements.

## 0.12.1

- Added the `rkyv` feature that derives the serialization traits from the
 [`rkyv`](https://crates.io/crates/rkyv) crate for the `ZalgoString` struct.
- Derive the `Error` trait from `core` instead of `std`.

## 0.12.0

### Breaking changes

- Added a backtrace to the `Error` type, and as a result the error type no longer
 implements `Clone`, `PartialEq`, `Eq`, or `Hash`.

## 0.11.1

- Added links to local versions of licenses.

## 0.11.0

### Breaking changes

- `Error` is now a struct.
- Removed the `Error::byte` function.

### Minor changes

- Added `Error::char` function.
- Added `Error::index` function.
- Implemented the `Default` trait for `ZalgoString`. It just allocates a buffer
 with a single `'E'`.
- Added `ZalgoString::with_capacity` function.
- Added `ZalgoString::encode_and_push_str` function.

## 0.10.4

- Implemented the `Index` trait for the different range types for `ZalgoString`.
- Added the `get` and `get_unchecked` functions to `ZalgoString` that work the same
 as `str::get` and `str::get_unchecked`.
- Added `into_combining_chars` to `ZalgoString` that returns a string that contains
 only the combining charaters of the grapheme cluster
 (that is, without the initial "E").

## 0.10.3

- Added `truncate` and `clear` to `ZalgoString`.

## 0.10.2

- Added `reserve` and `reserve_exact` to `ZalgoString`.

## 0.10.1

- Changed the links to licenses in the readme to be compatible with crates.io.

## 0.10.0

### Breaking changes

- Changed the `encode_and_push_str` method to `push_zalgo_str`, which takes a
 reference to an already encoded `ZalgoString` for an API that doesn't hide as
 many allocations. To port to this version simply change all

  ```rust
  zs.encode_and_push_str(s)?;
  ```

  to
  
  ```rust
  zs.push_zalgo_str(&ZalgoString::new(s)?); 
  ```

  which is what `encode_and_push_str` did under the hood.

### Minor changes

- Made the implementation of `PartialEq` for `ZalgoString` and other string types
 symmetric.
 That is, it's now possible to write equality checks that involve a `ZalgoString`
 in both directions,
 so both `assert_ne!(ZalgoString::new("stuff")?, "stuff");` and
 `assert_ne!("stuff", ZalgoString::new("stuff")?);` compile.
- Implemented `Add` and `AddAssign` to enable the user to append the encoded
 contents of one `ZalgoString` onto the end of another.
- Documentation improvements.

## 0.9.5

- Added `as_combining_chars` and `encode_and_push_str` to `ZalgoString`
- Documentation improvements.
