//! A crate for converting an ASCII text string or file to a single unicode character.
//! Can also do the same to python code while still letting the code run as before by wrapping it in a decoder.
//! This crate provides the non-macro functionality of the crate [`zalgo-codec`](https://crates.io/crates/zalgo-codec).

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
/// # use zalgo_codec_common::zalgo_encode;
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
/// # use zalgo_codec_common::zalgo_decode;
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

/// Decodes the contents of a file that has been encoded with `encode_file`
/// and stores the result in another file.
pub fn decode_file<P: AsRef<Path>>(in_file: P, out_file: P) -> Result<(), Box<dyn Error>> {
    let mut string_to_decode = fs::read_to_string(in_file)?;

    if string_to_decode.contains('\r') {
        eprintln!(
            r"file contains the carriage return character (\r). Will attempt to decode the file anyway by ignoring it"
        );
        string_to_decode = string_to_decode.replace('\r', "");
    }

    let decoded_string = zalgo_decode(&string_to_decode)?;

    let mut out_path = PathBuf::new();
    out_path.push(&out_file);

    if out_path.exists() {
        return Err("a file already exists with the output file name".into());
    }

    fs::File::create(&out_file)?;
    fs::write(out_file, decoded_string)?;
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
