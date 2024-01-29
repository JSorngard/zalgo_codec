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
//! # use zalgo_codec_common::{Error, zalgo_encode};
//! let s = "Zalgo";
//! let encoded = zalgo_encode(s)?;
//! assert_eq!(encoded, "É̺͇͌͏");
//! # Ok::<(), Error>(())
//! ```
//! Decode a grapheme cluster back into a string:
//! ```
//! # use zalgo_codec_common::zalgo_decode;
//! # use std::string::FromUtf8Error;
//! let encoded = "É̺͇͌͏";
//! let s = zalgo_decode(encoded)?;
//! assert_eq!(s, "Zalgo");
//! # Ok::<(), FromUtf8Error>(())
//! ```
//! The [`ZalgoString`] type can be used to encode a string and handle the result in various ways:
//! ```
//! # use zalgo_codec_common::{ZalgoString, Error};
//! let s = "Zalgo";
//! let zstr = ZalgoString::new(s)?;
//!
//! // Implements PartialEq with common string types
//! assert_eq!(zstr, "É̺͇͌͏");
//!
//! // Utility functions
//! assert_eq!(zstr.len(), 2 * s.len() + 1);
//! assert_eq!(zstr.decoded_len(), s.len());
//!
//! // Iterate over bytes and chars, in both encoded and decoded form
//! assert_eq!(zstr.bytes().next(), Some(69));
//! assert_eq!(zstr.decoded_bytes().nth_back(2), Some(b'l'));
//! assert_eq!(zstr.chars().nth(1), Some('\u{33a}'));
//! assert_eq!(zstr.decoded_chars().next_back(), Some('o'));
//!
//! // Decode inplace
//! assert_eq!(zstr.into_decoded_string(), "Zalgo");
//! # Ok::<(), Error>(())
//! ```
//!
//! # Features
//!
//! `std` *(enabled by default)*: implements the [`std::error::Error`] trait for the provided [`Error`] type.
//! If this feature is not enabled the library is `#[no_std]`, but still uses the `alloc` crate.
//!
//! `serde`: implements the [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize) traits
//! from [`serde`](https://crates.io/crates/serde) for [`ZalgoString`].
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
//! You can optionally enable the `gui` feature during installation to include a rudimentary GUI mode for the program.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{FromUtf8Error, String},
    vec,
    vec::Vec,
};
use core::{fmt, str};
#[cfg(feature = "std")]
use std::string::FromUtf8Error;

mod error;
pub mod zalgo_string;

