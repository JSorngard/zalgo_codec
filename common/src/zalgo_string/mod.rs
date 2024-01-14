//! Contains the implementation of [`ZalgoString`] as well as related iterators.
//!
//! A `ZalgoString` contains a grapheme cluster that was obtained from [`zalgo_encode`].
//! It allows for iteration over its characters and bytes in both encoded and decoded form.
//! It can be decoded in-place and the encoded information in other ZalgoStrings can be pushed
//! onto it.

mod iterators;

use crate::{decode_byte_pair, fmt, zalgo_encode, Error};
pub use iterators::{DecodedBytes, DecodedChars};

use core::{
    ops::{Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    slice::SliceIndex,
};

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

    // region: character access methods

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

    /// Returns a subslice of `self`.
    ///
    /// Same as [`str::get`].
    ///
    /// This is the non-panicking alternative to indexing the `ZalgoString`. Returns [`None`] whenever
    /// the equivalent indexing operation would panic.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// assert_eq!(zs.get(0..3), Some("E\u{33a}"));
    ///
    /// // indices not on UTF-8 sequence boundaries
    /// assert!(zs.get(0..4).is_none());
    ///
    /// // out of bounds
    /// assert!(zs.get(..42).is_none());
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<str>>::Output>
    where
        I: SliceIndex<str>,
    {
        self.0.get(index)
    }

    /// Returns an unchecked subslice of `self`.
    ///
    /// This is the unchecked alternative to indexing a `ZalgoString`.
    ///
    /// # Safety
    ///
    /// This function has the same safety requirements as [`str::get_unchecked`]:
    /// - The starting index must not exceed the ending index;
    /// - Indexes must be within bounds of the original slice;
    /// - Indexes must lie on UTF-8 sequence boundaries.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Zalgo")?;
    /// unsafe {
    ///     assert_eq!(zs.get_unchecked(..3), "E\u{33a}");
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> &<I as SliceIndex<str>>::Output
    where
        I: SliceIndex<str>,
    {
        self.0.get_unchecked(index)
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
        DecodedChars::new(self)
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

    // endregion: character access methods

    // region: byte access methods

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
        DecodedBytes::new(self)
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

    // endregion: byte access methods

    // region: metadata methods

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

    // endregion: metadata methods

    /// Returns a string slice of just the combining characters of the `ZalgoString` without the inital 'E'.
    ///
    /// Note that [`zalgo_decode`](crate::zalgo_decode) assumes that the initial 'E' is present,
    /// and can not decode the result of this method.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Hi")?;
    /// assert_eq!(zs.as_combining_chars(), "\u{328}\u{349}");
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn as_combining_chars(&self) -> &str {
        self.0.split_at(1).1
    }

    /// Converts `self` into a String that contains only the combining characters of the grapheme cluster.
    ///
    /// This is an `O(n)` operation since after it has removed the initial "E" it needs to copy every byte
    /// of the string down one index.
    ///
    /// It is the same as calling [`ZalgoString::into_string()`] followed by [`String::remove(0)`](String::remove).
    /// 
    /// Just like [`as_combining_chars`](ZalgoString::as_combining_chars) the result of this method can not
    /// be decoded by [`zalgo_decode`].
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let zs = ZalgoString::new("Hi")?;
    /// let s = zs.into_combining_chars();
    /// assert_eq!(s, "\u{328}\u{349}");
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_combining_chars(mut self) -> String {
        self.0.remove(0);
        self.0
    }

    /// Appends the combining characters of a different `ZalgoString` to the end of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let (s1, s2) = ("Zalgo", ", He comes!");
    ///
    /// let mut zs1 = ZalgoString::new(s1)?;
    /// let zs2 = ZalgoString::new(s2)?;
    ///
    /// zs1.push_zalgo_str(&zs2);
    ///
    /// assert_eq!(zs1.into_decoded_string(), format!("{s1}{s2}"));
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn push_zalgo_str(&mut self, zalgo_string: &Self) {
        self.0.push_str(zalgo_string.as_combining_chars());
    }

    // region: capacity manipulation methods

    /// Reserves capacity for at least `additional` bytes more than the current length.
    ///
    /// Same as [`String::reserve`].
    ///
    /// The allocator may reserve more space to speculatively avoid frequent allocations.
    /// After calling reserve, capacity will be greater than or equal to `self.len() + additional`.  
    ///
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Keep in mind that an encoded ASCII character takes up two bytes, and that a `ZalgoString`
    /// always begins with an unencoded "E" which means that the total length in bytes is always an odd number.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let mut zs = ZalgoString::new("Zalgo")?;
    /// let c = zs.capacity();
    /// zs.reserve(5);
    /// assert!(zs.capacity() >= c + 5);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    /// Reserves capacity for exactly `additional` bytes more than the current length.
    ///
    /// Same as [`String::reserve_exact`].
    ///
    /// Unlike [`reserve`](ZalgoString::reserve), this will not deliberately over-allocate
    /// to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to `self.len() + additional`.
    ///
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Keep in mind that an encoded ASCII character takes up two bytes, and that a `ZalgoString`
    /// always begins with an unencoded "E" which means that the total length in bytes is always an odd number.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let mut zs = ZalgoString::new("Zalgo")?;
    /// let c = zs.capacity();
    /// zs.reserve_exact(5);
    /// assert!(zs.capacity() >= c + 5);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    // endregion: capacity manipulation methods

    // region: length manipulation methods

    /// Shortens the `ZalgoString` to the specified length.
    ///
    /// A `ZalgoString` always takes up an odd number of bytes as the first "E" takes up one,
    /// and all subsequent characters take up two.
    ///
    /// If `new_len` is larger than its current length, this has no effect.
    ///
    /// This method has no effect of the allocated capacity.
    ///
    /// # Panics
    ///
    /// Panics if `new_len` is even.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let mut zs = ZalgoString::new("Zalgo")?;
    /// zs.truncate(5);
    /// assert_eq!(zs, "E\u{33a}\u{341}");
    /// assert_eq!(zs.into_decoded_string(), "Za");
    /// # Ok::<(), Error>(())
    /// ```
    /// Panics if `new_len` is even:
    /// ```should_panic
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let mut zs = ZalgoString::new("Zalgo")?;
    /// zs.truncate(0);
    /// # Ok::<(), Error>(())
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert_eq!(new_len % 2, 1, "the new length must be odd");
            self.0.truncate(new_len)
        }
    }

    /// Truncates this `ZalgoString`, removing all contents except the initial "E".
    ///
    /// This means the ZalgoString will have a length of one, but it does not affect its capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, ZalgoString};
    /// let mut zs = ZalgoString::new("Zalgo")?;
    /// zs.clear();
    /// assert_eq!(zs, "E");
    /// assert!(zs.decoded_is_empty());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn clear(&mut self) {
        self.truncate(1)
    }

    // endregion: length manipulation methods
}

