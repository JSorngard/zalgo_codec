//! Contains the definition of the error type used by the encoding functions in the crate.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The error returned by [`zalgo_encode`](crate::zalgo_encode), [`ZalgoString::new`](crate::ZalgoString::new), and [`zalgo_wrap_python`](crate::zalgo_wrap_python)
/// if they encounter a byte they can not encode.
///
/// Only implements the [`Error`](std::error::Error) trait if the `std` feature is enabled.
pub enum Error {
    /// Represents a valid ASCII character that is outside of the encodable set.
    /// The first `u8` in the variant is the byte value of the character, the first `usize`
    /// is the 1-indexed line number where the character occured, the second `usize` is
    /// the 1-indexed column in which the character occured and the `&str` is a description
    /// of the character.
    UnencodableAscii(u8, usize, usize, &'static str),
    /// Represents some other character.
    /// The two `usize`s represent the same thing as in the `UnencodableAscii` variant,
    NotAscii(char, usize, usize),
}

impl Error {
    /// Returns the 1-indexed line number of the line on which the unencodable byte occured.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("❤️").err().map(|e| e.line()), Some(1));
    /// assert_eq!(zalgo_encode("a\nb\nc\r\n").err().map(|e| e.line()), Some(3));
    /// ```
    #[inline]
    #[must_use = "the method returns a new valus and does not modify `self`"]
    pub const fn line(&self) -> usize {
        match self {
            Self::UnencodableAscii(_, line, _, _) | Self::NotAscii(_, line, _) => *line,
        }
    }

    /// Returns the 1-indexed column where the unencodable byte occured.
    /// Columns are counted from left to right and the count resets for each new line.
    ///
    /// # Example
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("I ❤️ 🎂").err().map(|e| e.column()), Some(3));
    /// assert_eq!(zalgo_encode("I\n❤️\n🎂").err().map(|e|e.column()), Some(1));
    /// ```
    #[inline]
    #[must_use = "the method returns a new valus and does not modify `self`"]
    pub const fn column(&self) -> usize {
        match self {
            Self::UnencodableAscii(_, _, column, _) | Self::NotAscii(_, _, column) => *column,
        }
    }

    /// Returns the value of the unencodable character.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("CRLF\r\n").err().map(|e| e.char()), Some('\r'));
    ///
    /// // Only the first unicode character is returned. E.g. some emojis consist of
    /// // many unicode characters:
    /// assert_eq!(zalgo_encode("❤️").err().map(|e| e.char()), Some('❤'));
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn char(&self) -> char {
        match self {
            Self::UnencodableAscii(byte, _, _, _) => *byte as char,
            Self::NotAscii(char, _, _) => *char,
        }
    }

    /// Return a representation of the unencodable byte.
    /// This exists if the character is an unencodable ASCII character.
    /// If it is some other unicode character we only know its first byte, so we can not
    /// accurately represent it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::zalgo_encode;
    /// assert_eq!(zalgo_encode("\r").err().map(|e| e.representation()).flatten(), Some("Carriage Return"));
    /// assert_eq!(zalgo_encode("❤️").err().map(|e| e.representation()).flatten(), None);
    /// ```
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn representation(&self) -> Option<&'static str> {
        match self {
            Self::UnencodableAscii(_, _, _, repr) => Some(*repr),
            Self::NotAscii(_, _, _) => None,
        }
    }

    /// Returns whether the error is the [NotAscii](Error::NotAscii) variant.
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn is_not_ascii(&self) -> bool {
        matches!(self, Self::NotAscii(_, _, _))
    }

    /// Returns whether the error is the [UnencodableAscii](Error::UnencodableAscii) variant.
    #[inline]
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn is_unencodable_ascii(&self) -> bool {
        matches!(self, Self::UnencodableAscii(_, _, _, _))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnencodableAscii(byte, line, column, repr) => write!(
                f,
                "line {line} at column {column}: can not encode ASCII \"{repr}\" character with byte value {byte}"
            ),
            Self::NotAscii(char, line, column) => write!(
                f,
                "line {line} at column {column}: unicode character '{char}' (U+{:x}) is not an ASCII character",
                u32::from(*char)
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(test)]
mod test {
    use super::Error;

    #[test]
    fn test_error() {
        let err = Error::NotAscii('å', 1, 7);
        assert_eq!(err.char(), 'å');
        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 7);
        assert_eq!(err.representation(), None);

        let err = Error::UnencodableAscii(13, 1, 2, "Carriage Return");
        assert_eq!(err.char(), '\r');
        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 2);
        assert_eq!(err.representation(), Some("Carriage Return"));
    }
}
