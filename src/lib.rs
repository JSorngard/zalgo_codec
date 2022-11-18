//! This is a crate implementing the zalgo encoding and decoding functions
//! [originally written in Python](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor) by Scott Conner.
//!
//! Using the functions defined in this crate you can transform an ASCII string into a unicode string that is a single
//! "character" wide. The encoding is reversible, but this string will be larger than the original in terms of bytes.
//! The crate also provides functions to encode python code and wrap the result in a decoder that
//! decodes and executes the encoded string. This way the file looks very different, but executes the same way as before.
//! This lets you do the mother of all refactoring by converting your entire python program into a single line of code.
//! Can not encode carriage returns, so files written on non-unix operating systems might not work. The file encoding
//! functions will attempt to encode files anyway by ignoring carriage returns.
//!
//! Explanation:  
//! Characters U+0300–U+036F are the combining characters for unicode Latin.
//! The fun thing about combining characters is that you can add as many of these characters
//! as you like to the original character and it does not create any new symbols,
//! it only adds symbols on top of the character. It's supposed to be used in order to
//! create characters such as á by taking a normal a and adding another character
//! to give it the mark (U+301, in this case). Fun fact, Unicode doesn't specify
//! any limit on the number of these characters.
//! Conveniently, this gives us 112 different characters we can map to,
//! which nicely maps to the ASCII character range 0x20 -> 0x7F, aka all the non-control characters.
//! The only issue is that we can't have new lines in this system, so to fix that,
//! we can simply map 0x7F (DEL) to 0x0A (LF).
//! This can be represented as (CHARACTER - 11) % 133 - 21, and decoded with (CHARACTER + 22) % 133 + 10.  
//! [Original post.](https://www.reddit.com/r/ProgrammerHumor/comments/yqof9f/the_most_upvoted_comment_picks_the_next_line_of/ivrd9ur/?context=3)

use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    str,
};

static UNKNOWN_CHAR_MAP: &[(u8, &str)] = &[
    (0, r"Null (\0)"),
    (1, "SOH"),
    (2, "STX"),
    (3, "ETX"),
    (4, "EOT"),
    (5, "ENQ"),
    (6, "ACK"),
    (7, "BEL"),
    (8, r"Backspace (\b)"),
    (9, r"Tab (\t)"),
    (11, r"Vertical Tab (\v)"),
    (12, r"Form Feed (\f)"),
    (13, r"Carriage Return (\r)"),
    (14, "SO"),
    (15, "SI"),
    (16, "DLE"),
    (17, "DC1"),
    (18, "DC2"),
    (19, "DC3"),
    (20, "DC4"),
    (21, "NAK"),
    (22, "SYN"),
    (23, "ETB"),
    (24, "CAN"),
    (25, "EM"),
    (26, "SUB"),
    (27, "ESC"),
    (28, "FS"),
    (29, "GS"),
    (30, "RS"),
    (31, "US"),
    (127, "DEL"),
];

/// Searches through the static UNKNOWN_CHAR_MAP for the given key
/// and returns the corresponding character if found.
fn get_nonprintable_char_repr(key: u8) -> Option<&'static str> {
    UNKNOWN_CHAR_MAP
        .binary_search_by(|(k, _)| k.cmp(&key))
        .map(|x| UNKNOWN_CHAR_MAP[x].1)
        .ok()
}

/// Takes in an ASCII string and "compresses" it to zalgo text
/// using a reversible encoding scheme. The resulting string should
/// only take up a single character space horizontally when displayed
/// (though this can vary between platforms depending on how they deal with unicode).
/// The resulting string will be larger than the original in terms of bytes, but it
/// can be decompressed to recover the original string using `zalgo_decode`.
/// # Example
/// ```
/// # use zalgo_codec::zalgo_encode;
/// assert_eq!(zalgo_encode("Zalgo").unwrap(), "É̺͇͌͏");
/// ```
pub fn zalgo_encode(string_to_compress: &str) -> Result<String, UnencodableCharacterError> {
    let mut line = 1;
    let mut result: Vec<u8> = vec![b'E'];

    for c in string_to_compress.bytes() {
        if c == b'\r' {
            return Err(UnencodableCharacterError::new(c, line));
        }

        if c == b'\n' {
            line += 1;
        }

        if !(32..=126).contains(&c) && c != b'\n' {
            return Err(UnencodableCharacterError::new(c, line));
        }

        let v = if c == b'\n' { 111 } else { (c - 11) % 133 - 21 };
        result.push((v >> 6) & 1 | 0b11001100);
        result.push((v & 63) | 0b10000000);
    }

    Ok(str::from_utf8(&result)
        .expect("the encoding process should not produce invalid utf8")
        .into())
}

