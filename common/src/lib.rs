//! A crate for converting an ASCII text string to a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).
//! 
//! # Features
//! `files`: enabled by default and provides the functions [`encode_file`], [`decode_file`] and [`wrap_python_file`].

#![forbid(unsafe_code)]

use core::{fmt, str};
use std::error::Error;

static UNKNOWN_CHAR_MAP: &[(u8, &str)] = &[
    (0, r"Null (\0)"),
    (1, "SOH"),
    (2, "STX"),
    (3, "ETX"),
    (4, "EOT"),
    (5, "ENQ"),
    (6, "ACK"),
    (7, "BEL"),
    (8, r"Backspace (\b)"),
    (9, r"Tab (\t)"),
    (11, r"Vertical Tab (\v)"),
    (12, r"Form Feed (\f)"),
    (13, r"Carriage Return (\r)"),
    (14, "SO"),
    (15, "SI"),
    (16, "DLE"),
    (17, "DC1"),
    (18, "DC2"),
    (19, "DC3"),
    (20, "DC4"),
    (21, "NAK"),
    (22, "SYN"),
    (23, "ETB"),
    (24, "CAN"),
    (25, "EM"),
    (26, "SUB"),
    (27, "ESC"),
    (28, "FS"),
    (29, "GS"),
    (30, "RS"),
    (31, "US"),
    (127, "DEL"),
];

/// Returns the representation of the given non-printable ASCII char if it is one.
fn get_nonprintable_char_repr(character: u8) -> Option<&'static str> {
    if character < 10 {
        Some(UNKNOWN_CHAR_MAP[usize::from(character)].1)
    } else if character < 32 {
        Some(UNKNOWN_CHAR_MAP[usize::from(character) - 1].1)
    } else if character == 127 {
        Some(UNKNOWN_CHAR_MAP[31].1)
    } else {
        None
    }
}

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
pub fn zalgo_encode(string_to_compress: &str) -> Result<String, UnencodableByteError> {
    let mut line = 1;
    let mut result: Vec<u8> = vec![b'E'];

    for c in string_to_compress.bytes() {
        if c == b'\r' {
            return Err(UnencodableByteError::new(c, line));
        }

        if c == b'\n' {
            line += 1;
        }

        if !(32..=126).contains(&c) && c != b'\n' {
            return Err(UnencodableByteError::new(c, line));
        }

        let v = if c == b'\n' { 111 } else { (c - 11) % 133 - 21 };
        result.push((v >> 6) & 1 | 0b11001100);
        result.push((v & 63) | 0b10000000);
    }

    Ok(str::from_utf8(&result)
        .expect("the encoding process should not produce invalid utf8")
        .into())
}

/// Takes in a string that was encoded by [`zalgo_encode`]
/// and decodes it back into an ASCII string.
///
/// # Example
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert_eq!(zalgo_decode("É̺͇͌͏").unwrap(), "Zalgo");
/// ```
pub fn zalgo_decode(encoded: &str) -> Result<String, str::Utf8Error> {
    let bytes: Vec<u8> = encoded
        .bytes()
        .skip(1)
        .step_by(2)
        .zip(encoded.bytes().skip(2).step_by(2))
        .map(|(odds, evens)| (((odds << 6 & 64 | evens & 63) + 22) % 133 + 10))
        .collect();

    str::from_utf8(&bytes).map(|s| s.to_owned())
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

#[derive(Debug)]
/// The error returned by the encoding functions
/// if they encounter a byte they can not encode.
pub struct UnencodableByteError {
    byte: u8,
    line: usize,
}

impl UnencodableByteError {
    fn new(byte: u8, line: usize) -> Self {
        Self { byte, line }
    }

    /// Returns the number of the line on which the unencodable byte occured.
    pub fn line_number(&self) -> usize {
        self.line
    }

    /// Returns the byte value of the unencodable character. Note that this might
    /// not be the complete representation of the character in unicode, just the first
    /// byte of it.
    pub fn unencodable_character_value(&self) -> u8 {
        self.byte
    }
}

impl fmt::Display for UnencodableByteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.byte < 128 {
            match get_nonprintable_char_repr(self.byte) {
                Some(repr) => write!(f, "line {}: cannot encode {repr} character", self.line),
                None => write!(
                    f,
                    "line {}: cannot encode ASCII character #{}",
                    self.line, self.byte
                ),
            }
        } else {
            write!(f, "line {}: attempt to encode Utf-8 character sequence (this program can only encode non-control ASCII characters and newlines)", self.line)
        }
    }
}

impl Error for UnencodableByteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_unknown_char_map() {
        for i in 0_u8..10 {
            assert_eq!(
                get_nonprintable_char_repr(i).unwrap(),
                UNKNOWN_CHAR_MAP[usize::from(i)].1
            );
        }
        for i in 11_u8..32 {
            assert_eq!(
                get_nonprintable_char_repr(i).unwrap(),
                UNKNOWN_CHAR_MAP[usize::from(i - 1)].1
            );
        }
        assert_eq!(
            get_nonprintable_char_repr(127).unwrap(),
            UNKNOWN_CHAR_MAP[31].1
        );
    }
}
