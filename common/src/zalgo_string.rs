use crate::{decode_byte_pair, fmt, zalgo_encode, ZalgoError};
use core::{borrow::Borrow, iter::FusedIterator};
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};

/// A thin wrapper around a [`String`] that's been encoded with [`zalgo_encode`]. The main benefit of using this type is that
/// decoding can safely be done in-place since it is known how it was encoded. If the `serde_support` feature is enabled this struct derives the
/// [`Serialize`] and [`Deserialize`] traits.
#[derive(Debug, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ZalgoString(String);

impl ZalgoString {
    /// Encodes the given string with [`zalgo_encode`] and stores the result in a new allocation.
    /// Returns an error if the input string contains bytes that don't correspond to printable
    /// ASCII characters or newlines.
    /// # Examples
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert_eq!(ZalgoString::try_new("Zalgo").unwrap(), "É̺͇͌͏");
    /// ```
    /// A `ZalgoString` created from a `String` is the same as one created from a `&str`:
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert_eq!(
    ///     ZalgoString::try_new(String::from("Zalgo\nHe comes")).unwrap(),
    ///     ZalgoString::try_new("Zalgo\nHe comes").unwrap(),
    /// );
    /// ```
    /// Can only encode printable ASCII and newlines:
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert!(ZalgoString::try_new("❤️").is_err());
    /// assert!(ZalgoString::try_new("\r").is_err());
    /// ```
    pub fn try_new<S: Borrow<str>>(s: S) -> Result<Self, ZalgoError> {
        zalgo_encode(s.borrow()).map(Self)
    }

    /// Decodes `self` into a normal `String` in-place. This method has no effect on the allocated capacity.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::try_new("Zalgo").unwrap();
    /// assert_eq!("Zalgo", zs.into_decoded_string());
    /// // println!("{zs}"); // Error: value used after move
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_decoded_string(self) -> String {
        // Safety: we know that the starting string was encoded from valid ASCII to begin with
        // so every decoded byte is a valid utf-8 character.
        // This invariant can be broken by improper use of the function `ZalgoString::as_bytes_mut`.
        unsafe { String::from_utf8_unchecked(self.into_decoded_bytes()) }
    }

    /// Decodes `self` into a `Vec<u8>` in-place. This method has no effect on the allocated capacity.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::try_new("Zalgo").unwrap();
    /// assert_eq!(b"Zalgo".to_vec(), zs.into_decoded_bytes());
    /// // println!("{zs}"); // Error: value used after move
    /// ```
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

    /// Returns the contents of `self` as a string slice.
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

    /// Returns the contents of `self` as a mutable byte slice.
    /// # Safety
    /// This function has the same safety invariants as [`str::as_bytes_mut`] in addition to requiring
    /// that the result of decoding the string after the borrow ends also results in
    /// valid UTF-8.
    ///
    /// Use of a `ZalgoString`, `String`, or `str` whose contents is not valid UFT-8 is undefined behaviour.
    #[inline]
    #[must_use]
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.0.as_bytes_mut()
    }

    /// Returns the length of `self` in bytes. The allocated capacity is the same.
    #[inline]
    #[must_use]
    pub fn len(&self) -> core::num::NonZeroUsize {
        self.0
            .len()
            .try_into()
            .expect("the length is always at least 1 due to the initial 'E' in encoded strings")
    }

    /// Returns the length of the `ZalgoString` in bytes if it were to be decoded.  
    /// This is computed without any decoding.
    #[inline]
    #[must_use]
    pub fn decoded_len(&self) -> usize {
        (self.len().get() - 1) / 2
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
    /// let zs = ZalgoString::try_new("Zalgo").unwrap();
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

    /// Returns an iterator over the characters of the `ZalgoString`. For a `ZalgoString` the characters are the different accents and zero-width joiners that make it up.
    /// See [`core::str::chars`](https://doc.rust-lang.org/1.70.0/core/primitive.str.html#method.chars) for more information.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::try_new("Zalgo").unwrap();
    /// let mut chars = zs.chars();
    /// // A ZalgoString always begins with an 'E'
    /// assert_eq!(chars.next(), Some('E'));
    /// // After that it gets weird
    /// assert_eq!(chars.next(), Some('\u{33a}'));
    /// ```
    #[inline]
    pub fn chars(&self) -> core::str::Chars<'_> {
        self.0.chars()
    }

    /// Returns an iterator over the decoded characters of the `ZalgoString`. These characters are guaranteed to be valid ASCII.
    /// # Example
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// let zs = ZalgoString::try_new("Zlgoa").unwrap();
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

    /// Converts `self` into a byte vector.
    /// This simply returns the underlying buffer without any cloning or decoding.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    /// Converts `self` into a `String`.  
    /// This simply returns the underlying `String` without any cloning or decoding.
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_string(self) -> String {
        self.0
    }
}

/// An iterator over the decoded bytes of a [`ZalgoString`].
///
/// This struct is obtained by calling the [`decoded_bytes`](ZalgoString::decoded_bytes) method on a [`ZalgoString`].
/// See its documentation for more.
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

/// An iterator over the decoded characters of a [`ZalgoString`].
///
/// This struct is obtained by calling the [`decoded_chars`](ZalgoString::decoded_chars) method on a [`ZalgoString`].
/// See it's documentation for more.
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
