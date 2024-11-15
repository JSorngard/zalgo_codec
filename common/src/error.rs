//! Contains the definition of the error type used by the encoding functions in the crate.

use core::{fmt, str::Utf8Error};
#[cfg(feature = "std")]
use std::backtrace::Backtrace;

#[cfg(not(feature = "std"))]
use alloc::{string::FromUtf8Error, vec::Vec};
#[cfg(feature = "std")]
use std::string::FromUtf8Error;

#[derive(Debug)]
/// The error returned by [`zalgo_encode`](crate::zalgo_encode), [`ZalgoString::new`](crate::ZalgoString::new), and [`zalgo_wrap_python`](crate::zalgo_wrap_python)
/// if they encounter a byte they can not encode.
pub struct EncodeError {
    unencodable_character: char,
    line: usize,
    column: usize,
    index: usize,
    #[cfg(feature = "std")]
    backtrace: Backtrace,
}

impl EncodeError {
    /// Creates a new `Error`.
    ///
    /// # Note
    ///
    /// This associated method does not check the validity of its inputs,
    /// and just constructs a new `Error` instance.
    #[inline]
    #[must_use = "this associated method does not modify its inputs and just returns a new value"]
    pub(crate) fn new(
        unencodable_character: char,
        line: usize,
        column: usize,
        index: usize,
    ) -> Self {
        Self {
            unencodable_character,
            line,
            column,
            index,
            #[cfg(feature = "std")]
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns the 1-indexed line number of the line on which the unencodable byte occured.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, zalgo_encode};
    /// assert_eq!(zalgo_encode("❤️").map_err(|e| e.line()), Err(1));
    /// assert_eq!(zalgo_encode("a\nb\nc\r\n").map_err(|e| e.line()), Err(3));
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the 1-indexed column where the unencodable byte occured.
    /// Columns are counted from left to right and the count resets for each new line.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{EncodeError, zalgo_encode};
    /// assert_eq!(zalgo_encode("I ❤️ 🎂").map_err(|e| e.column()), Err(3));
    /// assert_eq!(zalgo_encode("I\n❤️\n🎂").map_err(|e|e.column()), Err(1));
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Returns the unencodable character that caused the error.
    ///
    /// This may not match with what you see when you look at the unencoded string in a text editor since
    /// some grapheme clusters consist of many unicode characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("CRLF\r\n").map_err(|e| e.char()), Err('\r'));
    ///
    /// ```  
    /// The ❤️ emoji consists of two characters, the heart `U+2764` and the color variant selector `U+FE0F`.
    /// Since the heart is not encodable, that is the place where the error is generated:
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("❤️").map_err(|e| e.char()), Err('❤'));
    /// ```
    /// The grapheme cluster `á` consists of a normal `a` and a combining acute accent, `U+301`.
    /// The `a` can be encoded and the combining acute accent can not, so the error points only to the accent:
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("á").map_err(|e| e.char()), Err('\u{301}'))
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn char(&self) -> char {
        self.unencodable_character
    }

    /// Returns the index of the string where the unencodable character occured.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("ab\ncdë").map_err(|e| e.index()), Err(5));
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[cfg(feature = "std")]
    /// Returns a reference to a [`Backtrace`] that was captured when the error was created.
    ///
    /// See the documentation of [`Backtrace::capture`] for more information about how to make it
    /// show more information when displayed.
    #[inline]
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "can not encode {:?} character at string index {}, on line {} at column {}",
            self.char(),
            self.index(),
            self.line(),
            self.column(),
        )
    }
}

impl core::error::Error for EncodeError {}

#[derive(Debug)]
pub struct DecodeError {
    kind: DecodeErrorKind,
    #[cfg(feature = "std")]
    backtrace: Backtrace,
}

impl DecodeError {
    pub(crate) fn new(possible_error: Option<FromUtf8Error>) -> Self {
        Self {
            #[cfg(feature = "std")]
            backtrace: Backtrace::capture(),
            kind: match possible_error {
                Some(e) => DecodeErrorKind::InvalidUtf8(e),
                None => DecodeErrorKind::EmptyInput,
            },
        }
    }

    #[cfg(feature = "std")]
    /// Returns a backtrace to where the error was crated.
    ///
    /// The error was captured with [`Backtrace::capture`], see it for more information
    /// on how to make it show information when printed.
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// If the error happened because the decoding resulted in invalid UTF-8
    /// this method returns the [`Utf8Error`] that was created during the failed
    /// conversion of the bytes into a `String`.
    pub fn utf8_error(&self) -> Option<Utf8Error> {
        match self.kind {
            DecodeErrorKind::InvalidUtf8(ref e) => Some(e.utf8_error()),
            DecodeErrorKind::EmptyInput => None,
        }
    }

    /// If the input wasn't empty this function returns
    /// the bytes that resulted from the decoding
    /// and were then attempted to be converted into a `String`.
    ///
    /// This method does not allocate.
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self.kind {
            DecodeErrorKind::InvalidUtf8(e) => Some(e.into_bytes()),
            DecodeErrorKind::EmptyInput => None,
        }
    }

    /// If the error happened because the decoding resulted in invalid UTF-8,
    /// this function returns a slice of the bytes that resulted from the decoding and
    /// which were then attempted to be converted into a `String`.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self.kind {
            DecodeErrorKind::InvalidUtf8(ref e) => Some(e.as_bytes()),
            DecodeErrorKind::EmptyInput => None,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not decode the string because {}", self.kind)
    }
}

impl core::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self.kind {
            DecodeErrorKind::InvalidUtf8(ref e) => Some(e),
            DecodeErrorKind::EmptyInput => None,
        }
    }
}

/// The kind of error the caused the decoding failure.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DecodeErrorKind {
    /// The given string was empty.
    EmptyInput,
    /// Decoding the string resulted in invalid UTF-8.
    InvalidUtf8(FromUtf8Error),
}

impl fmt::Display for DecodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "the string was empty"),
            Self::InvalidUtf8(e) => write!(f, "decoding resulted in invalid utf8: {e}"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{DecodeError, EncodeError};

    #[test]
    fn test_error() {
        let err = EncodeError::new('å', 1, 7, 6);
        assert_eq!(err.char(), 'å');
        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 7);
        assert_eq!(err.index(), 6);
    }

    #[test]
    fn test_decode_error() {
        let err = DecodeError::new(None);
        assert_eq!(err.as_bytes(), None);
        assert_eq!(err.utf8_error(), None);
        assert_eq!(err.into_bytes(), None);
        let err = DecodeError::new(String::from_utf8(vec![255, 255, 255, 255, 255, 255]).err());
        assert_eq!(err.as_bytes(), Some([255; 6].as_slice()));
        assert_eq!(err.into_bytes(), Some(vec![255; 6]));
    }
}
