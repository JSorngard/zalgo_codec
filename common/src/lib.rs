//! A crate for converting an ASCII text string to a single unicode grapheme cluster and back.
//! Provides the non-macro functionality of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/).
//!
use core::{fmt, str};
use std::error::Error;

pub use zalgo_string::ZalgoString;
mod zalgo_string {
    use super::{decode_byte_pair, fmt, zalgo_decode, zalgo_encode, ZalgoError};
    use core::{borrow::Borrow, convert, iter::FusedIterator};
    #[cfg(feature = "serde_support")]
    use serde::{Deserialize, Serialize};

    /// A thin wrapper around a [`String`] that's been encoded with [`zalgo_encode`]. The main benefit of using this type is that
    /// decoding can safely be done in-place since it is known how it was encoded.
    #[derive(Debug, Clone, PartialEq, Hash)]
    #[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
    pub struct ZalgoString(String);

    impl ZalgoString {
        /// Encodes the given string with [`zalgo_encode`] and stores the result in a new allocation.
        /// # Example
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// assert_eq!(ZalgoString::try_encode_new("Zalgo").unwrap(), "É̺͇͌͏");
        /// ```
        /// A `ZalgoString` created from a `String` is the same as one created from a `&str`:
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// assert_eq!(
        ///     ZalgoString::try_encode_new(String::from("Zalgo\nHe comes")).unwrap(),
        ///     ZalgoString::try_encode_new("Zalgo\nHe comes").unwrap(),
        /// );
        /// ```
        /// Can only encode printable ASCII and newlines:
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// assert!(ZalgoString::try_encode_new("❤️").is_err());
        /// assert!(ZalgoString::try_encode_new("\r").is_err());
        /// ```
        pub fn try_encode_new<S: Borrow<str>>(s: S) -> Result<Self, ZalgoError> {
            zalgo_encode(s.borrow()).map(Self)
        }

        /// Decodes `self` into a normal `String` in-place. This method has no effect on the allocated capacity.
        /// # Example
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// let zs = ZalgoString::try_encode_new("Zalgo").unwrap();
        /// assert_eq!("Zalgo", zs.into_decoded());
        /// // println!("{zs}"); // Error: value borrowed here after move
        /// ```
        #[must_use = "`self` will be dropped if the result is not used"]
        pub fn into_decoded(self) -> String {
            let mut w = 0;
            let mut bytes = self.into_bytes();
            for r in (1..bytes.len()).step_by(2) {
                bytes[w] = decode_byte_pair(bytes[r], bytes[r + 1]);
                w += 1;
            }
            bytes.truncate(w);
            // Safety: we know that the starting string was encoded from valid ASCII to begin with
            // so every decoded byte is a valid utf-8 character.
            unsafe { String::from_utf8_unchecked(bytes) }
        }

        /// Decode the contents of `self` into a new `String`.
        /// # Example
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// let zs = ZalgoString::try_encode_new("Zalgo").unwrap();
        /// assert_eq!(zs.decoded(), "Zalgo");
        /// // We can still use the ZalgoString
        /// println!("{zs}");
        /// ```
        pub fn decoded(&self) -> String {
            zalgo_decode(&self.0).expect("we know that the original string is valid ASCII")
        }

        /// Extracts a string slice containing the entire `ZalgoString`.
        #[inline]
        #[must_use]
        pub fn as_str(&self) -> &str {
            &self.0
        }

        /// Returns the contents of `self` as a byte slice.
        #[inline]
        #[must_use]
        pub fn as_bytes(&self) -> &[u8] {
            self.0.as_bytes()
        }

        /// Returns the length of `self` in bytes. The allocated capacity is the same.
        #[inline]
        #[must_use]
        pub fn len(&self) -> usize {
            self.0.len()
        }

        /// Returns an iterator over the bytes of the `ZalgoString`. See [`core::str::bytes`](https://doc.rust-lang.org/1.70.0/core/primitive.str.html#method.bytes) for more information.
        #[inline]
        pub fn bytes(&self) -> core::str::Bytes<'_> {
            self.0.bytes()
        }

