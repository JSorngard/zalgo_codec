//! This crate lets you convert an ASCII text string into a single unicode grapheme cluster and back.
//! It also provides a procedural macro that lets you embed such a grapheme cluster and decode it into source code at compile time.  
//! This lets you reach new lows in the field of self-documenting code.
//!
//! The encoded string will be ~2 times larger than the original in terms of bytes.
//!
//! Additionally the crate provides a function to encode Python code and wrap the result in a decoder that
//! decodes and executes the encoded string, retaining the functionality of the original code.
//!
//! There are two ways of interacting with the codec.
//! The first one is to call the encoding and decoding functions directly,
//! and the second one is to use the [`ZalgoString`] wrapper type.
//!
//! # Examples
//!
//! Encode a string to a grapheme cluster with `zalgo_encode`:
//! ```
//! # use zalgo_codec::{Error, zalgo_encode};
//! let s = "Zalgo";
//! let encoded = zalgo_encode(s)?;
//! assert_eq!(encoded, "É̺͇͌͏");
//! # Ok::<(), Error>(())
//! ```
//! Decode a grapheme cluster back into a string:
//! ```
//! # use zalgo_codec::zalgo_decode;
//! # extern crate alloc;
//! # use alloc::string::FromUtf8Error;
//! let encoded = "É̺͇͌͏";
//! let s = zalgo_decode(encoded)?;
//! assert_eq!(s, "Zalgo");
//! # Ok::<(), FromUtf8Error>(())
//! ```
//! The [`ZalgoString`] type can be used to encode a string and handle the result in various ways:
//! ```
//! # use zalgo_codec::{Error, ZalgoString};
//! let s = "Zalgo";
//! let zstr = ZalgoString::new(s)?;
//! assert_eq!(zstr, "É̺͇͌͏");
//! assert_eq!(zstr.len(), 2 * s.len() + 1);
//! assert_eq!(zstr.decoded_len(), s.len());
//! assert_eq!(zstr.bytes().next(), Some(69));
//! assert_eq!(zstr.decoded_chars().next_back(), Some('o'));
//! # Ok::<(), Error>(())
//! ```
//!
#![cfg_attr(
    feature = "macro",
    doc = " Encode Rust source code and embed it in your program with the [`zalgo_embed!`] proc-macro.\n ```\n # use zalgo_codec::zalgo_embed;\n // This grapheme cluster was made by encoding \"fn add(x: i32, y: i32) -> i32 {x + y}\"\n zalgo_embed!(\"E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝\");\n\n // The `add` function is now available\n assert_eq!(add(10, 20), 30);\n ```"
)]
//!
//! # Features
//!
//! `std` *(enabled by default)*: links the standard library and uses it to implement the [`std::error::Error`] trait for the provided [`Error`] type.
//! If this feature is not enabled the library is `#[no_std]`, but still uses the `alloc` crate.
//!
//! `serde`: implements the `Serialize` and `Deserialize` traits from [`serde`](https://crates.io/crates/serde) for [`ZalgoString`].
//!
//! `macro` *(enabled by default)*: exports the procedural macro [`zalgo_embed!`].
//!   
//! # Explanation
//!
//! Characters U+0300–U+036F are the combining characters for unicode Latin.
//! The fun thing about combining characters is that you can add as many of these characters
//! as you like to the original character and it does not create any new symbols,
//! it only adds symbols on top of the character. It's supposed to be used in order to
//! create characters such as `á` by taking a normal `a` and adding another character
//! to give it the mark (U+301, in this case). Fun fact: Unicode doesn't specify
//! any limit on the number of these characters.
//! Conveniently, this gives us 112 different characters we can map to,
//! which nicely maps to the ASCII character range 0x20 -> 0x7F, aka all the non-control characters.
//! The only issue is that we can't have new lines in this system, so to fix that,
//! we can simply map 0x7F (DEL) to 0x0A (LF).
//! This can be represented as `(CHARACTER - 11) % 133 - 21`, and decoded with `(CHARACTER + 22) % 133 + 10`.  
//!
//! # Experiment with the codec
//!
//! There is an executable available for experimenting with the codec on text and files.
//! It can also be used to generate grapheme clusters from source code for use with [`zalgo_embed!`].
//! It can be installed with `cargo install zalgo-codec --features binary`.
//! You can optionally enable the `gui` feature during installation to include a rudimentary GUI mode for the program.

#![cfg_attr(not(feature = "std"), no_std)]

