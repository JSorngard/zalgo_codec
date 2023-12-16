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
//! `std` *(enabled by default)*: links the standard library and uses it to implement the [`std::error::Error`] trait for the provided [`Error`] type.
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

/// Takes in an ASCII string without control characters (except newlines)
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
///
/// # Example
///
/// Basic usage:
/// ```
/// # use zalgo_codec_common::{Error, zalgo_encode};
/// assert_eq!(zalgo_encode("Zalgo")?, "É̺͇͌͏");
/// # Ok::<(), Error>(())
/// ```
/// Can not encode ASCII control characters except newlines.
/// Notably this means that this function can not encode carriage returns,
/// which are present in e.g. line endings on Windows:
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// assert!(zalgo_encode("CRLF\r\n").is_err());
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

    for batch in string.as_bytes().chunks(BATCH_SIZE) {
        let mut buffer = [0; 2 * BATCH_SIZE];
        let mut encoded = 0;
        for byte in batch {
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
                match nonprintable_char_repr(*byte) {
                    Some(repr) => return Err(Error::UnencodableAscii(*byte, line, column, repr)),
                    None => return Err(Error::NotAscii(*byte, line, column)),
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

#[must_use = "the function returns a new value and does not modify its inputs"]
#[inline]
fn decode_byte_pair(odd: u8, even: u8) -> u8 {
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
///     // € is not an ASCII character, the first byte in its utf-8 representation is 226
///     // and it is the 22nd character on the first line in the string.
///     Err(Error::NotAscii(226, 1, 22))
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
const fn nonprintable_char_repr(byte: u8) -> Option<&'static str> {
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
