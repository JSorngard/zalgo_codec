//! A crate for converting a string containing only printable ASCII and newlines
//! into a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).
//!
//! There are two ways of interacting with the codec.
//! The first is to call the encoding and decoding functions directly,
//! and the second is to use the [`ZalgoString`] wrapper type.
//!
//! # Examples
//!
//! Encode a string to a grapheme cluster with [`zalgo_encode`]:
//! ```
//! # use zalgo_codec_common::{EncodeError, zalgo_encode};
//! let s = "Zalgo";
//! let encoded = zalgo_encode(s)?;
//! assert_eq!(encoded, "̺͇́͌͏");
//! # Ok::<(), EncodeError>(())
//! ```
//! Decode a grapheme cluster back into a string:
//! ```
//! # use zalgo_codec_common::{zalgo_decode, DecodeError};
//! let encoded = "̺͇́͌͏";
//! let s = zalgo_decode(encoded)?;
//! assert_eq!(s, "Zalgo");
//! # Ok::<(), DecodeError>(())
//! ```
//!
//! The [`ZalgoString`] type can be used to encode a string and handle the result in various ways:
//!
//! ```
//! # use zalgo_codec_common::{ZalgoString, EncodeError};
//! # fn main() -> Result<(), EncodeError> {
//! let s = "Zalgo";
//! let zstr = ZalgoString::try_from(s)?;
//!
//! // Implements PartialEq with common string types
//! assert_eq!(zstr, "̺͇́͌͏");
//!
//! // Utility functions
//! assert_eq!(zstr.len(), 2 * s.len());
//! assert_eq!(zstr.decoded_len(), s.len());
//!
//! // Iterate over bytes and chars, in both encoded and decoded form
//! assert_eq!(zstr.bytes().next(), Some(204));
//! assert_eq!(zstr.decoded_bytes().nth_back(2), Some(b'l'));
//! assert_eq!(zstr.chars().nth(1), Some('\u{341}'));
//! assert_eq!(zstr.decoded_chars().next_back(), Some('o'));
//!
//! // Decode inplace
//! assert_eq!(zstr.into_decoded_string(), "Zalgo");
//! # Ok(())
//! # }
//! ```
//!
//! # Feature flags
//!
//! `std`: enables [`EncodeError`] and [`DecodeError`] to capture a [`Backtrace`](std::backtrace::Backtrace).
//! If this feature is not enabled the library is `no_std` compatible, but still uses the `alloc` crate.
//!
//! `serde`: derives the [`serde::Serialize`] and [`serde::Deserialize`] traits
//! from [`serde`] for [`ZalgoString`].
//!
//! `rkyv`: derives the [`rkyv::Serialize`], [`rkyv::Deserialize`], and [`rkyv::Archive`] traits from [`rkyv`] for [`ZalgoString`].
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
//! <details><summary><b>Full conversion table</b></summary>
//!
//! | ASCII character | Encoded |  
//! |---|---|  
//! | A | U+321 |  
//! | B | U+322 |  
//! | C | U+323 |  
//! | D | U+324 |  
//! | E | U+325 |  
//! | F | U+326 |  
//! | G | U+327 |  
//! | H | U+328 |  
//! | I | U+329 |  
//! | J | U+32A |  
//! | K | U+32B |  
//! | L | U+32C |  
//! | M | U+32D |  
//! | N | U+32E |  
//! | O | U+32F |  
//! | P | U+330 |  
//! | Q | U+331 |  
//! | R | U+332 |  
//! | S | U+333 |  
//! | T | U+334 |  
//! | U | U+335 |  
//! | V | U+336 |  
//! | W | U+337 |  
//! | X | U+338 |  
//! | Y | U+339 |  
//! | Z | U+33A |  
//! | a | U+341 |  
//! | b | U+342 |  
//! | c | U+343 |  
//! | d | U+344 |  
//! | e | U+345 |  
//! | f | U+346 |  
//! | g | U+347 |  
//! | h | U+348 |  
//! | i | U+349 |  
//! | j | U+34A |  
//! | k | U+34B |  
//! | l | U+34C |  
//! | m | U+34D |  
//! | n | U+34E |  
//! | o | U+34F |  
//! | p | U+350 |  
//! | q | U+351 |  
//! | r | U+352 |  
//! | s | U+353 |  
//! | t | U+354 |  
//! | u | U+355 |  
//! | v | U+356 |  
//! | w | U+357 |  
//! | x | U+358 |  
//! | y | U+359 |  
//! | z | U+35A |  
//! | 1 | U+311 |  
//! | 2 | U+312 |  
//! | 3 | U+313 |  
//! | 4 | U+314 |  
//! | 5 | U+315 |  
//! | 6 | U+316 |  
//! | 7 | U+317 |  
//! | 8 | U+318 |  
//! | 9 | U+319 |  
//! | 0 | U+310 |  
//! |   | U+300 |  
//! | ! | U+301 |  
//! | " | U+302 |  
//! | # | U+303 |  
//! | $ | U+304 |  
//! | % | U+305 |  
//! | & | U+306 |  
//! | ' | U+307 |  
//! | ( | U+308 |  
//! | ) | U+309 |  
//! | * | U+30A |  
//! | + | U+30B |  
//! | , | U+30C |  
//! | - | U+30D |  
//! | \ | U+33C |  
//! | . | U+30E |  
//! | / | U+30F |  
//! | : | U+31A |  
//! | ; | U+31B |  
//! | < | U+31C |  
//! | = | U+31D |  
//! | > | U+31E |  
//! | ? | U+31F |  
//! | @ | U+320 |  
//! | \n| U+36F |  
//!
//! </details>
//!
//! # Experiment with the codec
//!
//! There is an executable available for experimenting with the codec on text and files.
//! It can be installed with `cargo install zalgo-codec --features binary`.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;
use alloc::{format, string::String, vec, vec::Vec};
use core::{fmt, str};