pub use zalgo_codec_common::{
    zalgo_decode, zalgo_encode, zalgo_string, zalgo_wrap_python, Error, ZalgoString,
};

#[cfg(feature = "macro")]
pub use zalgo_codec_macro::zalgo_embed;

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::string::String;
    use core::str;
    use rand::{
        distributions::{DistString, Distribution},
        seq::SliceRandom,
        Rng,
    };
    use unicode_segmentation::UnicodeSegmentation;

    struct PrintableAsciiAndNewline;

    impl Distribution<char> for PrintableAsciiAndNewline {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
            *b" !\"#$%&'()*,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVXYZ[\\]^_`abcdefghijklmnopqrstuvxyz{|}~\n".choose(rng).unwrap() as char
        }
    }

    impl DistString for PrintableAsciiAndNewline {
        fn append_string<R: Rng + ?Sized>(&self, rng: &mut R, string: &mut String, len: usize) {
            string.reserve(len);
            for _ in 0..len {
                string.push(self.sample(rng));
            }
        }
    }

    #[cfg(feature = "macro")]
    #[test]
    fn test_embed_function() {
        let code = "fn add(x: i32, y: i32) -> i32 {x + y}";

        let encoded = zalgo_encode(code).unwrap();
        assert_eq!(encoded, "E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");

        zalgo_embed!("E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");

        // Now the `add` function is available
        assert_eq!(add(10, 20), 30)
    }

    #[cfg(feature = "macro")]
    #[test]
    fn test_embed_expression() {
        let x = 20;
        let y = -10;

        let expr = "x + y";

        let encoded = zalgo_encode(expr).unwrap();

        assert_eq!(encoded, "È͙̋̀͘");

        // It works on expressions, too!
        let z = zalgo_embed!("È͙̋̀͘");
        assert_eq!(z, x + y);
    }

    #[test]
    fn verify() {
        const TEST_STRING_1: &str = "the greatest adventure is going to bed";
        let out_string = str::from_utf8(b"E\xcd\x94\xcd\x88\xcd\x85\xcc\x80\xcd\x87\xcd\x92\xcd\x85\xcd\x81\xcd\x94\xcd\x85\xcd\x93\xcd\x94\xcc\x80\xcd\x81\xcd\x84\xcd\x96\xcd\x85\xcd\x8e\xcd\x94\xcd\x95\xcd\x92\xcd\x85\xcc\x80\xcd\x89\xcd\x93\xcc\x80\xcd\x87\xcd\x8f\xcd\x89\xcd\x8e\xcd\x87\xcc\x80\xcd\x94\xcd\x8f\xcc\x80\xcd\x82\xcd\x85\xcd\x84").unwrap();
        assert_eq!(zalgo_encode(TEST_STRING_1).unwrap(), out_string);

        const TEST_STRING_2: &str =
            "I'll have you know I graduated top of my class in the Navy Seals";
        assert_eq!(
            zalgo_decode(&zalgo_encode(TEST_STRING_2).unwrap()).unwrap(),
            TEST_STRING_2
        );

        const ASCII_CHAR_TABLE: &str = r##"ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvxyz1234567890 !"#$%&'()*+,-\./:;<=>?@"##;
        assert_eq!(
            zalgo_decode(&zalgo_encode(ASCII_CHAR_TABLE).unwrap()).unwrap(),
            ASCII_CHAR_TABLE
        );

        // Checking that randomly generated alphanumeric strings are encoded in a lossless fashion, and that they contain a single grapheme cluster
        for _ in 0..100 {
            let s = PrintableAsciiAndNewline.sample_string(&mut rand::thread_rng(), 100);
            let encoded = zalgo_encode(&s).unwrap();
            assert_eq!(zalgo_decode(&encoded).unwrap(), s);
            assert_eq!(encoded.as_str().graphemes(true).count(), 1)
        }
    }

    #[test]
    fn newlines() {
        assert_eq!(&zalgo_encode("\n").unwrap(), "Eͯ",);
        const TEST_STRING: &str = "The next sentence is true.\nThe previous sentence is false.";
        assert_eq!(
            zalgo_decode(&zalgo_encode(TEST_STRING).unwrap()).unwrap(),
            TEST_STRING,
        );
    }

    #[test]
    fn check_errors() {
        assert!(zalgo_encode("We got the Ä Ö Å, you aint got the Ä Ö Å").is_err());
        assert!(zalgo_encode("\t").is_err());
        assert!(zalgo_encode("\r").is_err());
        assert!(zalgo_encode("\0").is_err());
    }
}