        /// Returns an iterator over the decoded bytes of the `ZalgoString`. These bytes are guaranteed to represent valid ASCII.
        /// # Example
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// let zs = ZalgoString::try_encode_new("Zalgo").unwrap();
        /// let mut decoded_bytes = zs.decoded_bytes();
        /// assert_eq!(decoded_bytes.next().unwrap(), 90);
        /// assert_eq!(decoded_bytes.collect::<Vec<u8>>(), vec![97, 108, 103, 111]);
        #[inline]
        pub fn decoded_bytes(&self) -> DecodedBytes<'_> {
            DecodedBytes {
                zs: self.as_bytes(),
                index: 1,
                back_index: self.as_bytes().len(),
            }
        }

        /// Returns an iterator over the characters of the `ZalgoString`. For a `ZalgoString` the characters are the different accents and zero-width joiners that make it up.
        /// See [`core::str::chars`](https://doc.rust-lang.org/1.70.0/core/primitive.str.html#method.chars) for more information.
        /// # Example
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// let zs = ZalgoString::try_encode_new("Zalgo").unwrap();
        /// let mut chars = zs.chars();
        /// // A ZalgoString always begin with an 'E'
        /// assert_eq!(chars.next().unwrap(), 'E');
        /// // After that it gets weird
        /// assert_eq!(chars.next().unwrap(), '\u{33a}');
        /// ```
        #[inline]
        pub fn chars(&self) -> core::str::Chars<'_> {
            self.0.chars()
        }

        /// Returns an iterator over the decoded characters of the `ZalgoString`. These characters are guaranteed to be valid ASCII.
        /// # Example
        /// ```
        /// # use zalgo_codec_common::ZalgoString;
        /// let zs = ZalgoString::try_encode_new("Zalgo").unwrap();
        /// let mut decoded_chars = zs.decoded_chars();
        /// assert_eq!(decoded_chars.next().unwrap(), 'Z');
        /// assert_eq!(decoded_chars.collect::<String>(), "algo");
        /// ```
        #[inline]
        pub fn decoded_chars(&self) -> DecodedChars<'_> {
            DecodedChars {
                dcb: self.decoded_bytes(),
            }
        }

        /// Converts `self` into a byte vector.
        /// This consumes the `ZalgoString`, so we do not need to copy its contents.
        #[inline]
        #[must_use = "`self` will be dropped if the result is not used"]
        pub fn into_bytes(self) -> Vec<u8> {
            self.0.into_bytes()
        }
    }

    pub struct DecodedBytes<'a> {
        zs: &'a [u8],
        index: usize,
        back_index: usize,
    }

    impl<'a> Iterator for DecodedBytes<'a> {
        type Item = u8;
        fn next(&mut self) -> Option<Self::Item> {
            if self.index < self.back_index {
                let t = Some(decode_byte_pair(
                    self.zs[self.index],
                    self.zs[self.index + 1],
                ));
                self.index += 2;
                t
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let left = (self.back_index.saturating_sub(self.index)) / 2;
            (left, Some(left))
        }
    }

    impl<'a> DoubleEndedIterator for DecodedBytes<'a> {
        fn next_back(&mut self) -> Option<Self::Item> {
            if self.back_index > self.index {
                let t = Some(decode_byte_pair(
                    self.zs[self.back_index - 2],
                    self.zs[self.back_index - 1],
                ));
                self.back_index -= 2;
                t
            } else {
                None
            }
        }
    }

    impl<'a> FusedIterator for DecodedBytes<'a> {}

    pub struct DecodedChars<'a> {
        dcb: DecodedBytes<'a>,
    }

    impl<'a> Iterator for DecodedChars<'a> {
        type Item = char;
        fn next(&mut self) -> Option<Self::Item> {
            self.dcb.next().map(char::from)
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.dcb.size_hint()
        }
    }

    impl<'a> DoubleEndedIterator for DecodedChars<'a> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.dcb.next_back().map(char::from)
        }
    }

    impl<'a> FusedIterator for DecodedChars<'a> {}

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
    impl_partial_eq! {String, &str, str, std::borrow::Cow<'a, str>}

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
        .map(|(odd, even)| decode_byte_pair(odd, even))
        .collect();

    String::from_utf8(bytes)
}

#[must_use]
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
