# zalgo codec

This is a crate implementing the zalgo encoding and decoding functions [originally written in Python](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor) by Scott Conner and extends them for Rust by providing a procedural macro that can run encoded source code.

With the functions defined in this crate you can transform an ASCII string into a unicode string that is a single "character" wide. While the encoding is reversible the encoded string will be larger than the original in terms of bytes.

The crate also provides the `zalgo_embed!` macro that can be used to decode a string of encoded source code and pass the results on to the compiler, resulting in no runtime penalty. This way code can be replaced by a single short line for the ultimate in refactoring.

Additionally the crate provides functions to encode python code and wrap the result in a decoder that decodes and executes the encoded string.

Can not encode carriage returns, so files written on non-unix operating systems might not work. The file encoding functions will attempt to encode files anyway by ignoring carriage returns, but the string encoding functions will return an error.

# Examples

We can execute encoded code with the macro:

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

# Explanation
Characters U+0300–U+036F are the combining characters for unicode Latin. The fun thing about combining characters is that you can add as many of these characters as you like to the original character and it does not create any new symbols, it only adds symbols on top of the character. It's supposed to be used in order to create characters such as á by taking a normal a and adding another character to give it the mark (U+301, in this case). Fun fact, Unicode doesn't specify any limit on the number of these characters. Conveniently, this gives us 112 different characters we can map to, which nicely maps to the ASCII character range 0x20 -> 0x7F, aka all the non-control characters. The only issue is that we can't have new lines in this system, so to fix that, we can simply map 0x7F (DEL) to 0x0A (LF). This can be represented as (CHARACTER - 11) % 133 - 21, and decoded with (CHARACTER + 22) % 133 + 10.  


# Links
The [original post](https://www.reddit.com/r/ProgrammerHumor/comments/yqof9f/the_most_upvoted_comment_picks_the_next_line_of/ivrd9ur/?context=3) where the python code was first presented together with the above explanation.  
[The page for this crate on docs.rs](https://docs.rs/crate/zalgo-codec/latest).
