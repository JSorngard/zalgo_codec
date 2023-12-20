# zalgo-codec-macro

This crate provides the macro part of [`zalgo-codec`](https://crates.io/crates/zalgo-codec) by defining the procedural macro [`zalgo_embed!`](https://docs.rs/zalgo-codec-macro/latest/zalgo_codec_macro/macro.zalgo_embed.html).

It lets you take source code that's been converted into a single grapheme cluster by the
[`zalgo-codec-common`](https://crates.io/crates/zalgo-codec-common) crate and compile it as if it was never zalgo-ified.