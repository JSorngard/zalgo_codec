//! A crate for converting an ASCII text string to a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).
//! 
use core::{fmt, str};
use std::error::Error;

pub use zalgo_string::ZalgoString;
mod zalgo_string {
    use super::{fmt, zalgo_encode, ZalgoError};
    use core::convert;
    #[cfg(feature = "serde_support")]
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Hash)]
    #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
    pub struct ZalgoString(String);

    impl ZalgoString {
        /// Encodes the given string as a new `ZalgoString`
        #[must_use = "the function returns a new `ZalgoString` and the input is dropped"]
        pub fn encode(s: String) -> Result<Self, ZalgoError> {
            zalgo_encode(&s).map(Self)
        }

        /// Decodes `self` into a normal `String` in-place. This method has no effect on the allocated capacity.
        #[must_use = "`self` will be dropped if the result is not used"]
        pub fn decode(self) -> String {
            let mut w = 0;
            let mut bytes = self.into_bytes();
            for r in (1..bytes.len()).step_by(2) {
                bytes[w] = ((bytes[r] << 6 & 64 | bytes[r + 1] & 63) + 22) % 133 + 10;
                w += 1;
            }
            bytes.truncate(w);
            // Safety: we know that the starting string was encoded from valid ASCII to begin with
            unsafe {
                String::from_utf8_unchecked(bytes)
            }
        }

        /// Extracts a string slice containing the entire `ZalgoString`.
        #[inline]
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }

        /// Returns the contents of the `ZalgoString` as a byte slice.
        #[inline]
        #[must_use]
        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_bytes()
        }

        /// Returns the length of the `ZalgoString` in bytes.
        #[inline]
        #[must_use]
        pub fn len(&self) -> usize {
            self.0.len()
        }

        /// Returns an iterator over the bytes of the ZalgoString. See [`core::str::bytes`](https://doc.rust-lang.org/1.70.0/core/primitive.str.html#method.bytes) for more information.
        #[inline]
        pub fn bytes(&self) -> core::str::Bytes<'_> {
            self.0.bytes()
        }

        /// Returns an iterator over the characters of the ZalgoString. For a `ZalgoString` the characters are the different accents and zero-width joiners that make it up.
        /// See [`core::str::chars`](https://doc.rust-lang.org/1.70.0/core/primitive.str.html#method.chars) for more information.
        #[inline]
        pub fn chars(&self) -> core::str::Chars<'_> {
            self.0.chars()
        }

        /// Converts a `ZalgoString` into a byte vector.
        /// This consumes the `ZalgoString`, so we do not need to copy its contents.
        #[inline]
        #[must_use = "`self` will be dropped if the result is not used"]
        pub fn into_bytes(self) -> Vec<u8> {
            self.0.into_bytes()
        }
    }

    impl convert::From<ZalgoString> for String {
        /// Converts the `ZalgoString` into a `String` *without decoding it*.
        #[inline]
        fn from(zs: ZalgoString) -> Self {
            zs.0
        }
    }

    impl<'a> convert::From<&'a ZalgoString> for &'a str {
        /// Converts a `&ZalgoString` to a `&str` *without any decoding*.
        #[inline]
        fn from(zs: &'a ZalgoString) -> Self {
            &zs.0
        }
    }

    macro_rules! impl_partial_eq {
        ($($rhs:ty),+) => {
            $(
                impl<'a> PartialEq<$rhs> for ZalgoString {
                    #[inline]
                    fn eq(&self, other: &$rhs) -> bool {
                        &self.0 == other
                    }
                }
            )+
        };
    }
    impl_partial_eq! {String, str, std::borrow::Cow<'a, str>}

    impl fmt::Display for ZalgoString {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

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
pub fn zalgo_encode(string_to_encode: &str) -> Result<String, ZalgoError> {
    let mut line = 1;
    let mut result = Vec::<u8>::with_capacity(2 * string_to_encode.len() + 1);
    result.push(b'E');
    for b in string_to_encode.bytes() {
        match nonprintable_char_repr(b) {
            Some(repr) => return Err(ZalgoError::NonprintableAscii(b, line, repr)),
            None => {
                if b == b'\n' {
                    line += 1;
                }
                if b < 127 {
                    let v = if b == b'\n' { 111 } else { (b - 11) % 133 - 21 };
                    result.push((v >> 6) & 1 | 0b11001100);
                    result.push((v & 63) | 0b10000000);
                } else {
                    return Err(ZalgoError::NotAscii(b, line));
                }
            }
        }
    }

    Ok(String::from_utf8(result).expect("the encoding process does not produce invalid utf8 given valid ascii text, which is verified before this point"))
}

/// Takes in a string that was encoded by [`zalgo_encode`]
/// and decodes it back into an ASCII string. Can fail if
/// the given a string that was not encoded by [`zalgo_encode`].
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
pub fn zalgo_wrap_python(string_to_encode: &str) -> Result<String, ZalgoError> {
    let encoded_string = zalgo_encode(string_to_encode)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))"))
}

#[derive(Debug, Clone, PartialEq)]
/// The error returned by [`zalgo_encode`] and [`zalgo_wrap_python`]
/// if they encounter a byte they can not encode.
pub enum ZalgoError {
    NonprintableAscii(u8, usize, &'static str),
    NotAscii(u8, usize),
}

impl ZalgoError {
    /// Returns the (1-indexed) line number of the line on which the unencodable byte occured.
    pub const fn line(&self) -> usize {
        match self {
            Self::NonprintableAscii(_, line, _) | Self::NotAscii(_, line) => *line,
        }
    }

    /// Returns the byte value of the unencodable character. Note that this might
    /// not be the complete representation of the character in unicode, just the first
    /// byte of it.
    /// ```
    /// # use zalgo_codec_common::{ZalgoError, zalgo_encode};
    /// assert_eq!(zalgo_encode("\r").err().unwrap().byte(), 13);
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().byte(),  226);
    /// ```
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
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().representation(), None);
    /// ```
    pub const fn representation(&self) -> Option<&'static str> {
        match self {
            Self::NonprintableAscii(_, _, repr) => Some(*repr),
            Self::NotAscii(_, _) => None,
        }
    }
}

impl fmt::Display for ZalgoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NonprintableAscii(byte, line, repr) => write!(
                f,
                "line {line}: can not encode ASCII \"{repr}\" characters with byte value {byte}"
            ),
            Self::NotAscii(byte, line) => write!(
                f,
                "line {line}: byte value {byte} does not correspond to an ASCII character"
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
