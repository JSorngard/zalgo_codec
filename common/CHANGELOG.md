# Changelog

This document contains all changes to the crate since version 0.9.4.

## 0.9.6
 - Documentation improvements
 - Make the implementation of `PartialEq` for `ZalgoString` and other string types symmetric. That is, it's now possible
 to write equality checks that involve a `ZalgoString` in both directions, so both 
 `assert_ne!(ZalgoString::new("stuff")?, "stuff");` and `assert_ne!("stuff", ZalgoString::new("stuff")?);` compile.

## 0.9.5

 - Documentation improvements
 - Add `as_combining_chars` and `encode_and_push_str` to `ZalgoString`