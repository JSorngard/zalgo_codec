use crate::{decode_byte_pair, fmt, zalgo_encode, ZalgoError};
use core::iter::{ExactSizeIterator, FusedIterator};
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// A thin wrapper around a [`String`] that has been encoded with [`zalgo_encode`].
/// This struct can be decoded in-place and also allows iteration over its characters and bytes, both in
/// decoded and encoded form.
/// If the `serde_support` feature is enabled this struct derives the
/// [`Serialize`] and [`Deserialize`] traits.
#[derive(Debug, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ZalgoString {
    string: String,
}

impl ZalgoString {
    /// Encodes the given string slice with [`zalgo_encode`] and stores the result in a new allocation.
    /// Returns an error if the input string contains bytes that don't correspond to printable
    /// ASCII characters or newlines.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert_eq!(ZalgoString::new("Zalgo").unwrap(), "É̺͇͌͏");
    /// ```
    /// Can only encode printable ASCII and newlines:
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert!(ZalgoString::new("❤️").is_err());
    /// assert!(ZalgoString::new("\r").is_err());
    /// ```
    #[must_use = "this function returns a new `ZalgoString`, it does not modify the input"]
    pub fn new(s: &str) -> Result<Self, ZalgoError> {
        zalgo_encode(s).map(|string| Self { string })
    }

    /// Returns the encoded contents of `self` as a string slice.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Oh boy!").unwrap();
    /// assert_eq!(zs.as_str(), "È̯͈͂͏͙́");
    /// ```
    /// Iterate through the encoded [`char`](prim@char)s
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("42").unwrap();
    /// let mut chars = zs.as_str().chars();
    /// // A `ZalgoString` always begins with an 'E'
    /// assert_eq!(chars.next(), Some('E'));
    /// // After that it gets weird
    /// assert_eq!(chars.next(), Some('\u{314}'));
    /// ```
    /// Combining characters lie deep in the dark depths of Unicode,
    /// and may not match with your intuition of what a character is.
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zalgo").unwrap();
    /// let mut ci = zs.as_str().char_indices();
    /// assert_eq!(ci.next(), Some((0, 'E')));
    /// assert_eq!(ci.next(), Some((1,'\u{33a}')));
    /// // Note the 3 here, the combining characters can take up multiple bytes.
    /// assert_eq!(ci.next(), Some((3, '\u{341}')));
    /// // The final character begins at position 9
    /// assert_eq!(ci.last(), Some((9, '\u{34f}')));
    /// // even though the length in bytes is 11
    /// assert_eq!(zs.len(), 11);
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.string
    }

    /// Returns an iterator over the decoded characters of the `ZalgoString`. These characters are guaranteed to be valid ASCII.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zlgoa").unwrap();
    /// let mut decoded_chars = zs.decoded_chars();
    /// assert_eq!(decoded_chars.next(), Some('Z'));
    /// assert_eq!(decoded_chars.next_back(), Some('a'));
    /// assert_eq!(decoded_chars.next(), Some('l'));
    /// assert_eq!(decoded_chars.next(), Some('g'));
    /// assert_eq!(decoded_chars.next_back(), Some('o'));
    /// assert_eq!(decoded_chars.next(), None);
    /// assert_eq!(decoded_chars.next_back(), None);
    /// ```
    #[inline]
    pub fn decoded_chars(&self) -> DecodedChars<'_> {
        DecodedChars {
            dcb: self.decoded_bytes(),
        }
    }

    /// Converts `self` into a `String`.
    /// This simply returns the underlying `String` without any cloning or decoding.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zalgo\n He comes!").unwrap();
    /// let es = "É̺͇͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ";
    /// assert_eq!(zs.to_string(), es);
    /// assert_eq!(zs.into_string(), es);
    /// // println!("{zs}"); // Error: value used after move
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_string(self) -> String {
        self.string
    }

    /// Decodes `self` into a `String` in-place. This method has no effect on the allocated capacity.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let s = "Zalgo";
    /// let zs = ZalgoString::new(s).unwrap();
    /// assert_eq!(s, zs.into_decoded_string());
    /// // println!("{zs}"); // Error: value used after move
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_decoded_string(self) -> String {
        // Safety: we know that the starting string was encoded from valid ASCII to begin with
        // so every decoded byte is a valid utf-8 character.
        unsafe { String::from_utf8_unchecked(self.into_decoded_bytes()) }
    }

    /// Returns the contents of `self` as a byte slice.
    /// The first byte is always 69, after that the bytes no longer correspond to ASCII characters.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zalgo").unwrap();
    /// let mut bytes = zs.as_bytes();
    /// assert_eq!(bytes[0], 69);
    /// assert_eq!(&bytes[1..5], &[204, 186, 205, 129]);
    /// ```
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    /// Returns an iterator over the decoded bytes of the `ZalgoString`. These bytes are guaranteed to represent valid ASCII.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zalgo").unwrap();
    /// let mut decoded_bytes = zs.decoded_bytes();
    /// assert_eq!(decoded_bytes.next(), Some(90));
    /// assert_eq!(decoded_bytes.next_back(), Some(111));
    /// assert_eq!(decoded_bytes.collect::<Vec<u8>>(), vec![97, 108, 103]);
    #[inline]
    pub fn decoded_bytes(&self) -> DecodedBytes<'_> {
        DecodedBytes {
            zs: self.as_bytes(),
            index: 1,
            back_index: self.as_bytes().len(),
        }
    }

    /// Converts `self` into a byte vector.
    /// This simply returns the underlying buffer without any cloning or decoding.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zalgo").unwrap();
    /// assert_eq!(zs.into_bytes(), vec![69, 204, 186, 205, 129, 205, 140, 205, 135, 205, 143]);
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> Vec<u8> {
        self.string.into_bytes()
    }

    /// Decodes `self` into a byte vector in-place. This method has no effect on the allocated capacity.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Zalgo").unwrap();
    /// assert_eq!(b"Zalgo".to_vec(), zs.into_decoded_bytes());
    /// // println!("{zs}"); // Error: value used after move
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_decoded_bytes(self) -> Vec<u8> {
        let mut w = 0;
        let mut bytes = self.into_bytes();
        for r in (1..bytes.len()).step_by(2) {
            bytes[w] = decode_byte_pair(bytes[r], bytes[r + 1]);
            w += 1;
        }
        bytes.truncate(w);
        bytes
    }

    /// Returns the length of `self` in bytes. The allocated capacity is the same.
    /// This length is twice the length of the original `String` plus one.
    /// # Example
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::new("Z").unwrap();
    /// assert_eq!(zs.len(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.string.len()
    }

    /// Returns the length of the `ZalgoString` in bytes if it were to be decoded.  
    /// This is computed without any decoding.
    /// # Example
    /// Basic usage
    /// let s = "Zalgo, He comes!";
    /// let zs = ZalgoString::new(s);
    /// assert_eq!(s.len(), zs.decoded_len());
    #[inline]
    #[must_use]
    pub fn decoded_len(&self) -> usize {
        (self.len() - 1) / 2
    }
}

