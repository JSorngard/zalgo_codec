//! A crate for converting an ASCII text string to a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).

use core::{fmt, str};
use std::error::Error;

pub mod zalgo_string;
pub use zalgo_string::ZalgoString;

/// Takes in an ASCII string without control characters (except newlines)
/// and encodes it to zalgo text using a reversible encoding scheme.
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
pub fn zalgo_encode(string_to_encode: &str) -> Result<String, ZalgoError> {
    let mut line = 1;
    let mut column = 1;
    let mut result = Vec::with_capacity(2 * string_to_encode.len() + 1);
    result.push(b'E');
    for byte in string_to_encode.bytes() {
        match nonprintable_char_repr(byte) {
            Some(repr) => return Err(ZalgoError::NonprintableAscii(byte, line, column, repr)),
            None => {
                if byte == b'\n' {
                    line += 1;
                    // Still 1-indexed since the newline will be counted at the end of the loop iteration.
                    column = 0;
                }
                if byte < 127 {
                    let v = if byte == b'\n' {
                        111
                    } else {
                        (byte - 11) % 133 - 21
                    };
                    result.push((v >> 6) & 1 | 0b11001100);
                    result.push((v & 63) | 0b10000000);
                } else {
                    return Err(ZalgoError::NotAscii(byte, line, column));
                }
            }
        }
        column += 1;
    }

    Ok(String::from_utf8(result).expect("the encoding process does not produce invalid utf8 given valid ascii text, which is verified before this point"))
}

/// Takes in a string that was encoded by [`zalgo_encode`] and decodes it back into an ASCII string.
/// Returns an error if the decoded string is not valid UTF-8.
/// This can happen if the input is a string that was not encoded by [`zalgo_encode`],
/// since the byte manipulations that this functions does could result in invalid unicode in that case.
///
/// # Example
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert_eq!(zalgo_decode("É̺͇͌͏").unwrap(), "Zalgo");
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_decode(encoded: &str) -> Result<String, std::string::FromUtf8Error> {
    let bytes: Vec<u8> = encoded
        .bytes()
        .skip(1)
        .step_by(2)
        .zip(encoded.bytes().skip(2).step_by(2))
        .map(|(odd, even)| decode_byte_pair(odd, even))
        .collect();

    String::from_utf8(bytes)
}

#[must_use = "the function returns a new value and does not modify its inputs"]
#[inline]
fn decode_byte_pair(odd: u8, even: u8) -> u8 {
    ((odd << 6 & 64 | even & 63) + 22) % 133 + 10
}

/// zalgo-encodes an ASCII string containing Python code and
/// wraps it in a decoder that decodes and executes it.
/// The resulting Python code should retain the functionality of the original.
/// # Notes
/// May not work correctly on python versions before 3.10,
/// see [this github issue](https://github.com/DaCoolOne/DumbIdeas/issues/1) for more information.
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_wrap_python(string_to_encode: &str) -> Result<String, ZalgoError> {
    let encoded_string = zalgo_encode(string_to_encode)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))"))
}

#[derive(Debug, Clone, PartialEq)]
/// The error returned by [`zalgo_encode`], [`ZalgoString::new`], and [`zalgo_wrap_python`]
/// if they encounter a byte they can not encode.
pub enum ZalgoError {
    /// Represents a valid ASCII character that is outside of the encodable set.
    NonprintableAscii(u8, usize, usize, &'static str),
    /// Represents some other unicode character.
    NotAscii(u8, usize, usize),
}

impl ZalgoError {
    /// Returns the 1-indexed line number of the line on which the unencodable byte occured.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::{ZalgoError, zalgo_encode};
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().line(), 1);
    /// assert_eq!(zalgo_encode("a\nb\nc\r\n").err().unwrap().line(), 3);
    /// ```
    #[must_use = "the method returns a new valus and does not modify `self`"]
    pub const fn line(&self) -> usize {
        match self {
            Self::NonprintableAscii(_, line, _, _) | Self::NotAscii(_, line, _) => *line,
        }
    }

    /// Returns the 1-indexed column where the unencodable byte occured.
    /// Columns are counted from left to right and the count resets for each new line.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::{ZalgoError, zalgo_encode};
    /// assert_eq!(zalgo_encode("I ❤️ 🎂").err().unwrap().column(), 3);
    /// assert_eq!(zalgo_encode("I\n❤️\n🎂").err().unwrap().column(), 1);
    /// ```
    pub const fn column(&self) -> usize {
        match self {
            Self::NonprintableAscii(_, _, column, _) | Self::NotAscii(_, _, column) => *column,
        }
    }

    /// Returns the value of the first byte of the unencodable character.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::{ZalgoError, zalgo_encode};
    /// assert_eq!(zalgo_encode("\r").err().unwrap().byte(), 13);
    /// ```
    /// Note that this might not be the complete representation of
    /// the character in unicode, just the first byte of it.
    /// ```
    /// # use zalgo_codec_common::{ZalgoError, zalgo_encode};
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().byte(), 226);
    /// // Even though
    /// assert_eq!("❤️".as_bytes(), &[226, 157, 164, 239, 184, 143])
    /// ```
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn byte(&self) -> u8 {
        match self {
            Self::NonprintableAscii(byte, _, _, _) | Self::NotAscii(byte, _, _) => *byte,
        }
    }

    /// Return a representation of the unencodable byte.
    /// This exists if the character is an unencodable ASCII character.
    /// If it is some other unicode character we only know its first byte, so we can not
    /// accurately represent it.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("\r").err().unwrap().representation(), Some("Carriage Return"));
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().representation(), None);
    /// ```
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn representation(&self) -> Option<&'static str> {
        match self {
            Self::NonprintableAscii(_, _, _, repr) => Some(*repr),
            Self::NotAscii(_, _, _) => None,
        }
    }
}

impl fmt::Display for ZalgoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NonprintableAscii(byte, line, column, repr) => write!(
                f,
                "line {line} at column {column}: can not encode ASCII \"{repr}\" character with byte value {byte}"
            ),
            Self::NotAscii(byte, line, column) => write!(
                f,
                "line {line} at column {column}: byte value {byte} does not correspond to an ASCII character"
            ),
        }
    }
}

impl Error for ZalgoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Returns the representation of the given ASCII byte if it's not printable.
#[must_use = "the function returns a new value and does not modify the input"]
const fn nonprintable_char_repr(byte: u8) -> Option<&'static str> {
    if byte < 10 {
        first_ten(byte as usize)
    } else if byte >= 11 && byte < 32 {
        other_21(byte as usize)
    } else if byte == 127 {
        last_one()
    } else {
        None
    }
}

#[cold]
const fn first_ten(index: usize) -> Option<&'static str> {
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
        ][index],
    )
}

#[cold]
const fn other_21(index: usize) -> Option<&'static str> {
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
        ][index - 11],
    )
}

#[cold]
const fn last_one() -> Option<&'static str> {
    Some("Delete")
}