/// zalgo-encodes an ASCII string containing python code and
/// wraps it in a decoder that decodes and executes it.
pub fn zalgo_encode_python(string_to_encode: &str) -> Result<String, UnencodableCharacterError> {
    let encoded_string = zalgo_encode(string_to_encode)?;
    Ok(format!("b='{encoded_string}'.encode();exec(''.join(chr(((h<<6&64|c&63)+22)%133+10)for h,c in zip(b[1::2],b[2::2])))"))
}

/// Takes in a string that was compressed by `zalgo_encode` and decompresses it
/// to an ASCII string.
///
/// # Example
/// ```
/// # use zalgo_codec::zalgo_decode;
/// assert_eq!(zalgo_decode("É̺͇͌͏").unwrap(), "Zalgo");
/// ```
pub fn zalgo_decode(compressed: &str) -> Result<String, str::Utf8Error> {
    let bytes: Vec<u8> = compressed
        .bytes()
        .skip(1)
        .step_by(2)
        .zip(compressed.bytes().skip(2).step_by(2))
        .map(|(odds, evens)| (((odds << 6 & 64 | evens & 63) + 22) % 133 + 10))
        .collect();

    str::from_utf8(&bytes).map(|s| s.to_owned())
}

/// Encodes the contents of the file and stores the result in another file.
/// If carriage return characters are found it will print a message and
/// attempt to encode the file anyway by ignoring them.
pub fn encode_file<P: AsRef<Path>>(in_file: P, out_file: P) -> Result<(), Box<dyn Error>> {
    let mut string_to_encode = fs::read_to_string(in_file)?;

    if string_to_encode.contains('\t') {
        eprintln!("found tabs in the file, replacing with four spaces");
        string_to_encode = string_to_encode.replace('\t', "    ");
    }

    if string_to_encode.contains('\r') {
        eprintln!(
            r"file contains the carriage return character (\r). Will attempt to encode the file anyway by ignoring it. This may result in a different file when decoded"
        );
        string_to_encode = string_to_encode.replace('\r', "");
    }

    let encoded_string = zalgo_encode(&string_to_encode)?;

    match zalgo_decode(&encoded_string) {
        Ok(s) => {
            if s != string_to_encode {
                return Err("unknown error: encoding process corrupted the input string".into());
            }
        }
        Err(e) => return Err(e.into()),
    }

    let mut out_path = PathBuf::new();
    out_path.push(&out_file);

    if out_path.exists() {
        return Err("a file already exists with the output file name".into());
    }

    fs::File::create(&out_file)?;
    fs::write(out_file, encoded_string)?;
    Ok(())
}

/// Encodes the contents of the given file and stores the result wrapped in
/// a decoder in another file. This file will still work the same
/// as the original python code. If the source file contains carriage return characters
/// this function will print a message and then attempt to encode the file anyway by ignoring them.  
/// # Notes
/// The resulting python file may not work correctly on python versions before 3.10,
/// (see [this github issue](https://github.com/DaCoolOne/DumbIdeas/issues/1)).
pub fn encode_python_file<P: AsRef<Path>>(in_file: P, out_file: P) -> Result<(), Box<dyn Error>> {
    let mut string_to_encode = fs::read_to_string(in_file)?;

    if string_to_encode.contains('\t') {
        eprintln!("found tabs in the file, replacing with four spaces");
        string_to_encode = string_to_encode.replace('\t', "    ");
    }

    if string_to_encode.contains('\r') {
        eprintln!(
            r"file contains the carriage return character (\r). Will attempt to encode the file anyway by ignoring it. This may result in a different file when decoded"
        );
        string_to_encode = string_to_encode.replace('\r', "");
    }

    let encoded_string = zalgo_encode_python(&string_to_encode)?;

    let mut out_path = PathBuf::new();
    out_path.push(&out_file);

    if out_path.exists() {
        return Err("a file already exists with the output file name".into());
    }

    fs::File::create(&out_file)?;
    fs::write(out_file, encoded_string)?;
    Ok(())
}

#[derive(Debug)]
/// The error returned by the encoding functions
/// if they encounter a character they can not encode.
/// Contains a string that references which type of character and which line caused the error.
pub struct UnencodableCharacterError {
    descriptor: String,
    character: u8,
    line: usize,
}