mod error;
pub mod zalgo_string;

pub use error::{DecodeError, EncodeError};
pub use zalgo_string::ZalgoString;

/// Takes in a string slice that consists of only printable ACII and newline characters
/// and encodes it into a single grapheme cluster using a reversible encoding scheme.
///
/// The resulting string is a single unicode grapheme cluster and should
/// only take up a single character space horizontally when displayed
/// (though this can vary between platforms depending on how they deal with unicode).
/// The resulting string will be ~2 times larger than the original in terms of bytes, and it
/// can be decoded to recover the original string with [`zalgo_decode`].
///
/// # Errors
///
/// Returns an error if the input contains a byte that does not correspond to a printable
/// ASCII character or newline.
/// Notably this means that this function can not encode tab or carriage return characters.
/// Carriage returns are present in e.g. line endings on Windows.
///
/// # Example
///
/// Basic usage:
/// ```
/// # use zalgo_codec_common::{EncodeError, zalgo_encode};
/// assert_eq!(zalgo_encode("Zalgo")?, "̺͇́͌͏");
/// # Ok::<(), EncodeError>(())
/// ```
/// Can not encode non-ASCII characters or ASCII control characters except newlines:
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// assert!(zalgo_encode("Windows line ending: \r\n").is_err());
/// assert!(zalgo_encode("Zålgö").is_err());
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_encode(string: &str) -> Result<String, EncodeError> {
    // We will encode this many bytes at a time before pushing onto the result vector.
    const BATCH_SIZE: usize = 16;

    // The line we are currently encoding
    let mut line = 1;
    // The column on that line we are currently encoding
    let mut column = 1;
    // These are used for reporting a useful error if the encoding process fails.

    // Every byte in the input will encode to two bytes. The extra byte is for the initial letter
    // which is there in order for the output to be displayable in an intuitive way.
    let mut result = Vec::with_capacity(2 * string.len());

    for (i, batch) in string.as_bytes().chunks(BATCH_SIZE).enumerate() {
        let mut buffer = [0; 2 * BATCH_SIZE];
        let mut encoded = 0;
        for (j, byte) in batch.iter().enumerate() {
            // Only encode ASCII bytes corresponding to printable characters or newlines.
            if (32..127).contains(byte) || *byte == b'\n' {
                if *byte == b'\n' {
                    line += 1;
                    // `column` is still 1-indexed since it gets incremented at the end of the current loop iteration.
                    column = 0;
                }

                let v = ((i16::from(*byte) - 11).rem_euclid(133) - 21) as u8;
                buffer[encoded] = (v >> 6) & 1 | 0b1100_1100;
                buffer[encoded + 1] = (v & 63) | 0b1000_0000;
                encoded += 2;
                column += 1;
            } else {
                let index = i * BATCH_SIZE + j;
                // The panic should never trigger since we know that string[i*BATCH_SIZE + j]
                // has some value which is stored in `byte`, and that this value is the first
                // byte of a non-ascii character and that Strings in Rust are valid utf-8.
                // All of this means that the value that starts at this index is a utf-8 encoded
                // character, which `chars.next()` will extract.
                let unencodable_character = string[index..].chars().next()
                // TODO: Find a way to get rid of the expect.
                    .expect("i*BATCH_SIZE + j is within the string and on a char boundary, so string.chars().next() should find a char");
                return Err(EncodeError::new(unencodable_character, line, column, index));
            }
        }
        result.extend_from_slice(&buffer[..encoded]);
    }

    // Safety: the encoding process does not produce invalid UTF-8
    // if given valid printable ASCII + newlines,
    // which is checked before this point
    Ok(unsafe { String::from_utf8_unchecked(result) })
}

