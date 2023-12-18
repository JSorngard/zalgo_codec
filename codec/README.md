[![Latest Version](https://img.shields.io/crates/v/zalgo-codec.svg)](https://crates.io/crates/zalgo-codec)
[![codecov](https://codecov.io/gh/JSorngard/zalgo_codec/graph/badge.svg?token=X7TTODVC8I)](https://codecov.io/gh/JSorngard/zalgo_codec)

# zalgo codec

This crate lets you convert an ASCII text string into a single unicode grapheme cluster and back. It also provides a procedural macro that lets you embed such a grapheme cluster and decode it into source code at compile time.  
This lets you reach new lows in the field of self-documenting code.

The encoded string will be ~2 times larger than the original in terms of bytes.

Additionally the crate provides a function to encode Python code and wrap the result in a decoder that decodes and executes it such that the result retains the functionality of the original code.

## Examples

Encode a string to a grapheme cluster with `zalgo_encode`:
```rust
let s = "Zalgo";
let encoded = zalgo_encode(s)?;
assert_eq!(encoded, "É̺͇͌͏");
```
Decode a grapheme cluster back into a string with `zalgo_decode`:
```rust
let encoded = "É̺͇͌͏";
let s = zalgo_decode(encoded)?;
assert_eq!(s, "Zalgo");
```
The `ZalgoString` type can be used to encode a string and handle the result in various ways:
```rust
let s = "Zalgo";
let zstr = ZalgoString::new(s)?;
assert_eq!(zstr, "É̺͇͌͏");
assert_eq!(zstr.len(), 2 * s.len() + 1);
assert_eq!(zstr.decoded_len(), s.len());
assert_eq!(zstr.bytes().next(), Some(69));
assert_eq!(zstr.decoded_chars().next_back(), Some('o'));
```

We can execute zalgo encoded rust code with the macro `zalgo_embed!`:

```rust
// This expands to the code
// `fn add(x: i32, y: i32) -> i32 {x + y}`
zalgo_embed!("E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");

// The `add` function is now available
assert_eq!(add(10, 20), 30);
```

as well as evaluate expressions:

```rust
let x = 20;
let y = -10;
// This expands to the code 
// `x + y`
let z = zalgo_embed!("È͙̋̀͘");
assert_eq!(z, x + y);
```

We can also do the opposite of [obfstr](https://crates.io/crates/obfstr): obfuscate a string while coding and deobfuscate it during compile time
```rust
let secret_string = zalgo_embed!("Ê̤͏͎͔͔͈͉͓͍̇̀͒́̈́̀̀ͅ͏͍́̂");
assert_eq!(secret_string, "Don't read this mom!");
```

The cursed character at the bottom of this section is the standard "Lorem ipsum" encoded with the encoding function in this crate.

\
\
\
\
\
\
\
E̬͏͍͉͓͕͍͒̀͐̀̈́ͅ͏͌͏͓͉͔͍͔͒̀̀́̌̀̓ͅ͏͎͓͔͔͕͉͉͓͉͎͇͉͔͓̓͒̀́̈́͐̓̀͌̌̀̈́̀̈́ͅͅͅͅ͏͉͕͓͍̀ͅ͏͔͍̈́̀͐ͅ͏͉͎͉͉͕͎͔͕͔͒̀̓̈́̈́̀̀͌́͂͏͔͒̀̀̈́ͅͅ͏͌͏͍͇͎͉͒̀́́̀́͌ͅ
\
\
\
\
\
\
\

## Explanation
Characters U+0300–U+036F are the combining characters for unicode Latin. The fun thing about combining characters is that you can add as many of these characters as you like to the original character and it does not create any new symbols, it only adds symbols on top of the character. It's supposed to be used in order to create characters such as `á` by taking a normal `a` and adding another character to give it the mark (U+301, in this case). Fun fact, Unicode doesn't specify any limit on the number of these characters. Conveniently, this gives us 112 different characters we can map to, which nicely maps to the ASCII character range 0x20 -> 0x7F, aka all the non-control characters. The only issue is that we can't have new lines in this system, so to fix that, we can simply map 0x7F (DEL) to 0x0A (LF). This can be represented as `(CHARACTER - 11) % 133 - 21`, and decoded with `(CHARACTER + 22) % 133 + 10`.  


## Experiment with the codec

There is an executable available for experimenting with the codec on text and files.
It can also be used to generate grapheme clusters from source code for use with `zalgo_embed!`.
It can be installed with `cargo install zalgo-codec --features binary`. 
You can optionally enable the `gui` feature during installation to include a rudimentary GUI mode for the program.

## Links
The crate is based on the encoding and decoding functions [originally written in Python](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor) by Scott Conner. They were first presented in [this post](https://www.reddit.com/r/ProgrammerHumor/comments/yqof9f/the_most_upvoted_comment_picks_the_next_line_of/ivrd9ur/?context=3) together with the above explanation.

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