pub use error::Error;
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
/// # use zalgo_codec_common::{Error, zalgo_encode};
/// assert_eq!(zalgo_encode("Zalgo")?, "É̺͇͌͏");
/// # Ok::<(), Error>(())
/// ```
/// Can not encode non-ASCII characters or ASCII control characters except newlines:
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// assert!(zalgo_encode("Windows line ending: \r\n").is_err());
/// assert!(zalgo_encode("Zålgö").is_err());
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_encode(string: &str) -> Result<String, Error> {
    // We will encode this many bytes at a time before pushing onto the result vector.
    const BATCH_SIZE: usize = 16;

    // The line we are currently encoding
    let mut line = 1;
    // The column on that line we are currently encoding
    let mut column = 1;
    // These are used for reporting a useful error if the encoding process fails.

    // Every byte in the input will encode to two bytes. The extra byte is for the initial letter
    // which is there in order for the output to be displayable in an intuitive way.
    let mut result = Vec::with_capacity(2 * string.len() + 1);
    result.push(b'E');

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
                match nonprintable_ascii_repr(*byte) {
                    Some(repr) => return Err(Error::UnencodableAscii(*byte, line, column, repr)),
                    None => {
                        // The panic should never trigger since we know that string[i*BATCH_SIZE + j]
                        // has some value which is stored in `byte`, and that this value is the first
                        // byte of a non-ascii character and that Strings in Rust are valid utf-8.
                        // All of this means that the value that starts at this index is a utf-8 encoded
                        // character, which `chars.next()` will extract.
                        let char = string[i*BATCH_SIZE + j..].chars().next()
                            .expect("i*BATCH_SIZE + j is within the string and on a char boundary, so string.chars().next() should find a char");
                        return Err(Error::NotAscii(char, line, column));
                    }
                }
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
/// This can happen if the input is a string that was not encoded by [`zalgo_encode`],
/// since the byte manipulations that this function does could result in invalid unicode in that case.
/// Even if no error is returned in such a case the results are not meaningful.
/// If you want to be able to decode without this check, consider using a [`ZalgoString`].
///
/// # Examples
///
/// Basic usage:
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// # use std::string::FromUtf8Error;
/// assert_eq!(zalgo_decode("É̺͇͌͏")?, "Zalgo");
/// # Ok::<(), FromUtf8Error>(())
/// ```
/// Decoding arbitrary strings that were not produced by [`zalgo_encode`] will most likely lead to errors:
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert!(zalgo_decode("Zalgo").is_err());
/// ```
/// If it doesn't the results are not meaningful:
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert_eq!(zalgo_decode("awö")?, "c");
/// # Ok::<(), std::string::FromUtf8Error>(())
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_decode(encoded: &str) -> Result<String, FromUtf8Error> {
    let mut res = vec![0; (encoded.len() - 1) / 2];
    let bytes = encoded.as_bytes();

    for (write, read) in (1..encoded.len()).step_by(2).enumerate() {
        match bytes.get(read + 1) {
            Some(next) => res[write] = decode_byte_pair(bytes[read], *next),
            None => break,
        }
    }

    String::from_utf8(res)
}

#[inline]
#[must_use = "the function returns a new value and does not modify its inputs"]
const fn decode_byte_pair(odd: u8, even: u8) -> u8 {
    ((odd << 6 & 64 | even & 63) + 22) % 133 + 10
}

/// zalgo-encodes an ASCII string containing Python code and
/// wraps it in a decoder that decodes and executes it.
/// The resulting Python code should retain the functionality of the original.
///
/// # Example
///
/// Encode a simple hello world program in Python
/// ```
/// # use zalgo_codec_common::{Error, zalgo_wrap_python};
/// let py_hello_world = "print(\"Hello, world!\")\n";
/// let py_hello_world_enc = zalgo_wrap_python(py_hello_world)?;
/// assert_eq!(
///     py_hello_world_enc,
///     "b='Ę͉͎͔͐͒̈̂͌͌ͅ͏̌̀͗͏͒͌̈́́̂̉ͯ'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))",
/// );
/// # Ok::<(), Error>(())
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
/// # use zalgo_codec_common::{Error, zalgo_wrap_python};
/// assert_eq!(
///     zalgo_wrap_python(r#"print("That will be 5€ please")"#),
///     Err(Error::NotAscii('€', 1, 22))
/// );
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_wrap_python(python: &str) -> Result<String, Error> {
    let encoded_string = zalgo_encode(python)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))"))
}

