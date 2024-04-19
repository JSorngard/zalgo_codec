# zalgo-codec-macro

This crate provides the macro part of the crate [`zalgo-codec`](https://crates.io/crates/zalgo-codec) by defining the procedural macros `zalgo_embed!` and `zalgofy!`.

The first lets you take source code that's been converted into a single grapheme cluster by the
[`zalgo-codec-common`](https://crates.io/crates/zalgo-codec-common) crate and compile it as if it was never zalgo-ified.  
This lets you reach new lows in the field of self-documenting code.

The second lets you encode a string into a single grapheme cluster at compile time.

## Example

If we run [`zalgo-codec-common::zalgo_encode`](https://docs.rs/zalgo-codec-common/latest/zalgo_codec_common/fn.zalgo_encode.html) on the string "fn square(x: i32) -> i32 {x * x}" we can include the `square` function in our program
by putting the resulting grapheme cluster inside `zalgo_embed!`:
```rust
zalgo_embed!("E͎͓͕͉̞͉͆̀͑́͒̈̀̓̒̉̀̍̀̓̒̀͛̀̊̀͘̚͘͘͝ͅ");
assert_eq!(square(10), 100);
```

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
