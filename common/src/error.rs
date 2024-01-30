//! Contains the definition of the error type used by the encoding functions in the crate.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// The error returned by [`zalgo_encode`](crate::zalgo_encode), [`ZalgoString::new`](crate::ZalgoString::new), and [`zalgo_wrap_python`](crate::zalgo_wrap_python)
/// if they encounter a byte they can not encode.
///
/// Only implements the [`Error`](std::error::Error) trait if the `std` feature is enabled.
pub struct Error {
    unencodable_character: char,
    line: usize,
    column: usize,
    index: usize,
}

impl Error {
    /// Creates a new `Error`.
    ///
    /// # Note
    ///
    /// This associated method does not check the validity of its inputs,
    /// and just constructs a new `Error` instance.
    #[inline]
    #[must_use = "this associated method does not modify its inputs and just returns a new value"]
    pub(crate) const fn new(
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
        }
    }

    /// Returns the 1-indexed line number of the line on which the unencodable byte occured.
    ///
    /// # Examples
    ///
    /// ```
    /// # use zalgo_codec_common::{Error, zalgo_encode};
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
    /// # use zalgo_codec_common::{Error, zalgo_encode};
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
    /// The ❤️ emoji consists of two characters, the heart `U+2764` and the color variant selector `U+FE0F`
    /// Since the heart in not encodable, that is the place where the error is generated:
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
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "at string index {}, on line {} at column {}: can not encode character {:?}",
            self.index(),
            self.line(),
            self.column(),
            self.char()
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(test)]
mod test {
    use super::Error;

    #[test]
    fn test_error() {
        let err = Error {
            line: 1,
            column: 7,
            unencodable_character: 'å',
            index: 6,
        };
        assert_eq!(err.char(), 'å');
        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 7);

        let err2 = Error {
            line: 1,
            column: 2,
            unencodable_character: '\r',
            index: 1,
        };
        assert_eq!(err2.line(), 1);
        assert_eq!(err2.column(), 2);

        assert_ne!(err, err2);
        let err3 = err;
        assert_eq!(err, err3);
    }
}