/// Takes in a string that was encoded by [`zalgo_encode`] and decodes it back into an ASCII string.
///
/// # Errors
///
/// Returns an error if the decoded string is not valid UTF-8.
/// This can happen if the input is empty, or if it is a string that was not encoded by [`zalgo_encode`],
/// since the byte manipulations that this function does could result in invalid unicode in that case.
/// Even if no error is returned in such a case the results are not meaningful.
/// If you want to be able to decode without this check, consider using a [`ZalgoString`].
///
/// # Examples
///
/// Basic usage:
/// ```
/// # use zalgo_codec_common::{zalgo_decode, DecodeError};
/// assert_eq!(zalgo_decode("̺͇́͌͏")?, "Zalgo");
/// # Ok::<(), DecodeError>(())
/// ```
/// Decoding arbitrary strings that were not produced by [`zalgo_encode`] will most likely lead to errors:
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert!(zalgo_decode("Blorbo goes to space").is_err());
/// ```
/// If it doesn't the results are not meaningful:
/// ```
/// # use zalgo_codec_common::{zalgo_decode, DecodeError};
/// assert_eq!(zalgo_decode("awö")?, "\u{12}\u{11}");
/// # Ok::<(), DecodeError>(())
/// ```
pub fn zalgo_decode(encoded: &str) -> Result<String, DecodeError> {
    if encoded.is_empty() {
        return Ok(String::new());
    }
    let mut res = vec![0; encoded.len() / 2];
    let bytes = encoded.as_bytes();

    for (write, read) in (0..encoded.len()).step_by(2).enumerate() {
        match bytes.get(read + 1) {
            Some(next) => res[write] = decode_byte_pair(bytes[read], *next),
            None => break,
        }
    }

    String::from_utf8(res).map_err(|e| DecodeError::new(e))
}

#[inline]
#[must_use = "the function returns a new value and does not modify its inputs"]
const fn decode_byte_pair(odd: u8, even: u8) -> u8 {
    (((odd << 6) & 64 | even & 63) + 22) % 133 + 10
}