impl UnencodableCharacterError {
    fn new(character: u8, line: usize) -> Self {
        UnencodableCharacterError {
            descriptor: if character < 128 {
                match get_nonprintable_char_repr(character) {
                    Some(repr) => format!("line {line}: cannot encode {repr} character"),
                    None => format!("line {line}: cannot encode ASCII character #{character}"),
                }
            } else {
                format!("line {line}: attempt to encode UTF8 character sequence (this program can only encode non-control ASCII characters and newlines)")
            },
            character,
            line,
        }
    }

    /// Returns the number of the line on which the unencodable character occured.
    pub fn line_number(&self) -> usize {
        self.line
    }

    /// Returns the byte value of the unencodable character. Note that this might
    /// not be the complete representation of the character in unicode, just the first
    /// byte of it.
    pub fn unencodable_character_value(&self) -> u8 {
        self.character
    }
}

impl fmt::Display for UnencodableCharacterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.descriptor)
    }
}

impl Error for UnencodableCharacterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn verify() {
        const TEST_STRING_1: &str = "the greatest adventure is going to bed";
        let out_string = str::from_utf8(b"E\xcd\x94\xcd\x88\xcd\x85\xcc\x80\xcd\x87\xcd\x92\xcd\x85\xcd\x81\xcd\x94\xcd\x85\xcd\x93\xcd\x94\xcc\x80\xcd\x81\xcd\x84\xcd\x96\xcd\x85\xcd\x8e\xcd\x94\xcd\x95\xcd\x92\xcd\x85\xcc\x80\xcd\x89\xcd\x93\xcc\x80\xcd\x87\xcd\x8f\xcd\x89\xcd\x8e\xcd\x87\xcc\x80\xcd\x94\xcd\x8f\xcc\x80\xcd\x82\xcd\x85\xcd\x84").unwrap();
        assert_eq!(zalgo_encode(TEST_STRING_1).unwrap(), out_string);

        const TEST_STRING_2: &str =
            "I'll have you know I graduated top of my class in the Navy Seals";
        assert_eq!(
            zalgo_decode(&zalgo_encode(TEST_STRING_2).unwrap()).unwrap(),
            TEST_STRING_2
        );

        const ASCII_CHAR_TABLE: &str = r##"ABCDEFGHIJKLMNOPQRSTUVXYZabcdefghijklmnopqrstuvxyz1234567890 !"#$%&'()*+,-\./:;<=>?@"##;
        assert_eq!(
            zalgo_decode(&zalgo_encode(ASCII_CHAR_TABLE).unwrap()).unwrap(),
            ASCII_CHAR_TABLE
        );
    }

    #[test]
    fn newlines() {
        assert_eq!(&zalgo_encode("\n").unwrap(), "Eͯ",);
        const TEST_STRING: &str = "The next sentence is true.\nThe previous sentence is false.";
        assert_eq!(
            zalgo_decode(&zalgo_encode(TEST_STRING).unwrap()).unwrap(),
            TEST_STRING,
        );
    }

    #[test]
    fn check_errors() {
        assert!(zalgo_encode("We got the Ä Ö Å, you aint got the Ä Ö Å").is_err());
        assert!(zalgo_encode("\t").is_err());
        assert!(zalgo_encode("\r").is_err());
        assert!(zalgo_encode("\0").is_err());
    }

    #[test]
    fn file_encoding() {
        let mut lorem_path = PathBuf::new();
        let mut zalgo_path = PathBuf::new();
        lorem_path.push("tests");
        lorem_path.push("lorem.txt");
        zalgo_path.push("tests");
        zalgo_path.push("zalgo.txt");
        encode_file(&lorem_path, &zalgo_path).unwrap();
        assert!(encode_file(&lorem_path, &zalgo_path).is_err());
        let zalgo_text = fs::read_to_string(&zalgo_path).unwrap();
        let lorem_text = fs::read_to_string(lorem_path).unwrap();
        assert_eq!(
            zalgo_decode(&zalgo_text).unwrap(),
            //remove carriage return on windows
            lorem_text.replace("\r", "")
        );
        fs::remove_file(zalgo_path).unwrap();
    }

    #[test]
    fn python_encoding() {
        let mut lorem_path = PathBuf::new();
        let mut zalgo_path = PathBuf::new();
        lorem_path.push("tests");
        lorem_path.push("lorem.py");
        zalgo_path.push("tests");
        zalgo_path.push("zalgo.py");
        encode_python_file(&lorem_path, &zalgo_path).unwrap();
        let _zalgo_text = fs::read_to_string(&zalgo_path).unwrap();
        let _lorem_text = fs::read_to_string(lorem_path).unwrap();
        fs::remove_file(zalgo_path).unwrap();
    }
}
