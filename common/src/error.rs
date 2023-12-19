//! Contains the definition of the error type used by the eoncoding functions in the crate.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// but the `u8` is only the first byte of the character.
    NotAscii(u8, usize, usize),
}

impl Error {
    /// Returns the 1-indexed line number of the line on which the unencodable byte occured.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().line(), 1);
    /// assert_eq!(zalgo_encode("a\nb\nc\r\n").err().unwrap().line(), 3);
    /// ```
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
    /// assert_eq!(zalgo_encode("I ❤️ 🎂").err().unwrap().column(), 3);
    /// assert_eq!(zalgo_encode("I\n❤️\n🎂").err().unwrap().column(), 1);
    /// ```
    #[must_use = "the method returns a new valus and does not modify `self`"]
    pub const fn column(&self) -> usize {
        match self {
            Self::UnencodableAscii(_, _, column, _) | Self::NotAscii(_, _, column) => *column,
        }
    }

    /// Returns the value of the first byte of the unencodable character.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("\r").err().unwrap().byte(), 13);
    /// ```
    /// Note that this might not be the complete representation of
    /// the character in unicode, just the first byte of it.
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().byte(), 226);
    /// // Even though
    /// assert_eq!("❤️".as_bytes(), &[226, 157, 164, 239, 184, 143])
    /// ```
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn byte(&self) -> u8 {
        match self {
            Self::UnencodableAscii(byte, _, _, _) | Self::NotAscii(byte, _, _) => *byte,
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
    /// assert_eq!(zalgo_encode("\r").err().unwrap().representation(), Some("Carriage Return"));
    /// assert_eq!(zalgo_encode("❤️").err().unwrap().representation(), None);
    /// ```
    #[must_use = "the method returns a new value and does not modify `self`"]
    pub const fn representation(&self) -> Option<&'static str> {
        match self {
            Self::UnencodableAscii(_, _, _, repr) => Some(*repr),
            Self::NotAscii(_, _, _) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnencodableAscii(byte, line, column, repr) => write!(
                f,
                "line {line} at column {column}: can not encode ASCII \"{repr}\" character with byte value {byte}"
            ),
            Self::NotAscii(byte, line, column) => write!(
                f,
                "line {line} at column {column}: byte value {byte} does not correspond to an ASCII character"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
