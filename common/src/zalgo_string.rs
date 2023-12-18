//! Contains the implementation of [`ZalgoString`] as well as related iterators.

use crate::{decode_byte_pair, fmt, zalgo_encode, Error};

use core::iter::{ExactSizeIterator, FusedIterator};

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::borrow::Cow;

/// A [`String`] that has been encoded with [`zalgo_encode`].
/// This struct can be decoded in-place and also allows iteration over its characters and bytes, both in
/// decoded and encoded form.
///
/// If the `serde` feature is enabled this struct implements the
/// [`Serialize`](serde::Serialize) and [`Deserialize`](serde::Deserialize) traits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ZalgoString(String);

impl ZalgoString {
    /// Encodes the given string slice with [`zalgo_encode`] and stores the result in a new allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the input string contains bytes that don't correspond to printable
    /// ASCII characters or newlines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// assert_eq!(ZalgoString::new("Zalgo")?, "É̺͇͌͏");
    /// # Ok::<(), Error>(())
    /// ```
    /// Can only encode printable ASCII and newlines:
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert!(ZalgoString::new("❤️").is_err());
    /// assert!(ZalgoString::new("\r").is_err());
    /// ```
    #[must_use = "this function returns a new `ZalgoString` and does not modify the input"]
    pub fn new(s: &str) -> Result<Self, Error> {
        zalgo_encode(s).map(Self)
    }

    /// Returns the *encoded* contents of `self` as a string slice.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Oh boy!")?;
    /// assert_eq!(zs.as_str(), "È̯͈͂͏͙́");
    /// # Ok::<(), Error>(())
    /// ```
    /// Note that `ZalgoString` implements [`PartialEq`] with common string types,
    /// so the comparison in the above example could also be done directly
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// # let zs = ZalgoString::new("Oh boy!")?;
    /// assert_eq!(zs, "È̯͈͂͏͙́");
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a reference and does not modify `self`"]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns an iterator over the encoded characters of the `ZalgoString`.
    ///
    /// The first character is an "E", the others are unicode combining characters.
    ///
    /// # Example
    ///
    /// Iterate through the encoded [`char`]s:
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("42")?;
    /// let mut chars = zs.chars();
    /// assert_eq!(chars.next(), Some('E'));
    /// assert_eq!(chars.next(), Some('\u{314}'));
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn chars(&self) -> core::str::Chars<'_> {
        self.0.chars()
    }

    /// Returns an iterator over the encoded characters of the `ZalgoString` and their positions.
    ///
    /// # Example
    ///
    /// Combining characters lie deep in the dark depths of Unicode,
    /// and may not match with your intuition of what a character is.
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// let mut ci = zs.char_indices();
    /// assert_eq!(ci.next(), Some((0, 'E')));
    /// assert_eq!(ci.next(), Some((1,'\u{33a}')));
    /// // Note the 3 here, the combining characters take up two bytes.
    /// assert_eq!(ci.next(), Some((3, '\u{341}')));
    /// // The final character begins at position 9
    /// assert_eq!(ci.next_back(), Some((9, '\u{34f}')));
    /// // even though the length in bytes is 11
    /// assert_eq!(zs.len(), 11);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn char_indices(&self) -> core::str::CharIndices<'_> {
        self.0.char_indices()
    }

    /// Returns an iterator over the decoded characters of the `ZalgoString`.
    ///
    /// These characters are guaranteed to be valid ASCII.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zlgoa")?;
    /// let mut decoded_chars = zs.decoded_chars();
    /// assert_eq!(decoded_chars.next(), Some('Z'));
    /// assert_eq!(decoded_chars.next_back(), Some('a'));
    /// assert_eq!(decoded_chars.next(), Some('l'));
    /// assert_eq!(decoded_chars.next(), Some('g'));
    /// assert_eq!(decoded_chars.next_back(), Some('o'));
    /// assert_eq!(decoded_chars.next(), None);
    /// assert_eq!(decoded_chars.next_back(), None);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn decoded_chars(&self) -> DecodedChars<'_> {
        DecodedChars(self.decoded_bytes())
    }

    /// Converts `self` into a `String`.
    ///
    /// This simply returns the underlying `String` without any cloning or decoding.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo\n He comes!")?;
    /// assert_eq!(zs.into_string(), "É̺͇͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ");
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_string(self) -> String {
        self.0
    }

    /// Decodes `self` into a `String` in-place.
    ///
    /// This method has no effect on the allocated capacity.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let s = "Zalgo";
    /// let zs = ZalgoString::new(s)?;
    /// assert_eq!(s, zs.into_decoded_string());
    /// # Ok::<(), Error>(())
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_decoded_string(self) -> String {
        // Safety: we know that the starting string was encoded from valid ASCII to begin with
        // so every decoded byte is a valid utf-8 character.
        unsafe { String::from_utf8_unchecked(self.into_decoded_bytes()) }
    }

    /// Returns the encoded contents of `self` as a byte slice.
    ///
    /// The first byte is always 69, after that the bytes no longer correspond to ASCII characters.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// let bytes = zs.as_bytes();
    /// assert_eq!(bytes[0], 69);
    /// assert_eq!(&bytes[1..5], &[204, 186, 205, 129]);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a reference and does not modify `self`"]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns an iterator over the encoded bytes of the `ZalgoString`.
    ///
    /// Since a `ZalgoString` always begins with an "E", the first byte is always 69.
    /// After that the bytes no longer correspond to ASCII values.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Bytes")?;
    /// let mut bytes = zs.bytes();
    /// assert_eq!(bytes.next(), Some(69));
    /// assert_eq!(bytes.nth(5), Some(148));
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn bytes(&self) -> core::str::Bytes<'_> {
        self.0.bytes()
    }

    /// Returns an iterator over the decoded bytes of the `ZalgoString`.
    ///
    /// These bytes are guaranteed to represent valid ASCII.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// let mut decoded_bytes = zs.decoded_bytes();
    /// assert_eq!(decoded_bytes.next(), Some(90));
    /// assert_eq!(decoded_bytes.next_back(), Some(111));
    /// assert_eq!(decoded_bytes.collect::<Vec<u8>>(), vec![97, 108, 103]);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn decoded_bytes(&self) -> DecodedBytes<'_> {
        DecodedBytes(self.0.bytes().skip(1))
    }

    /// Converts `self` into a byte vector.
    ///
    /// This simply returns the underlying buffer without any cloning or decoding.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// assert_eq!(zs.into_bytes(), vec![69, 204, 186, 205, 129, 205, 140, 205, 135, 205, 143]);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    /// Decodes `self` into a byte vector in-place.
    ///
    /// This method has no effect on the allocated capacity.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// assert_eq!(b"Zalgo".to_vec(), zs.into_decoded_bytes());
    /// # Ok::<(), Error>(())
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

    /// Returns the length of `self` in bytes.
    ///
    /// This length is twice the length of the original `String` plus one.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Z")?;
    /// assert_eq!(zs.len(), 3);
    /// # Ok::<(), Error>(())
    /// ```
    // Since the length is never empty it makes no sense to have an is_empty function.
    // The decoded length can be empty though, so `decoded_is_empty` is provided instead.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the capacity of the underlying encoded string in bytes.
    ///
    /// The `ZalgoString` is preallocated to the needed capacity of twice the length
    /// of the original unencoded `String` plus one.
    /// However, this size is not guaranteed since the allocator can choose to allocate more space.
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Returns the length of the `ZalgoString` in bytes if it were to be decoded.  
    ///
    /// This is computed without any decoding.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let s = "Zalgo, He comes!";
    /// let zs = ZalgoString::new(s)?;
    /// assert_eq!(s.len(), zs.decoded_len());
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn decoded_len(&self) -> usize {
        (self.len() - 1) / 2
    }

    /// Returns whether the string would be empty if decoded.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("")?;
    /// assert!(zs.decoded_is_empty());
    /// let zs = ZalgoString::new("Blargh")?;
    /// assert!(!zs.decoded_is_empty());
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn decoded_is_empty(&self) -> bool {
        self.decoded_len() == 0
    }

    /// Encodes the given string and appends the result to the end of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let s1 = "Zalgo";
    /// let s2 = ", He comes!";
    /// let mut zs = ZalgoString::new(s1)?;
    /// zs.encode_and_push_str(s2)?;
    /// assert_eq!(
    ///     zs.into_decoded_string(),
    ///     format!("{s1}{s2}")
    /// );
    /// # Ok::<(), Error>(())
    /// ```
    pub fn encode_and_push_str(&mut self, s: &str) -> Result<(), Error> {
        let zs = zalgo_encode(s)?;
        let (_, combining_chars) = zs.split_at(1);
        self.string.push_str(combining_chars);
        Ok(())
    }
}

