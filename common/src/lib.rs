//! A crate for converting an ASCII text string to a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).
//!
//! # Features
//! `files`: enabled by default and provides the functions [`encode_file`], [`decode_file`] and [`wrap_python_file`].

#![forbid(unsafe_code)]

use core::{fmt, str};
use std::error::Error;

#[cfg(any(doc, feature = "files"))]
mod files;

#[cfg(any(doc, feature = "files"))]
pub use files::*;

/// Takes in an ASCII string without control characters (except newlines)
/// and "compresses" it to zalgo text using a reversible encoding scheme.
/// The resulting string is a single unicode grapheme cluster and should
/// only take up a single character space horizontally when displayed
/// (though this can vary between platforms depending on how they deal with unicode).
/// The resulting string will be ~2 times larger than the original in terms of bytes, and it
/// can be decoded to recover the original string using [`zalgo_decode`].
/// # Example
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// assert_eq!(zalgo_encode("Zalgo").unwrap(), "É̺͇͌͏");
/// ```
/// # Notes
/// Can not encode carriage returns, present in e.g. line endings on Windows.
pub fn zalgo_encode(string_to_encode: &str) -> Result<String, UnencodableByteError> {
    let mut line = 1;
    let mut result = Vec::<u8>::with_capacity(2 * string_to_encode.len() + 1);
    result.push(b'E');

    for c in string_to_encode.bytes() {
        if !(32..=126).contains(&c) && c != b'\n' {
            return Err(UnencodableByteError::new(c, line));
        }

        if c == b'\n' {
            line += 1;
        }

        let v = if c == b'\n' { 111 } else { (c - 11) % 133 - 21 };
        result.push((v >> 6) & 1 | 0b11001100);
        result.push((v & 63) | 0b10000000);
    }

    Ok(String::from_utf8(result).expect("the encoding process should not produce invalid utf8"))
}

/// Takes in a string that was encoded by [`zalgo_encode`]
/// and decodes it back into an ASCII string.
///
/// # Example
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert_eq!(zalgo_decode("É̺͇͌͏").unwrap(), "Zalgo");
/// ```
pub fn zalgo_decode(encoded: &str) -> Result<String, std::string::FromUtf8Error> {
    let bytes: Vec<u8> = encoded
        .bytes()
        .skip(1)
        .step_by(2)
        .zip(encoded.bytes().skip(2).step_by(2))
        .map(|(odds, evens)| (((odds << 6 & 64 | evens & 63) + 22) % 133 + 10))
        .collect();

    String::from_utf8(bytes)
}

/// zalgo-encodes an ASCII string containing Python code and
/// wraps it in a decoder that decodes and executes it.
/// The resulting Python code should retain the functionality of the original.
/// # Notes
/// May not work correctly on python versions before 3.10,
/// see [this github issue](https://github.com/DaCoolOne/DumbIdeas/issues/1) for more information.
pub fn zalgo_wrap_python(string_to_encode: &str) -> Result<String, UnencodableByteError> {
    let encoded_string = zalgo_encode(string_to_encode)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))"))
}

#[derive(Debug, Clone)]
/// The error returned by the encoding functions
/// if they encounter a byte they can not encode.
pub enum UnencodableByteError {
    NonprintableAscii(u8, usize, Option<&'static str>),
    NotAscii(u8, usize),
}

impl UnencodableByteError {
    const fn new(byte: u8, line: usize) -> Self {
        if byte < 128 {
            Self::NonprintableAscii(byte, line, get_nonprintable_char_repr(byte))
        } else {
            Self::NotAscii(byte, line)
        }
    }

    /// Returns the (1-indexed) line number of the line on which the unencodable byte occured.
    pub const fn line(&self) -> usize {
        match self {
            Self::NonprintableAscii(_, line, _) | Self::NotAscii(_, line) => *line,
        }
    }

    /// Returns the byte value of the unencodable character. Note that this might
    /// not be the complete representation of the character in unicode, just the first
    /// byte of it.
    pub const fn byte(&self) -> u8 {
        match self {
            Self::NonprintableAscii(byte, _, _) | Self::NotAscii(byte, _) => *byte,
        }
    }

    /// Return a representation of the unencodable byte if there is one.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("\r").err().unwrap().representation(), Some("Carriage Return"));
    /// assert_eq!(zalgo_encode("❤").err().unwrap().representation(), None);
    /// ```
    pub const fn representation(&self) -> Option<&'static str> {
        match self {
            Self::NonprintableAscii(_, _, repr) => *repr,
            Self::NotAscii(_, _) => None,
        }
    }
}

impl fmt::Display for UnencodableByteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NonprintableAscii(byte, line, r) => match r {
                Some(repr) => write!(
                    f,
                    "line {line}: can not encode ASCII \"{repr}\" characters with byte value {byte}"
                ),
                None => write!(
                    f,
                    "line {line}: could not encode ASCII character with byte value {byte}"
                ),
            },
            Self::NotAscii(byte, line) => write!(
                f,
                "line {line}: byte value {byte} does not correspond to an ASCII character"
            ),
        }
    }
}

impl Error for UnencodableByteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Returns the representation of the given ASCII byte if it's not printable.
const fn get_nonprintable_char_repr(byte: u8) -> Option<&'static str> {
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
