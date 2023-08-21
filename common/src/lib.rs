//! A crate for converting an ASCII text string to a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).

use core::{fmt, str};

/// Contains the implementation of [`ZalgoString`](zalgo_string::ZalgoString) as well as related iterators.
pub mod zalgo_string;

pub use zalgo_string::ZalgoString;

/// Takes in an ASCII string without control characters (except newlines)
/// and encodes it into a single grapheme cluster using a reversible encoding scheme.
///
/// The resulting string is a single unicode grapheme cluster and should
/// only take up a single character space horizontally when displayed
/// (though this can vary between platforms depending on how they deal with unicode).
/// The resulting string will be ~2 times larger than the original in terms of bytes, and it
/// can be decoded to recover the original string with [`zalgo_decode`].
/// # Example
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// assert_eq!(zalgo_encode("Zalgo").unwrap(), "É̺͇͌͏");
/// ```
/// # Notes
/// Can not encode carriage returns, present in e.g. line endings on Windows.
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_encode(string_to_encode: &str) -> Result<String, Error> {
    let mut line = 1;
    let mut column = 1;
    let mut result = Vec::with_capacity(2 * string_to_encode.len() + 1);
    result.push(b'E');
    const BATCH_SIZE: usize = 16;
    for bytes in string_to_encode.as_bytes().chunks(BATCH_SIZE) {
        let mut batch = [0u8; 2 * BATCH_SIZE];
        let mut i = 0;
        for byte in bytes {
            // Only encode ASCII bytes corresponding to printable characters or newlines.
            if (32..127).contains(byte) || *byte == b'\n' {
                if *byte == b'\n' {
                    line += 1;
                    // `column` is still 1-indexed since it gets incremented at the end of the current loop iteration.
                    column = 0;
                }

                let v = ((i16::from(*byte) - 11).rem_euclid(133) - 21) as u8;
                batch[i] = (v >> 6) & 1 | 0b11001100;
                batch[i + 1] = (v & 63) | 0b10000000;
                i += 2;
            } else {
                match nonprintable_char_repr(*byte) {
                    Some(repr) => return Err(Error::NonprintableAscii(*byte, line, column, repr)),
                    None => return Err(Error::NotAscii(*byte, line, column)),
                }
            }
            column += 1;
        }
        result.extend_from_slice(&batch[..i]);
    }

    // Safety: the encoding process does not produce invalid UTF-8
    // if given valid printable ASCII + newlines,
    // which is checked before this point
    Ok(unsafe { String::from_utf8_unchecked(result) })
}

#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_encode_2(string_to_encode: &str) -> Result<String, Error> {
    let mut line = 1;
    let mut column = 1;
    let mut result = Vec::with_capacity(2 * string_to_encode.len() + 1);
    result.push(b'E');
    for byte in string_to_encode.bytes() {
        // Only encode ASCII bytes corresponding to printable characters or newlines.
        if (32..127).contains(&byte) || byte == b'\n' {
            if byte == b'\n' {
                line += 1;
                // `column` is still 1-indexed since it gets incremented at the end of the current loop iteration.
                column = 0;
            }

            let v = ((i16::from(byte) - 11).rem_euclid(133) - 21) as u8;
            result.push((v >> 6) & 1 | 0b11001100);
            result.push((v & 63) | 0b10000000);
        } else {
            match nonprintable_char_repr(byte) {
                Some(repr) => return Err(Error::NonprintableAscii(byte, line, column, repr)),
                None => return Err(Error::NotAscii(byte, line, column)),
            }
        }
        column += 1;
    }

    // Safety: the encoding process does not produce invalid UTF-8
    // if given valid printable ASCII + newlines,
    // which is checked before this point
    Ok(unsafe { String::from_utf8_unchecked(result) })
}

/// Takes in a string that was encoded by [`zalgo_encode`] and decodes it back into an ASCII string.
///
/// Returns an error if the decoded string is not valid UTF-8.
/// This can happen if the input is a string that was not encoded by [`zalgo_encode`],
/// since the byte manipulations that this function does could result in invalid unicode in that case.
/// If you want to be able to decode without this check, consider using a [`ZalgoString`].
///
/// # Examples
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert_eq!(zalgo_decode("É̺͇͌͏").unwrap(), "Zalgo");
/// ```
/// Decoding arbitrary strings will most likely lead to errors:
/// ```
/// # use zalgo_codec_common::zalgo_decode;
/// assert!(zalgo_decode("Shmårgl").is_err());
/// ```
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_decode(encoded: &str) -> Result<String, std::string::FromUtf8Error> {
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
/// # Notes
/// May not work correctly on python versions before 3.10,
/// see [this github issue](https://github.com/DaCoolOne/DumbIdeas/issues/1) for more information.
#[must_use = "the function returns a new value and does not modify the input"]
pub fn zalgo_wrap_python(string_to_encode: &str) -> Result<String, Error> {
    let encoded_string = zalgo_encode(string_to_encode)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))"))
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The error returned by [`zalgo_encode`], [`ZalgoString::new`], and [`zalgo_wrap_python`]
/// if they encounter a byte they can not encode.
pub enum Error {
    /// Represents a valid ASCII character that is outside of the encodable set.
    NonprintableAscii(u8, usize, usize, &'static str),
    /// Represents some other unicode character.
    NotAscii(u8, usize, usize),
}

impl Error {
    /// Returns the 1-indexed line number of the line on which the unencodable byte occured.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
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
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("I ❤️ 🎂").err().unwrap().column(), 3);
    /// assert_eq!(zalgo_encode("I\n❤️\n🎂").err().unwrap().column(), 1);
    /// ```
    #[must_use = "the method returns a new valus and does not modify `self`"]
    pub const fn column(&self) -> usize {
        match self {
            Self::NonprintableAscii(_, _, column, _) | Self::NotAscii(_, _, column) => *column,
        }
    }

    /// Returns the value of the first byte of the unencodable character.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("\r").err().unwrap().byte(), 13);
    /// ```
    /// Note that this might not be the complete representation of
    /// the character in unicode, just the first byte of it.
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
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

impl fmt::Display for Error {
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

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
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
