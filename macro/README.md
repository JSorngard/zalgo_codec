# zalgo-codec-macro

This crate provides the macro part of the crate [`zalgo-codec`](https://crates.io/crates/zalgo-codec) by defining the procedural macro `zalgo_embed!`.

It lets you take source code that's been converted into a single grapheme cluster by the
[`zalgo-codec-common`](https://crates.io/crates/zalgo-codec-common) crate and compile it as if it was never zalgo-ified.

## Example

If we run [`zalgo-codec-common::zalgo_encode`](https://docs.rs/zalgo-codec-common/latest/zalgo_codec_common/fn.zalgo_encode.html) on the string "fn square(x: i32) -> i32 {x * x}" we can include the `square` function in our program
by putting the resulting grapheme cluster inside `zalgo_embed!`:
```rust
zalgo_embed!("E͎͓͕͉̞͉͆̀͑́͒̈̀̓̒̉̀̍̀̓̒̀͛̀̊̀͘̚͘͘͝ͅ");
assert_eq!(add(10, 20), 30);
```