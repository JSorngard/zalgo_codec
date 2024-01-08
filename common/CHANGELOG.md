# Changelog

This document contains all changes to the crate since version 0.9.4.

## 0.10.2

 - Add `reserve` and `reserve_exact` to `ZalgoString`.

## 0.10.1

 - Change the links to licenses in the readme to be compatible with crates.io.

## 0.10.0

### Breaking changes

 - Change the `encode_and_push_str` method to `push_zalgo_str` that takes a reference to an already encoded `ZalgoString` for a more intuitive API that doesn't hide as many allocations.

### Minor changes

 - Make the implementation of `PartialEq` for `ZalgoString` and other string types symmetric. That is, it's now possible to write equality checks that involve a `ZalgoString` in both directions, so both `assert_ne!(ZalgoString::new("stuff")?, "stuff");` and `assert_ne!("stuff", ZalgoString::new("stuff")?);` compile.
 - Implement `Add` and `AddAssign` to enable the user to append the encoded contents of one `ZalgoString` onto the end of another.
 - Documentation improvements

## 0.9.5

 - Add `as_combining_chars` and `encode_and_push_str` to `ZalgoString`
 - Documentation improvements