/// Returns the representation of the given ASCII byte if it's not printable.
#[inline]
#[must_use = "the function returns a new value and does not modify the input"]
const fn nonprintable_ascii_repr(byte: u8) -> Option<&'static str> {
    if byte < 10 {
        Some(
            [
                "Null",
                "Start Of Heading",
                "Start Of Text",
                "End Of Text",
                "End Of Transmission",
                "Enquiry",
                "Acknowledge",
                "Bell",
                "Backspace",
                "Horizontal Tab",
            ][byte as usize],
        )
    } else if byte >= 11 && byte < 32 {
        Some(
            [
                "Vertical Tab",
                "Form Feed",
                "Carriage Return",
                "Shift Out",
                "Shift In",
                "Data Link Escape",
                "Data Control 1",
                "Data Control 2",
                "Data Control 3",
                "Data Control 4",
                "Negative Acknowledge",
                "Synchronous Idle",
                "End Of Transmission Block",
                "Cancel",
                "End Of Medium",
                "Substitute",
                "Escape",
                "File Separator",
                "Group Separator",
                "Record Separator",
                "Unit Separator",
            ][byte as usize - 11],
        )
    } else if byte == 127 {
        Some("Delete")
    } else {
        None
    }
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
    fn verify_conversion_table() {
        assert_eq!(zalgo_encode("A").unwrap(), "E\u{321}");
        assert_eq!(zalgo_decode("E\u{321}").unwrap(), "A");
        assert_eq!(zalgo_encode("B").unwrap(), "E\u{322}");
        assert_eq!(zalgo_decode("E\u{322}").unwrap(), "B");
        assert_eq!(zalgo_encode("C").unwrap(), "E\u{323}");
        assert_eq!(zalgo_decode("E\u{323}").unwrap(), "C");
        assert_eq!(zalgo_encode("D").unwrap(), "E\u{324}");
        assert_eq!(zalgo_decode("E\u{324}").unwrap(), "D");
        assert_eq!(zalgo_encode("E").unwrap(), "E\u{325}");
        assert_eq!(zalgo_decode("E\u{325}").unwrap(), "E");
        assert_eq!(zalgo_encode("F").unwrap(), "E\u{326}");
        assert_eq!(zalgo_decode("E\u{326}").unwrap(), "F");
        assert_eq!(zalgo_encode("G").unwrap(), "E\u{327}");
        assert_eq!(zalgo_decode("E\u{327}").unwrap(), "G");
        assert_eq!(zalgo_encode("H").unwrap(), "E\u{328}");
        assert_eq!(zalgo_decode("E\u{328}").unwrap(), "H");
        assert_eq!(zalgo_encode("I").unwrap(), "E\u{329}");
        assert_eq!(zalgo_decode("E\u{329}").unwrap(), "I");
        assert_eq!(zalgo_encode("J").unwrap(), "E\u{32a}");
        assert_eq!(zalgo_decode("E\u{32a}").unwrap(), "J");
        assert_eq!(zalgo_encode("K").unwrap(), "E\u{32b}");
        assert_eq!(zalgo_decode("E\u{32b}").unwrap(), "K");
        assert_eq!(zalgo_encode("L").unwrap(), "E\u{32c}");
        assert_eq!(zalgo_decode("E\u{32c}").unwrap(), "L");
        assert_eq!(zalgo_encode("M").unwrap(), "E\u{32d}");
        assert_eq!(zalgo_decode("E\u{32d}").unwrap(), "M");
        assert_eq!(zalgo_encode("N").unwrap(), "E\u{32e}");
        assert_eq!(zalgo_decode("E\u{32e}").unwrap(), "N");
        assert_eq!(zalgo_encode("O").unwrap(), "E\u{32f}");
        assert_eq!(zalgo_decode("E\u{32f}").unwrap(), "O");
        assert_eq!(zalgo_encode("P").unwrap(), "E\u{330}");
        assert_eq!(zalgo_decode("E\u{330}").unwrap(), "P");
        assert_eq!(zalgo_encode("Q").unwrap(), "E\u{331}");
        assert_eq!(zalgo_decode("E\u{331}").unwrap(), "Q");
        assert_eq!(zalgo_encode("R").unwrap(), "E\u{332}");
        assert_eq!(zalgo_decode("E\u{332}").unwrap(), "R");
        assert_eq!(zalgo_encode("S").unwrap(), "E\u{333}");
        assert_eq!(zalgo_decode("E\u{333}").unwrap(), "S");
        assert_eq!(zalgo_encode("T").unwrap(), "E\u{334}");
        assert_eq!(zalgo_decode("E\u{334}").unwrap(), "T");
        assert_eq!(zalgo_encode("U").unwrap(), "E\u{335}");
        assert_eq!(zalgo_decode("E\u{335}").unwrap(), "U");
        assert_eq!(zalgo_encode("V").unwrap(), "E\u{336}");
        assert_eq!(zalgo_decode("E\u{336}").unwrap(), "V");
        assert_eq!(zalgo_encode("W").unwrap(), "E\u{337}");
        assert_eq!(zalgo_decode("E\u{337}").unwrap(), "W");
        assert_eq!(zalgo_encode("X").unwrap(), "E\u{338}");
        assert_eq!(zalgo_decode("E\u{338}").unwrap(), "X");
        assert_eq!(zalgo_encode("Y").unwrap(), "E\u{339}");
        assert_eq!(zalgo_decode("E\u{339}").unwrap(), "Y");
        assert_eq!(zalgo_encode("Z").unwrap(), "E\u{33a}");
        assert_eq!(zalgo_decode("E\u{33a}").unwrap(), "Z");
        assert_eq!(zalgo_encode("a").unwrap(), "E\u{341}");
        assert_eq!(zalgo_decode("E\u{341}").unwrap(), "a");
        assert_eq!(zalgo_encode("b").unwrap(), "E\u{342}");
        assert_eq!(zalgo_decode("E\u{342}").unwrap(), "b");
        assert_eq!(zalgo_encode("c").unwrap(), "E\u{343}");
        assert_eq!(zalgo_decode("E\u{343}").unwrap(), "c");
        assert_eq!(zalgo_encode("d").unwrap(), "E\u{344}");
        assert_eq!(zalgo_decode("E\u{344}").unwrap(), "d");
        assert_eq!(zalgo_encode("e").unwrap(), "E\u{345}");
        assert_eq!(zalgo_decode("E\u{345}").unwrap(), "e");
        assert_eq!(zalgo_encode("f").unwrap(), "E\u{346}");
        assert_eq!(zalgo_decode("E\u{346}").unwrap(), "f");
        assert_eq!(zalgo_encode("g").unwrap(), "E\u{347}");
        assert_eq!(zalgo_decode("E\u{347}").unwrap(), "g");
        assert_eq!(zalgo_encode("h").unwrap(), "E\u{348}");
        assert_eq!(zalgo_decode("E\u{348}").unwrap(), "h");
        assert_eq!(zalgo_encode("i").unwrap(), "E\u{349}");
        assert_eq!(zalgo_decode("E\u{349}").unwrap(), "i");
        assert_eq!(zalgo_encode("j").unwrap(), "E\u{34a}");
        assert_eq!(zalgo_decode("E\u{34a}").unwrap(), "j");
        assert_eq!(zalgo_encode("k").unwrap(), "E\u{34b}");
        assert_eq!(zalgo_decode("E\u{34b}").unwrap(), "k");
        assert_eq!(zalgo_encode("l").unwrap(), "E\u{34c}");
        assert_eq!(zalgo_decode("E\u{34c}").unwrap(), "l");
        assert_eq!(zalgo_encode("m").unwrap(), "E\u{34d}");
        assert_eq!(zalgo_decode("E\u{34d}").unwrap(), "m");
        assert_eq!(zalgo_encode("n").unwrap(), "E\u{34e}");
        assert_eq!(zalgo_decode("E\u{34e}").unwrap(), "n");
        assert_eq!(zalgo_encode("o").unwrap(), "E\u{34f}");
        assert_eq!(zalgo_decode("E\u{34f}").unwrap(), "o");
        assert_eq!(zalgo_encode("p").unwrap(), "E\u{350}");
        assert_eq!(zalgo_decode("E\u{350}").unwrap(), "p");
        assert_eq!(zalgo_encode("q").unwrap(), "E\u{351}");
        assert_eq!(zalgo_decode("E\u{351}").unwrap(), "q");
        assert_eq!(zalgo_encode("r").unwrap(), "E\u{352}");
        assert_eq!(zalgo_decode("E\u{352}").unwrap(), "r");
        assert_eq!(zalgo_encode("s").unwrap(), "E\u{353}");
        assert_eq!(zalgo_decode("E\u{353}").unwrap(), "s");
        assert_eq!(zalgo_encode("t").unwrap(), "E\u{354}");
        assert_eq!(zalgo_decode("E\u{354}").unwrap(), "t");
        assert_eq!(zalgo_encode("u").unwrap(), "E\u{355}");
        assert_eq!(zalgo_decode("E\u{355}").unwrap(), "u");
        assert_eq!(zalgo_encode("v").unwrap(), "E\u{356}");
        assert_eq!(zalgo_decode("E\u{356}").unwrap(), "v");
        assert_eq!(zalgo_encode("w").unwrap(), "E\u{357}");
        assert_eq!(zalgo_decode("E\u{357}").unwrap(), "w");
        assert_eq!(zalgo_encode("x").unwrap(), "E\u{358}");
        assert_eq!(zalgo_decode("E\u{358}").unwrap(), "x");
        assert_eq!(zalgo_encode("y").unwrap(), "E\u{359}");
        assert_eq!(zalgo_decode("E\u{359}").unwrap(), "y");
        assert_eq!(zalgo_encode("z").unwrap(), "E\u{35a}");
        assert_eq!(zalgo_decode("E\u{35a}").unwrap(), "z");
        assert_eq!(zalgo_encode("1").unwrap(), "E\u{311}");
        assert_eq!(zalgo_decode("E\u{311}").unwrap(), "1");
        assert_eq!(zalgo_encode("2").unwrap(), "E\u{312}");
        assert_eq!(zalgo_decode("E\u{312}").unwrap(), "2");
        assert_eq!(zalgo_encode("3").unwrap(), "E\u{313}");
        assert_eq!(zalgo_decode("E\u{313}").unwrap(), "3");
        assert_eq!(zalgo_encode("4").unwrap(), "E\u{314}");
        assert_eq!(zalgo_decode("E\u{314}").unwrap(), "4");
        assert_eq!(zalgo_encode("5").unwrap(), "E\u{315}");
        assert_eq!(zalgo_decode("E\u{315}").unwrap(), "5");
        assert_eq!(zalgo_encode("6").unwrap(), "E\u{316}");
        assert_eq!(zalgo_decode("E\u{316}").unwrap(), "6");
        assert_eq!(zalgo_encode("7").unwrap(), "E\u{317}");
        assert_eq!(zalgo_decode("E\u{317}").unwrap(), "7");
        assert_eq!(zalgo_encode("8").unwrap(), "E\u{318}");
        assert_eq!(zalgo_decode("E\u{318}").unwrap(), "8");
        assert_eq!(zalgo_encode("9").unwrap(), "E\u{319}");
        assert_eq!(zalgo_decode("E\u{319}").unwrap(), "9");
        assert_eq!(zalgo_encode("0").unwrap(), "E\u{310}");
        assert_eq!(zalgo_decode("E\u{310}").unwrap(), "0");
        assert_eq!(zalgo_encode(" ").unwrap(), "E\u{300}");
        assert_eq!(zalgo_decode("E\u{300}").unwrap(), " ");
        assert_eq!(zalgo_encode("!").unwrap(), "E\u{301}");
        assert_eq!(zalgo_decode("E\u{301}").unwrap(), "!");
        assert_eq!(zalgo_encode("\"").unwrap(), "E\u{302}");
        assert_eq!(zalgo_decode("E\u{302}").unwrap(), "\"");
        assert_eq!(zalgo_encode("#").unwrap(), "E\u{303}");
        assert_eq!(zalgo_decode("E\u{303}").unwrap(), "#");
        assert_eq!(zalgo_encode("$").unwrap(), "E\u{304}");
        assert_eq!(zalgo_decode("E\u{304}").unwrap(), "$");
        assert_eq!(zalgo_encode("%").unwrap(), "E\u{305}");
        assert_eq!(zalgo_decode("E\u{305}").unwrap(), "%");
        assert_eq!(zalgo_encode("&").unwrap(), "E\u{306}");
        assert_eq!(zalgo_decode("E\u{306}").unwrap(), "&");
        assert_eq!(zalgo_encode("'").unwrap(), "E\u{307}");
        assert_eq!(zalgo_decode("E\u{307}").unwrap(), "'");
        assert_eq!(zalgo_encode("(").unwrap(), "E\u{308}");
        assert_eq!(zalgo_decode("E\u{308}").unwrap(), "(");
        assert_eq!(zalgo_encode(")").unwrap(), "E\u{309}");
        assert_eq!(zalgo_decode("E\u{309}").unwrap(), ")");
        assert_eq!(zalgo_encode("*").unwrap(), "E\u{30a}");
        assert_eq!(zalgo_decode("E\u{30a}").unwrap(), "*");
        assert_eq!(zalgo_encode("+").unwrap(), "E\u{30b}");
        assert_eq!(zalgo_decode("E\u{30b}").unwrap(), "+");
        assert_eq!(zalgo_encode(",").unwrap(), "E\u{30c}");
        assert_eq!(zalgo_decode("E\u{30c}").unwrap(), ",");
        assert_eq!(zalgo_encode("-").unwrap(), "E\u{30d}");
        assert_eq!(zalgo_decode("E\u{30d}").unwrap(), "-");
        assert_eq!(zalgo_encode("\\").unwrap(), "E\u{33c}");
        assert_eq!(zalgo_decode("E\u{33c}").unwrap(), "\\");
        assert_eq!(zalgo_encode(".").unwrap(), "E\u{30e}");
        assert_eq!(zalgo_decode("E\u{30e}").unwrap(), ".");
        assert_eq!(zalgo_encode("/").unwrap(), "E\u{30f}");
        assert_eq!(zalgo_decode("E\u{30f}").unwrap(), "/");
        assert_eq!(zalgo_encode(":").unwrap(), "E\u{31a}");
        assert_eq!(zalgo_decode("E\u{31a}").unwrap(), ":");
        assert_eq!(zalgo_encode(";").unwrap(), "E\u{31b}");
        assert_eq!(zalgo_decode("E\u{31b}").unwrap(), ";");
        assert_eq!(zalgo_encode("<").unwrap(), "E\u{31c}");
        assert_eq!(zalgo_decode("E\u{31c}").unwrap(), "<");
        assert_eq!(zalgo_encode("=").unwrap(), "E\u{31d}");
        assert_eq!(zalgo_decode("E\u{31d}").unwrap(), "=");
        assert_eq!(zalgo_encode(">").unwrap(), "E\u{31e}");
        assert_eq!(zalgo_decode("E\u{31e}").unwrap(), ">");
        assert_eq!(zalgo_encode("?").unwrap(), "E\u{31f}");
        assert_eq!(zalgo_decode("E\u{31f}").unwrap(), "?");
        assert_eq!(zalgo_encode("@").unwrap(), "E\u{320}");
        assert_eq!(zalgo_decode("E\u{320}").unwrap(), "@");
        assert_eq!(zalgo_encode("\n").unwrap(), "E\u{36f}");
        assert_eq!(zalgo_decode("E\u{36f}").unwrap(), "\n");
    }
}