// region: Addition impls

/// Implements the `+` operator for concaternating two `ZalgoString`s.
/// Memorywise it works the same as the `Add` implementation for the normal
/// `String` type: it consumes the lefthand side, extends its buffer, and
/// copies the combining characters of the right hand side into it.
impl core::ops::Add<&ZalgoString> for ZalgoString {
    type Output = ZalgoString;
    #[inline]
    fn add(mut self, rhs: &Self) -> Self::Output {
        self.push_zalgo_str(rhs);
        self
    }
}

/// Implements the `+=` operator for appending to a `ZalgoString`.
///
/// This just calls [`push_zalgo_str`](ZalgoString::push_zalgo_str).
impl core::ops::AddAssign<&ZalgoString> for ZalgoString {
    #[inline]
    fn add_assign(&mut self, rhs: &ZalgoString) {
        self.push_zalgo_str(rhs);
    }
}

// endregion: Addition impls

// region: PartialEq impls

macro_rules! impl_partial_eq {
    ($($rhs:ty),+) => {
        $(
            impl PartialEq<$rhs> for ZalgoString {
                #[inline]
                fn eq(&self, other: &$rhs) -> bool {
                    &self.0 == other
                }
            }

            impl PartialEq<ZalgoString> for $rhs {
                #[inline]
                fn eq(&self, other: &ZalgoString) -> bool {
                    self == &other.0
                }
            }
        )+
    };
}
impl_partial_eq! {String, &str, str, Cow<'_, str>}

// endregion: PartialEq impls

/// Displays the encoded form of the `ZalgoString`.
impl fmt::Display for ZalgoString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// region: impl index

macro_rules! impl_index {
    ($($range:ty),+) => {
        $(
            impl Index<$range> for ZalgoString {
                type Output = str;
                #[inline]
                fn index(&self, index: $range) -> &Self::Output {
                    &self.0[index]
                }
            }
        )+
    };
}

impl_index! {Range<usize>, RangeTo<usize>, RangeFrom<usize>, RangeInclusive<usize>, RangeToInclusive<usize>, RangeFull}

// endregion: impl index

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::{format, string::ToString};

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
        assert_eq!(String::from(enc), zs);
        assert_eq!(Cow::from(enc), zs);
    }

    #[test]
    fn check_push_str() {
        let s1 = "Zalgo";
        let s2 = ", He comes";
        let mut zs = ZalgoString::new(s1).unwrap();
        let zs2 = ZalgoString::new(s2).unwrap();
        zs.push_zalgo_str(&zs2);
        assert_eq!(zs.clone().into_decoded_string(), format!("{s1}{s2}"));
        zs += &zs2;
        assert_eq!(
            (zs + &zs2).into_decoded_string(),
            format!("{s1}{s2}{s2}{s2}")
        );
    }

    #[test]
    fn check_as_combining_chars() {
        assert_eq!(
            ZalgoString::new("Hi").unwrap().as_combining_chars(),
            "\u{328}\u{349}"
        );
        assert_eq!(ZalgoString::new("").unwrap().as_combining_chars(), "");
    }

    #[test]
    fn check_decoded_chars() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        assert_eq!("oglaZ", zs.decoded_chars().rev().collect::<String>());
    }

    #[test]
    fn test_reserve() {
        let mut zs = ZalgoString::new("Zalgo").unwrap();
        zs.reserve(5);
        assert!(zs.capacity() >= 11 + 5);
        let c = zs.capacity();
        zs.reserve(1);
        assert_eq!(zs.capacity(), c);
    }

    #[test]
    fn test_reserve_exact() {
        let mut zs = ZalgoString::new("Zalgo").unwrap();
        zs.reserve_exact(5);
        assert_eq!(zs.capacity(), 11 + 5);
        let c = zs.capacity();
        zs.reserve_exact(1);
        assert_eq!(zs.capacity(), c);
    }

    #[test]
    fn test_truncate() {
        let mut zs = ZalgoString::new("Zalgo").unwrap();
        zs.truncate(100);
        assert_eq!(zs, "E\u{33a}\u{341}\u{34c}\u{347}\u{34f}");
        zs.truncate(5);
        assert_eq!(zs, "E\u{33a}\u{341}");
        assert_eq!(zs.into_decoded_string(), "Za");
    }

    #[test]
    #[should_panic]
    fn test_truncate_panic() {
        let mut zs = ZalgoString::new("Zalgo").unwrap();
        zs.truncate(0)
    }

    #[test]
    fn test_clear() {
        let mut zs = ZalgoString::new("Zalgo").unwrap();
        let c = zs.capacity();
        zs.clear();
        assert_eq!(zs.capacity(), c);
        assert_eq!(zs.len(), 1);
        assert_eq!(zs.decoded_len(), 0);
        assert!(zs.into_decoded_string().is_empty());
    }

    #[test]
    fn test_get() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        assert_eq!(zs.get(0..3), Some("E\u{33a}"));
        assert!(zs.get(0..2).is_none());
        assert!(zs.get(0..42).is_none());
    }

    #[test]
    fn test_get_unchecked() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        unsafe {
            assert_eq!(zs.get_unchecked(..3), "E\u{33a}");
        }
    }

    #[test]
    fn test_indexing() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        assert_eq!(&zs[0..3], "E\u{33a}");
        assert_eq!(&zs[..3], "E\u{33a}");
        assert_eq!(&zs[0..=2], "E\u{33a}");
        assert_eq!(&zs[..=2], "E\u{33a}");
        assert_eq!(zs[..], zs);
    }

    #[test]
    #[should_panic]
    fn test_index_panic() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        let _a = &zs[0..2];
    }

    #[test]
    fn test_decoded_bytes() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        assert_eq!(zs.decoded_bytes().nth(0), Some(b'Z'));
        assert_eq!(zs.decoded_bytes().nth(2), Some(b'l'));
        assert_eq!(zs.decoded_bytes().last(), Some(b'o'));
        let mut dcb = zs.decoded_bytes();
        assert_eq!(dcb.next(), Some(b'Z'));
        let dcb2 = dcb.clone();
        assert_eq!(dcb.count(), 4);
        assert_eq!(dcb2.last(), Some(b'o'));
    }

    #[test]
    fn test_decoded_chars() {
        let zs = ZalgoString::new("Zalgo").unwrap();
        assert_eq!(zs.decoded_chars().nth(0), Some('Z'));
        assert_eq!(zs.decoded_chars().nth(2), Some('l'));
        assert_eq!(zs.decoded_chars().last(), Some('o'));
        let mut dcc = zs.decoded_chars();
        assert_eq!(dcc.next(), Some('Z'));
        let dcc2 = dcc.clone();
        assert_eq!(dcc.count(), 4);
        assert_eq!(dcc2.last(), Some('o'));
    }

    #[test]
    fn test_into_combining_chars() {
        let zs = ZalgoString::new("Hi").unwrap();
        assert_eq!(zs.into_combining_chars(), "\u{328}\u{349}");
        let zs = ZalgoString::new("").unwrap();
        assert_eq!(zs.into_combining_chars(), "");
    }
}