/// An iterator over the decoded bytes of a [`ZalgoString`].
///
/// This struct is obtained by calling the [`decoded_bytes`](ZalgoString::decoded_bytes) method on a [`ZalgoString`].
/// See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DecodedBytes<'a>(core::iter::Skip<core::str::Bytes<'a>>);

impl<'a> Iterator for DecodedBytes<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .zip(self.0.next())
            .map(|(odd, even)| decode_byte_pair(odd, even))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.0.size_hint().0 / 2;
        (left, Some(left))
    }
}

impl<'a> DoubleEndedIterator for DecodedBytes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .zip(self.0.next_back())
            .map(|(even, odd)| decode_byte_pair(odd, even))
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
pub struct DecodedChars<'a>(DecodedBytes<'a>);

impl<'a> Iterator for DecodedChars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(char::from)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for DecodedChars<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(char::from)
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
                    &self.0 == other
                }
            }
        )+
    };
}
impl_partial_eq! {String, &str, str, Cow<'a, str>}

impl fmt::Display for ZalgoString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

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

    #[test]
    fn check_partial_eq() {
        let enc = "É̺͇͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ";
        let zs = ZalgoString::new("Zalgo\n He comes!").unwrap();
        assert_eq!(zs, enc);
        assert_eq!(zs, String::from(enc));
        assert_eq!(zs, Cow::from(enc));
    }

    #[test]
    fn check_push_str() {
        let s1 = "Zalgo";
        let s2 = ", He comes";
        let mut zs = ZalgoString::new(s1).unwrap();
        zs.encode_and_push_str(s2).unwrap();
        assert_eq!(zs.clone().into_decoded_string(), format!("{s1}{s2}"));
        zs.encode_and_push_str("").unwrap();
        assert_eq!(zs.clone().into_decoded_string(), format!("{s1}{s2}"));
        assert!(zs.encode_and_push_str("å").is_err());
    }
}