/// An iterator over the decoded bytes of a [`ZalgoString`].
///
/// This struct is obtained by calling the [`decoded_bytes`](ZalgoString::decoded_bytes) method on a [`ZalgoString`].
/// See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DecodedBytes<'a> {
    zs: &'a [u8],
    index: usize,
    back_index: usize,
}

impl<'a> Iterator for DecodedBytes<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.back_index {
            let t = decode_byte_pair(self.zs[self.index], self.zs[self.index + 1]);
            self.index += 2;
            Some(t)
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
impl<'a> ExactSizeIterator for DecodedBytes<'a> {}

/// An iterator over the decoded characters of a [`ZalgoString`].
///
/// This struct is obtained by calling the [`decoded_chars`](ZalgoString::decoded_chars) method on a [`ZalgoString`].
/// See it's documentation for more.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
impl<'a> ExactSizeIterator for DecodedChars<'a> {}

macro_rules! impl_partial_eq {
    ($($rhs:ty),+) => {
        $(
            impl<'a> PartialEq<$rhs> for ZalgoString {
                #[inline]
                fn eq(&self, other: &$rhs) -> bool {
                    &self.string == other
                }
            }
        )+
    };
}
impl_partial_eq! {String, &str, str, std::borrow::Cow<'a, str>}

impl fmt::Display for ZalgoString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[cfg(test)]
mod test {
    use super::ZalgoString;

    #[test]
    fn check_into_decoded_string() {
        let s = "Zalgo\n He comes!";
        let zs: ZalgoString = ZalgoString::new(s).unwrap();
        assert_eq!(zs.into_decoded_string(), s);

        let zs = ZalgoString::new("").unwrap();
        assert_eq!(zs.into_decoded_string(), "");
    }

    #[test]
    fn check_string_from_zalgo_string() {
        let zs = ZalgoString::new("Zalgo\n He comes!").unwrap();
        assert_eq!(zs.to_string(), "É̺͇͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ");
        assert_eq!(zs.into_string(), "É̺͇͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ");

        let zs = ZalgoString::new("").unwrap();
        assert_eq!(zs.into_string(), "E");
    }
}
