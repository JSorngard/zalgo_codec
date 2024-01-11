use crate::{decode_byte_pair, ZalgoString};
use core::iter::FusedIterator;

/// An iterator over the decoded bytes of a [`ZalgoString`].
///
/// This struct is obtained by calling the [`decoded_bytes`](ZalgoString::decoded_bytes) method on a [`ZalgoString`].
/// See its documentation for more.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DecodedBytes<'a>(core::str::Bytes<'a>);

impl<'a> DecodedBytes<'a> {
    #[inline]
    pub(crate) fn new(zs: &'a ZalgoString) -> Self {
        Self(zs.as_combining_chars().bytes())
    }
}

impl<'a> Iterator for DecodedBytes<'a> {
    type Item = u8;
    #[inline]
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

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0
            .nth(2 * n)
            .zip(self.0.next())
            .map(|(odd, even)| decode_byte_pair(odd, even))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.0
            .len()
            // Check if there are at least two bytes left
            .checked_sub(2)
            .and_then(|l| {
                self.0
                    // Get the next to last,
                    .nth(l)
                    // and the last
                    .zip(self.0.next())
                    // and decode them
                    .map(|(odd, even)| decode_byte_pair(odd, even))
            })
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count() / 2
    }
}

impl<'a> DoubleEndedIterator for DecodedBytes<'a> {
    #[inline]
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

impl<'a> DecodedChars<'a> {
    pub(crate) fn new(zs: &'a ZalgoString) -> Self {
        Self(zs.decoded_bytes())
    }
}

impl<'a> Iterator for DecodedChars<'a> {
    type Item = char;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(char::from)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(char::from)
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last().map(char::from)
    }
}

impl<'a> DoubleEndedIterator for DecodedChars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(char::from)
    }
}

impl<'a> FusedIterator for DecodedChars<'a> {}
impl<'a> ExactSizeIterator for DecodedChars<'a> {}