/// zalgo-encodes an ASCII string containing Python code and
/// wraps it in a decoder that decodes and executes it.
/// The resulting Python code should retain the functionality of the original.
///
/// # Example
///
/// Encode a simple hello world program in Python
/// ```
/// # use zalgo_codec_common::{EncodeError, zalgo_wrap_python};
/// let py_hello_world = "print(\"Hello, world!\")\n";
/// let py_hello_world_enc = zalgo_wrap_python(py_hello_world)?;
/// assert_eq!(
///     py_hello_world_enc,
///     "b='̨͉͎͔͐͒̈̂͌͌ͅ͏̌̀͗͏͒͌̈́́̂̉ͯ'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[0::2],b[1::2])))",
/// );
/// # Ok::<(), EncodeError>(())
/// ```
/// If the contents of the variable `py_hello_world_enc` in
/// the above code snippet is saved to a file
/// you can run it with python and it will produce the output
/// that is expected of the code in the variable `py_hello_world`.
/// In the example below the file is named `enc.py`.
/// ```bash
/// $ python enc.py
/// Hello, world!
/// ```
///
/// # Known issues
///
/// May not work correctly on python versions before 3.10,
/// see [this github issue](https://github.com/DaCoolOne/DumbIdeas/issues/1) for more information.
///
/// # Errors
///
/// Returns an error if the input contains a byte that does not correspond to a printable
/// ASCII character or newline.
/// ```
/// # use zalgo_codec_common::zalgo_wrap_python;
/// let res = zalgo_wrap_python(r#"print("That will be 5€ please")"#);
/// assert_eq!(
///     res.map_err(|e| (e.char(), e.line(), e.column())),
///     Err(('€', 1, 22)),
/// );
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_wrap_python(python: &str) -> Result<String, EncodeError> {
    let encoded_string = zalgo_encode(python)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[0::2],b[1::2])))"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_char() {
        assert_eq!(zalgo_encode("Zalgo\r").map_err(|e| e.char()), Err('\r'));
        assert_eq!(zalgo_encode("Zålgo").map_err(|e| e.char()), Err('å'));
    }

    #[test]
    fn test_empty_decode() {
        assert_eq!(zalgo_decode("").ok(), Some(String::new()));
    }

    #[test]
    fn verify_conversion_table() {
        assert_eq!(zalgo_encode("A").unwrap(), "\u{321}");
        assert_eq!(zalgo_decode("\u{321}").unwrap(), "A");
        assert_eq!(zalgo_encode("B").unwrap(), "\u{322}");
        assert_eq!(zalgo_decode("\u{322}").unwrap(), "B");
        assert_eq!(zalgo_encode("C").unwrap(), "\u{323}");
        assert_eq!(zalgo_decode("\u{323}").unwrap(), "C");
        assert_eq!(zalgo_encode("D").unwrap(), "\u{324}");
        assert_eq!(zalgo_decode("\u{324}").unwrap(), "D");
        assert_eq!(zalgo_encode("E").unwrap(), "\u{325}");
        assert_eq!(zalgo_decode("\u{325}").unwrap(), "E");
        assert_eq!(zalgo_encode("F").unwrap(), "\u{326}");
        assert_eq!(zalgo_decode("\u{326}").unwrap(), "F");
        assert_eq!(zalgo_encode("G").unwrap(), "\u{327}");
        assert_eq!(zalgo_decode("\u{327}").unwrap(), "G");
        assert_eq!(zalgo_encode("H").unwrap(), "\u{328}");
        assert_eq!(zalgo_decode("\u{328}").unwrap(), "H");
        assert_eq!(zalgo_encode("I").unwrap(), "\u{329}");
        assert_eq!(zalgo_decode("\u{329}").unwrap(), "I");
        assert_eq!(zalgo_encode("J").unwrap(), "\u{32a}");
        assert_eq!(zalgo_decode("\u{32a}").unwrap(), "J");
        assert_eq!(zalgo_encode("K").unwrap(), "\u{32b}");
        assert_eq!(zalgo_decode("\u{32b}").unwrap(), "K");
        assert_eq!(zalgo_encode("L").unwrap(), "\u{32c}");
        assert_eq!(zalgo_decode("\u{32c}").unwrap(), "L");
        assert_eq!(zalgo_encode("M").unwrap(), "\u{32d}");
        assert_eq!(zalgo_decode("\u{32d}").unwrap(), "M");
        assert_eq!(zalgo_encode("N").unwrap(), "\u{32e}");
        assert_eq!(zalgo_decode("\u{32e}").unwrap(), "N");
        assert_eq!(zalgo_encode("O").unwrap(), "\u{32f}");
        assert_eq!(zalgo_decode("\u{32f}").unwrap(), "O");
        assert_eq!(zalgo_encode("P").unwrap(), "\u{330}");
        assert_eq!(zalgo_decode("\u{330}").unwrap(), "P");
        assert_eq!(zalgo_encode("Q").unwrap(), "\u{331}");
        assert_eq!(zalgo_decode("\u{331}").unwrap(), "Q");
        assert_eq!(zalgo_encode("R").unwrap(), "\u{332}");
        assert_eq!(zalgo_decode("\u{332}").unwrap(), "R");
        assert_eq!(zalgo_encode("S").unwrap(), "\u{333}");
        assert_eq!(zalgo_decode("\u{333}").unwrap(), "S");
        assert_eq!(zalgo_encode("T").unwrap(), "\u{334}");
        assert_eq!(zalgo_decode("\u{334}").unwrap(), "T");
        assert_eq!(zalgo_encode("U").unwrap(), "\u{335}");
        assert_eq!(zalgo_decode("\u{335}").unwrap(), "U");
        assert_eq!(zalgo_encode("V").unwrap(), "\u{336}");
        assert_eq!(zalgo_decode("\u{336}").unwrap(), "V");
        assert_eq!(zalgo_encode("W").unwrap(), "\u{337}");
        assert_eq!(zalgo_decode("\u{337}").unwrap(), "W");
        assert_eq!(zalgo_encode("X").unwrap(), "\u{338}");
        assert_eq!(zalgo_decode("\u{338}").unwrap(), "X");
        assert_eq!(zalgo_encode("Y").unwrap(), "\u{339}");
        assert_eq!(zalgo_decode("\u{339}").unwrap(), "Y");
        assert_eq!(zalgo_encode("Z").unwrap(), "\u{33a}");
        assert_eq!(zalgo_decode("\u{33a}").unwrap(), "Z");
        assert_eq!(zalgo_encode("a").unwrap(), "\u{341}");
        assert_eq!(zalgo_decode("\u{341}").unwrap(), "a");
        assert_eq!(zalgo_encode("b").unwrap(), "\u{342}");
        assert_eq!(zalgo_decode("\u{342}").unwrap(), "b");
        assert_eq!(zalgo_encode("c").unwrap(), "\u{343}");
        assert_eq!(zalgo_decode("\u{343}").unwrap(), "c");
        assert_eq!(zalgo_encode("d").unwrap(), "\u{344}");
        assert_eq!(zalgo_decode("\u{344}").unwrap(), "d");
        assert_eq!(zalgo_encode("e").unwrap(), "\u{345}");
        assert_eq!(zalgo_decode("\u{345}").unwrap(), "e");
        assert_eq!(zalgo_encode("f").unwrap(), "\u{346}");
        assert_eq!(zalgo_decode("\u{346}").unwrap(), "f");
        assert_eq!(zalgo_encode("g").unwrap(), "\u{347}");
        assert_eq!(zalgo_decode("\u{347}").unwrap(), "g");
        assert_eq!(zalgo_encode("h").unwrap(), "\u{348}");
        assert_eq!(zalgo_decode("\u{348}").unwrap(), "h");
        assert_eq!(zalgo_encode("i").unwrap(), "\u{349}");
        assert_eq!(zalgo_decode("\u{349}").unwrap(), "i");
        assert_eq!(zalgo_encode("j").unwrap(), "\u{34a}");
        assert_eq!(zalgo_decode("\u{34a}").unwrap(), "j");
        assert_eq!(zalgo_encode("k").unwrap(), "\u{34b}");
        assert_eq!(zalgo_decode("\u{34b}").unwrap(), "k");
        assert_eq!(zalgo_encode("l").unwrap(), "\u{34c}");
        assert_eq!(zalgo_decode("\u{34c}").unwrap(), "l");
        assert_eq!(zalgo_encode("m").unwrap(), "\u{34d}");
        assert_eq!(zalgo_decode("\u{34d}").unwrap(), "m");
        assert_eq!(zalgo_encode("n").unwrap(), "\u{34e}");
        assert_eq!(zalgo_decode("\u{34e}").unwrap(), "n");
        assert_eq!(zalgo_encode("o").unwrap(), "\u{34f}");
        assert_eq!(zalgo_decode("\u{34f}").unwrap(), "o");
        assert_eq!(zalgo_encode("p").unwrap(), "\u{350}");
        assert_eq!(zalgo_decode("\u{350}").unwrap(), "p");
        assert_eq!(zalgo_encode("q").unwrap(), "\u{351}");
        assert_eq!(zalgo_decode("\u{351}").unwrap(), "q");
        assert_eq!(zalgo_encode("r").unwrap(), "\u{352}");
        assert_eq!(zalgo_decode("\u{352}").unwrap(), "r");
        assert_eq!(zalgo_encode("s").unwrap(), "\u{353}");
        assert_eq!(zalgo_decode("\u{353}").unwrap(), "s");
        assert_eq!(zalgo_encode("t").unwrap(), "\u{354}");
        assert_eq!(zalgo_decode("\u{354}").unwrap(), "t");
        assert_eq!(zalgo_encode("u").unwrap(), "\u{355}");
        assert_eq!(zalgo_decode("\u{355}").unwrap(), "u");
        assert_eq!(zalgo_encode("v").unwrap(), "\u{356}");
        assert_eq!(zalgo_decode("\u{356}").unwrap(), "v");
        assert_eq!(zalgo_encode("w").unwrap(), "\u{357}");
        assert_eq!(zalgo_decode("\u{357}").unwrap(), "w");
        assert_eq!(zalgo_encode("x").unwrap(), "\u{358}");
        assert_eq!(zalgo_decode("\u{358}").unwrap(), "x");
        assert_eq!(zalgo_encode("y").unwrap(), "\u{359}");
        assert_eq!(zalgo_decode("\u{359}").unwrap(), "y");
        assert_eq!(zalgo_encode("z").unwrap(), "\u{35a}");
        assert_eq!(zalgo_decode("\u{35a}").unwrap(), "z");
        assert_eq!(zalgo_encode("1").unwrap(), "\u{311}");
        assert_eq!(zalgo_decode("\u{311}").unwrap(), "1");
        assert_eq!(zalgo_encode("2").unwrap(), "\u{312}");
        assert_eq!(zalgo_decode("\u{312}").unwrap(), "2");
        assert_eq!(zalgo_encode("3").unwrap(), "\u{313}");
        assert_eq!(zalgo_decode("\u{313}").unwrap(), "3");
        assert_eq!(zalgo_encode("4").unwrap(), "\u{314}");
        assert_eq!(zalgo_decode("\u{314}").unwrap(), "4");
        assert_eq!(zalgo_encode("5").unwrap(), "\u{315}");
        assert_eq!(zalgo_decode("\u{315}").unwrap(), "5");
        assert_eq!(zalgo_encode("6").unwrap(), "\u{316}");
        assert_eq!(zalgo_decode("\u{316}").unwrap(), "6");
        assert_eq!(zalgo_encode("7").unwrap(), "\u{317}");
        assert_eq!(zalgo_decode("\u{317}").unwrap(), "7");
        assert_eq!(zalgo_encode("8").unwrap(), "\u{318}");
        assert_eq!(zalgo_decode("\u{318}").unwrap(), "8");
        assert_eq!(zalgo_encode("9").unwrap(), "\u{319}");
        assert_eq!(zalgo_decode("\u{319}").unwrap(), "9");
        assert_eq!(zalgo_encode("0").unwrap(), "\u{310}");
        assert_eq!(zalgo_decode("\u{310}").unwrap(), "0");
        assert_eq!(zalgo_encode(" ").unwrap(), "\u{300}");
        assert_eq!(zalgo_decode("\u{300}").unwrap(), " ");
        assert_eq!(zalgo_encode("!").unwrap(), "\u{301}");
        assert_eq!(zalgo_decode("\u{301}").unwrap(), "!");
        assert_eq!(zalgo_encode("\"").unwrap(), "\u{302}");
        assert_eq!(zalgo_decode("\u{302}").unwrap(), "\"");
        assert_eq!(zalgo_encode("#").unwrap(), "\u{303}");
        assert_eq!(zalgo_decode("\u{303}").unwrap(), "#");
        assert_eq!(zalgo_encode("$").unwrap(), "\u{304}");
        assert_eq!(zalgo_decode("\u{304}").unwrap(), "$");
        assert_eq!(zalgo_encode("%").unwrap(), "\u{305}");
        assert_eq!(zalgo_decode("\u{305}").unwrap(), "%");
        assert_eq!(zalgo_encode("&").unwrap(), "\u{306}");
        assert_eq!(zalgo_decode("\u{306}").unwrap(), "&");
        assert_eq!(zalgo_encode("'").unwrap(), "\u{307}");
        assert_eq!(zalgo_decode("\u{307}").unwrap(), "'");
        assert_eq!(zalgo_encode("(").unwrap(), "\u{308}");
        assert_eq!(zalgo_decode("\u{308}").unwrap(), "(");
        assert_eq!(zalgo_encode(")").unwrap(), "\u{309}");
        assert_eq!(zalgo_decode("\u{309}").unwrap(), ")");
        assert_eq!(zalgo_encode("*").unwrap(), "\u{30a}");
        assert_eq!(zalgo_decode("\u{30a}").unwrap(), "*");
        assert_eq!(zalgo_encode("+").unwrap(), "\u{30b}");
        assert_eq!(zalgo_decode("\u{30b}").unwrap(), "+");
        assert_eq!(zalgo_encode(",").unwrap(), "\u{30c}");
        assert_eq!(zalgo_decode("\u{30c}").unwrap(), ",");
        assert_eq!(zalgo_encode("-").unwrap(), "\u{30d}");
        assert_eq!(zalgo_decode("\u{30d}").unwrap(), "-");
        assert_eq!(zalgo_encode("\\").unwrap(), "\u{33c}");
        assert_eq!(zalgo_decode("\u{33c}").unwrap(), "\\");
        assert_eq!(zalgo_encode(".").unwrap(), "\u{30e}");
        assert_eq!(zalgo_decode("\u{30e}").unwrap(), ".");
        assert_eq!(zalgo_encode("/").unwrap(), "\u{30f}");
        assert_eq!(zalgo_decode("\u{30f}").unwrap(), "/");
        assert_eq!(zalgo_encode(":").unwrap(), "\u{31a}");
        assert_eq!(zalgo_decode("\u{31a}").unwrap(), ":");
        assert_eq!(zalgo_encode(";").unwrap(), "\u{31b}");
        assert_eq!(zalgo_decode("\u{31b}").unwrap(), ";");
        assert_eq!(zalgo_encode("<").unwrap(), "\u{31c}");
        assert_eq!(zalgo_decode("\u{31c}").unwrap(), "<");
        assert_eq!(zalgo_encode("=").unwrap(), "\u{31d}");
        assert_eq!(zalgo_decode("\u{31d}").unwrap(), "=");
        assert_eq!(zalgo_encode(">").unwrap(), "\u{31e}");
        assert_eq!(zalgo_decode("\u{31e}").unwrap(), ">");
        assert_eq!(zalgo_encode("?").unwrap(), "\u{31f}");
        assert_eq!(zalgo_decode("\u{31f}").unwrap(), "?");
        assert_eq!(zalgo_encode("@").unwrap(), "\u{320}");
        assert_eq!(zalgo_decode("\u{320}").unwrap(), "@");
        assert_eq!(zalgo_encode("\n").unwrap(), "\u{36f}");
        assert_eq!(zalgo_decode("\u{36f}").unwrap(), "\n");
    }
}
