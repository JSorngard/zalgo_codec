# zalgo-codec-common

A crate for converting a string containing only printable ASCII and newlines
into a single unicode grapheme cluster and back.
Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).

There are two ways of interacting with the codec.
The first is to call the encoding and decoding functions directly,
and the second is to use the `ZalgoString` wrapper type.

## Examples

Encode a string to a grapheme cluster with `zalgo_encode`:
```rust
let s = "Zalgo";
let encoded = zalgo_encode(s)?;
assert_eq!(encoded, "É̺͇͌͏");
```
Decode a grapheme cluster back into a string:
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

## Explanation
Characters U+0300–U+036F are the combining characters for unicode Latin.
The fun thing about combining characters is that you can add as many of these characters
as you like to the original character and it does not create any new symbols,
it only adds symbols on top of the character. It's supposed to be used in order to
create characters such as `á` by taking a normal `a` and adding another character
to give it the mark (U+301, in this case). Fun fact: Unicode doesn't specify
any limit on the number of these characters.
Conveniently, this gives us 112 different characters we can map to,
which nicely maps to the ASCII character range 0x20 -> 0x7F, aka all the non-control characters.
The only issue is that we can't have new lines in this system, so to fix that,
we can simply map 0x7F (DEL) to 0x0A (LF).
This can be represented as `(CHARACTER - 11) % 133 - 21`, and decoded with `(CHARACTER + 22) % 133 + 10`.

## Experiment with the codec

There is an executable available for experimenting with the codec on text and files.
It can be installed with `cargo install zalgo-codec --features binary`. 
You can optionally enable the `gui` feature during installation to include a GUI mode for the program.