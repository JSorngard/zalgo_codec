//! Contains the implementation of [`ZalgoString`] as well as related iterators.
//!
//! A `ZalgoString` contains a grapheme cluster that was obtained from [`zalgo_encode`].
//! It allows for iteration over its characters and bytes in both encoded and decoded form.
//! It can be decoded in-place and the encoded information in other ZalgoStrings can be pushed
//! onto it.

mod iterators;

use crate::{decode_byte_pair, fmt, zalgo_encode, EncodeError};
use core::{ops::Index, slice::SliceIndex};
pub use iterators::{DecodedBytes, DecodedChars};
#[cfg(feature = "rkyv")]
use rkyv::bytecheck::{
    rancor::{fail, Fallible, Source},
    CheckBytes, Verify,
};

use alloc::{borrow::Cow, string::String, vec::Vec};

/// A [`String`] that has been encoded with [`zalgo_encode`].
/// This struct can be decoded in-place and also allows iteration over its characters and bytes, both in
/// decoded and encoded form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "MaybeZalgoString"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, CheckBytes)
)]
#[cfg_attr(feature = "rkyv", bytecheck(verify))]
pub struct ZalgoString(String);

#[cfg(feature = "rkyv")]
unsafe impl<C> Verify<C> for ZalgoString
where
    C: Fallible + ?Sized,
    C::Error: Source,
{
    #[inline]
    fn verify(&self, _context: &mut C) -> Result<(), C::Error> {
        if let Err(e) = crate::zalgo_decode(&self.0) {
            fail!(e);
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
struct MaybeZalgoString(String);

#[cfg(feature = "serde")]
impl TryFrom<MaybeZalgoString> for ZalgoString {
    type Error = crate::DecodeError;

    fn try_from(MaybeZalgoString(encoded_string): MaybeZalgoString) -> Result<Self, Self::Error> {
        if let Err(e) = crate::zalgo_decode(&encoded_string) {
            Err(e)
        } else {
            Ok(ZalgoString(encoded_string))
        }
    }
}

/// Allocates a `String` that contains only the character "E" and no encoded content.
impl Default for ZalgoString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ZalgoString {
    /// Creates a new empty [`ZalgoString`].
    #[inline]
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Creates a new `ZalgoString` with at least the specified capacity.
    ///
    /// If you want the ZalgoString to have capacity for x encoded characters
    /// you must reserve a capacity of 2x.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    ///
    /// // Reserve capacity for two encoded characters
    /// let mut zs = ZalgoString::with_capacity(2*2);
    ///
    /// // This ZalgoString would decode into an empty string
    /// assert_eq!(zs.decoded_len(), 0);
    ///
    /// // This allocates,
    /// let zs2 = ZalgoString::try_from("Hi")?;
    ///
    /// // but this does not reallocate `zs`
    /// let cap = zs.capacity();
    /// zs.push_zalgo_str(&zs2);
    /// assert_eq!(zs.capacity(), cap);
    ///
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    #[must_use = "this associated method return a new `ZalgoString` and does not modify the input"]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    // region: character access methods

    /// Returns the *encoded* contents of `self` as a string slice.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Oh boy!")?;
    /// assert_eq!(zs.as_str(), "̯͈̀͂͏͙́");
    /// # Ok::<(), EncodeError>(())
    /// ```
    /// Note that `ZalgoString` implements [`PartialEq`] with common string types,
    /// so the comparison in the above example could also be done directly
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// # let zs = ZalgoString::try_from("Oh boy!")?;
    /// assert_eq!(zs, "̯͈̀͂͏͙́");
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// assert_eq!(zs.get(0..2), Some("\u{33a}"));
    ///
    /// // indices not on UTF-8 sequence boundaries
    /// assert!(zs.get(0..3).is_none());
    ///
    /// // out of bounds
    /// assert!(zs.get(..42).is_none());
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// unsafe {
    ///     assert_eq!(zs.get_unchecked(..2), "\u{33a}");
    /// }
    /// # Ok::<(), EncodeError>(())
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
    /// # Example
    ///
    /// Iterate through the encoded [`char`]s:
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("42")?;
    /// let mut chars = zs.chars();
    /// assert_eq!(chars.next(), Some('\u{314}'));
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// let mut ci = zs.char_indices();
    /// assert_eq!(ci.next(), Some((0,'\u{33a}')));
    /// // Note the 2 here, the combining characters take up two bytes.
    /// assert_eq!(ci.next(), Some((2, '\u{341}')));
    /// // The final character begins at position 8
    /// assert_eq!(ci.next_back(), Some((8, '\u{34f}')));
    /// // even though the length in bytes is 10
    /// assert_eq!(zs.len(), 10);
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zlgoa")?;
    /// let mut decoded_chars = zs.decoded_chars();
    /// assert_eq!(decoded_chars.next(), Some('Z'));
    /// assert_eq!(decoded_chars.next_back(), Some('a'));
    /// assert_eq!(decoded_chars.next(), Some('l'));
    /// assert_eq!(decoded_chars.next(), Some('g'));
    /// assert_eq!(decoded_chars.next_back(), Some('o'));
    /// assert_eq!(decoded_chars.next(), None);
    /// assert_eq!(decoded_chars.next_back(), None);
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo\n He comes!")?;
    /// assert_eq!(zs.into_string(), "̺͇́͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ");
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let s = "Zalgo";
    /// let zs = ZalgoString::try_from(s)?;
    /// assert_eq!(s, zs.into_decoded_string());
    /// # Ok::<(), EncodeError>(())
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
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// assert_eq!(&zs.as_bytes()[0..4], &[204, 186, 205, 129]);
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a reference and does not modify `self`"]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns an iterator over the encoded bytes of the `ZalgoString`.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Bytes")?;
    /// let mut bytes = zs.bytes();
    /// assert_eq!(bytes.nth(5), Some(148));
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// let mut decoded_bytes = zs.decoded_bytes();
    /// assert_eq!(decoded_bytes.next(), Some(90));
    /// assert_eq!(decoded_bytes.next_back(), Some(111));
    /// assert_eq!(decoded_bytes.collect::<Vec<u8>>(), vec![97, 108, 103]);
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// assert_eq!(zs.into_bytes(), vec![204, 186, 205, 129, 205, 140, 205, 135, 205, 143]);
    /// # Ok::<(), EncodeError>(())
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Zalgo")?;
    /// assert_eq!(b"Zalgo".to_vec(), zs.into_decoded_bytes());
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn into_decoded_bytes(self) -> Vec<u8> {
        let mut w = 0;
        let mut bytes = self.into_bytes();
        for r in (0..bytes.len()).step_by(2) {
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
    /// This length is twice the length of the original `String`.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::try_from("Z")?;
    /// assert_eq!(zs.len(), 2);
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the capacity of the underlying encoded string in bytes.
    ///
    /// The `ZalgoString` is preallocated to the needed capacity of twice the length
    /// of the original unencoded `String`.
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
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let s = "Zalgo, He comes!";
    /// let zs = ZalgoString::try_from(s)?;
    /// assert_eq!(s.len(), zs.decoded_len());
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn decoded_len(&self) -> usize {
        self.len() / 2
    }

    /// Returns whether the `ZalgoString` is empty.
    ///
    /// # Example
    ///
    /// Basic usage
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let zs = ZalgoString::new();
    /// assert!(zs.is_empty());
    ///
    /// let zs = ZalgoString::try_from("Blargh")?;
    /// assert!(!zs.is_empty());
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // endregion: metadata methods

    /// Appends the combining characters of a different `ZalgoString` to the end of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let (s1, s2) = ("Zalgo", ", He comes!");
    ///
    /// let mut zs1 = ZalgoString::try_from(s1)?;
    /// let zs2 = ZalgoString::try_from(s2)?;
    ///
    /// zs1.push_zalgo_str(&zs2);
    ///
    /// assert_eq!(zs1.into_decoded_string(), format!("{s1}{s2}"));
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    pub fn push_zalgo_str(&mut self, zalgo_string: &Self) {
        self.0.push_str(zalgo_string.as_str());
    }

    /// Encodes the given string and pushes it onto `self`.
    ///
    /// This method encodes the input string into an intermediate allocation and then appends
    /// the combining characters of the result to the end of `self`. The append step can
    /// also reallocate if the capacity is not large enough.
    ///
    /// See [`push_zalgo_str`](ZalgoString::push_zalgo_str) for a method that does not hide the
    /// intermediate allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the given string contains a character that's not a printable ASCII
    /// or newline character.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let (s1, s2) = ("Zalgo", ", He comes!");
    ///
    /// let mut zs = ZalgoString::try_from(s1)?;
    ///
    /// zs.encode_and_push_str(s2)?;
    ///
    /// assert_eq!(zs.into_decoded_string(), format!("{s1}{s2}"));
    /// # Ok::<(), EncodeError>(())
    /// ```
    pub fn encode_and_push_str(&mut self, string: &str) -> Result<(), EncodeError> {
        self.push_zalgo_str(&ZalgoString::try_from(string)?);
        Ok(())
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
    /// Keep in mind that an encoded ASCII character takes up two bytes,
    /// which means that the total length in bytes is always an even number.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let mut zs = ZalgoString::try_from("Zalgo")?;
    /// let c = zs.capacity();
    /// zs.reserve(4);
    /// assert!(zs.capacity() >= c + 4);
    /// # Ok::<(), EncodeError>(())
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
    /// Keep in mind that an encoded ASCII character takes up two bytes,
    /// which means that the total length in bytes is always an odd number.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let mut zs = ZalgoString::try_from("Zalgo")?;
    /// let c = zs.capacity();
    /// zs.reserve_exact(4);
    /// assert!(zs.capacity() >= c + 4);
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.0.reserve_exact(additional)
    }

    // endregion: capacity manipulation methods

    // region: length manipulation methods

    /// Shortens the `ZalgoString` to the specified length.
    ///
    /// A `ZalgoString` always takes up an even number of bytes as all encoded characters take up two bytes.
    ///
    /// If `new_len` is larger than its current length, this has no effect.
    ///
    /// This method has no effect of the allocated capacity.
    ///
    /// # Panics
    ///
    /// Panics if `new_len` is odd.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let mut zs = ZalgoString::try_from("Zalgo")?;
    /// zs.truncate(4);
    /// assert_eq!(zs, "\u{33a}\u{341}");
    /// assert_eq!(zs.into_decoded_string(), "Za");
    /// # Ok::<(), EncodeError>(())
    /// ```
    /// Panics if `new_len` is odd:
    /// ```should_panic
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let mut zs = ZalgoString::try_from("Zalgo")?;
    /// zs.truncate(1);
    /// # Ok::<(), EncodeError>(())
    /// ```
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert_eq!(new_len % 2, 0, "the new length must be even");
            self.0.truncate(new_len)
        }
    }

    /// Clears this `ZalgoString`, removing all contents.
    ///
    /// This means the ZalgoString will have a length of zero, but it does not affect its capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    /// let mut zs = ZalgoString::try_from("Zalgo")?;
    /// let cap = zs.capacity();
    ///
    /// zs.clear();
    ///
    /// assert!(zs.is_empty());
    /// assert_eq!(zs.capacity(), cap);
    /// # Ok::<(), EncodeError>(())
    /// ```
    pub fn clear(&mut self) {
        self.0.clear()
    }

    // endregion: length manipulation methods
}

impl TryFrom<&str> for ZalgoString {
    type Error = EncodeError;

    /// Encodes the given string slice and stores the result in a new allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the input string contains bytes that don’t correspond to printable ASCII characters or newlines.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, ZalgoString};
    ///
    /// assert_eq!(ZalgoString::try_from("Zalgo")?, "̺͇́͌͏");
    ///
    /// # Ok::<(), EncodeError>(())
    /// ```
    ///
    /// Can only encode printable ASCII and newlines:
    ///
    /// ```
    /// # use zalgo_codec_common::ZalgoString;
    /// assert!(ZalgoString::try_from("❤️").is_err());
    /// assert!(ZalgoString::try_from("\r").is_err());
    /// ```
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        zalgo_encode(s).map(Self)
    }
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

impl<I: SliceIndex<str>> Index<I> for ZalgoString {
    type Output = I::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.as_str().index(index)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::{
        format,
        string::{String, ToString},
    };

    #[test]
    fn check_into_decoded_string() {
        let s = "Zalgo\n He comes!";
        let zs: ZalgoString = ZalgoString::try_from(s).unwrap();
        assert_eq!(zs.into_decoded_string(), s);

        let zs = ZalgoString::new();
        assert_eq!(zs.into_decoded_string(), String::new());
    }

    #[test]
    fn check_string_from_zalgo_string() {
        let zs = ZalgoString::try_from("Zalgo\n He comes!").unwrap();
        assert_eq!(zs.to_string(), "̺͇́͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ");
        assert_eq!(zs.into_string(), "̺͇́͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ");

        let zs = ZalgoString::new();
        assert_eq!(zs.into_string(), String::new());
    }

    #[test]
    fn check_partial_eq() {
        let enc = "̺͇́͌͏̨ͯ̀̀̓ͅ͏͍͓́ͅ";
        let zs = ZalgoString::try_from("Zalgo\n He comes!").unwrap();
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
        let mut zs = ZalgoString::try_from(s1).unwrap();
        let zs2 = ZalgoString::try_from(s2).unwrap();
        zs.push_zalgo_str(&zs2);
        assert_eq!(zs.clone().into_decoded_string(), format!("{s1}{s2}"));
        zs += &zs2;
        assert_eq!(
            (zs + &zs2).into_decoded_string(),
            format!("{s1}{s2}{s2}{s2}")
        );
    }

    #[test]
    fn check_as_str() {
        assert_eq!(
            ZalgoString::try_from("Hi").unwrap().as_str(),
            "\u{328}\u{349}"
        );
        assert_eq!(ZalgoString::try_from("").unwrap().as_str(), "");
    }

    #[test]
    fn check_decoded_chars() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!("oglaZ", zs.decoded_chars().rev().collect::<String>());
    }

    #[test]
    fn test_reserve() {
        let mut zs = ZalgoString::try_from("Zalgo").unwrap();
        zs.reserve(5);
        assert!(zs.capacity() >= 10 + 5);
        let c = zs.capacity();
        zs.reserve(1);
        assert_eq!(zs.capacity(), c);
    }

    #[test]
    fn test_reserve_exact() {
        let mut zs = ZalgoString::try_from("Zalgo").unwrap();
        zs.reserve_exact(5);
        assert_eq!(zs.capacity(), 10 + 5);
        let c = zs.capacity();
        zs.reserve_exact(1);
        assert_eq!(zs.capacity(), c);
    }

    #[test]
    fn test_truncate() {
        let mut zs = ZalgoString::try_from("Zalgo").unwrap();
        zs.truncate(100);
        assert_eq!(zs, "\u{33a}\u{341}\u{34c}\u{347}\u{34f}");
        zs.truncate(4);
        assert_eq!(zs, "\u{33a}\u{341}");
        assert_eq!(zs.into_decoded_string(), "Za");
    }

    #[test]
    #[should_panic]
    fn test_truncate_panic() {
        let mut zs = ZalgoString::try_from("Zalgo").unwrap();
        zs.truncate(1)
    }

    #[test]
    fn test_default() {
        assert_eq!(ZalgoString::try_from("").unwrap(), ZalgoString::default());
    }

    #[test]
    fn test_with_capacity() {
        let mut zs = ZalgoString::with_capacity(10.try_into().unwrap());
        assert_eq!(zs.capacity(), 10);
        zs.encode_and_push_str("Hi!").unwrap();
        assert_eq!(zs.capacity(), 10);
        zs.encode_and_push_str("I am a dinosaur!").unwrap();
        assert!(zs.capacity() > 10);
    }

    #[test]
    fn test_as_str() {
        fn test_fn(_: &str) {}
        let s = "Zalgo";
        let zs = ZalgoString::try_from(s).unwrap();
        let encd = zalgo_encode(s).unwrap();
        test_fn(zs.as_str());
        assert_eq!(zs.as_str(), encd);
    }

    #[test]
    fn test_chars() {
        let s = "Zalgo";
        let zs = ZalgoString::try_from(s).unwrap();
        let encd = zalgo_encode(s).unwrap();
        for (a, b) in zs.chars().zip(encd.chars()) {
            assert_eq!(a, b);
        }
        assert_eq!(zs.chars().nth(1), Some('\u{341}'));
    }

    #[test]
    fn test_char_indices() {
        let s = "Zalgo";
        let zs = ZalgoString::try_from(s).unwrap();
        let encd = zalgo_encode(s).unwrap();
        for (a, b) in zs.char_indices().zip(encd.char_indices()) {
            assert_eq!(a, b);
        }
        assert_eq!(zs.char_indices().nth(1), Some((2, '\u{341}')));
    }

    #[test]
    fn test_as_bytes() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!(
            zs.as_bytes(),
            &[204, 186, 205, 129, 205, 140, 205, 135, 205, 143]
        );
    }

    #[test]
    fn test_bytes() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!(zs.bytes().next(), Some(204));
        assert_eq!(zs.bytes().nth(2), Some(205));
    }

    #[test]
    fn test_is_empty() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert!(!zs.is_empty());
        assert!(ZalgoString::default().is_empty());
    }

    #[test]
    fn test_encode_and_push_str() {
        let mut zs = ZalgoString::default();
        assert!(zs.encode_and_push_str("Zalgo").is_ok());
        assert!(zs.encode_and_push_str("Å").is_err());
        assert_eq!(zs.into_decoded_string(), "Zalgo");
    }

    #[test]
    fn test_clear() {
        let mut zs = ZalgoString::try_from("Zalgo").unwrap();
        let c = zs.capacity();
        zs.clear();
        assert_eq!(zs.capacity(), c);
        assert_eq!(zs.len(), 0);
        assert_eq!(zs.decoded_len(), 0);
        assert!(zs.is_empty());
        assert!(zs.into_decoded_string().is_empty());
    }

    #[test]
    fn test_get() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!(zs.get(0..2), Some("\u{33a}"));
        assert!(zs.get(0..1).is_none());
        assert!(zs.get(0..42).is_none());
    }

    #[test]
    fn test_get_unchecked() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        unsafe {
            assert_eq!(zs.get_unchecked(..2), "\u{33a}");
        }
    }

    #[test]
    fn test_indexing() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!(&zs[0..2], "\u{33a}");
        assert_eq!(&zs[..2], "\u{33a}");
        assert_eq!(&zs[0..=1], "\u{33a}");
        assert_eq!(&zs[..=1], "\u{33a}");
        assert_eq!(zs[..], zs);
    }

    #[test]
    #[should_panic]
    fn test_index_panic() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        let _a = &zs[0..3];
    }

    #[test]
    fn test_decoded_bytes() {
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!(zs.decoded_bytes().next(), Some(b'Z'));
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
        let zs = ZalgoString::try_from("Zalgo").unwrap();
        assert_eq!(zs.decoded_chars().next(), Some('Z'));
        assert_eq!(zs.decoded_chars().nth(2), Some('l'));
        assert_eq!(zs.decoded_chars().last(), Some('o'));
        let mut dcc = zs.decoded_chars();
        assert_eq!(dcc.next(), Some('Z'));
        let dcc2 = dcc.clone();
        assert_eq!(dcc.count(), 4);
        assert_eq!(dcc2.last(), Some('o'));
    }

    #[test]
    fn test_into_string() {
        let zs = ZalgoString::try_from("Hi").unwrap();
        assert_eq!(zs.into_string(), "\u{328}\u{349}");
        let zs = ZalgoString::try_from("").unwrap();
        assert_eq!(zs.into_string(), "");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_deserialize_zalgo_string() {
        use serde_json::from_str;
        let s = "Zalgo";
        let zs = ZalgoString::try_from(s).unwrap();
        let json = format!(r#""{}""#, zs);
        let deserialized: ZalgoString = from_str(&json).unwrap();
        assert_eq!(deserialized, zs);
        assert!(from_str::<ZalgoString>("Horse").is_err());
    }
}
