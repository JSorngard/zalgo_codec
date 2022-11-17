/// This is a crate implementing the zalgo encode and decode functions
/// originally written in Python by [Scott Conner](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor).

static UNKNOWN_CHAR_MAP: &[(u8, &str)] = &[
    (0, "Null (\\0)"),
    (1, "SOH"),
    (2, "STX"),
    (3, "ETX"),
    (4, "EOT"),
    (5, "ENQ"),
    (6, "ACK"),
    (7, "BEL"),
    (8, "Backspace (\\b)"),
    (9, "Tab (did you mean to indent with spaces?) (\\t)"),
    (11, "Vertical Tab (\\v)"),
    (12, "Form Feed (\\f)"),
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

fn get_nonprintable_char_repr(key: u8) -> Option<&'static str> {
    match UNKNOWN_CHAR_MAP
        .binary_search_by(|(k, _)| k.cmp(&key))
        .map(|x| UNKNOWN_CHAR_MAP[x].1)
    {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

fn is_zalgo_encodable(character: u8) -> bool {
    (32..=126).contains(&character) || character == b'\n'
}

struct UnknownCharacterError {
    descriptor: String,
}

impl UnknownCharacterError {
    fn new(character: u8, line: u64) -> Self {
        UnknownCharacterError {
            descriptor: if character < 128 {
                match get_nonprintable_char_repr(character) {
                    Some(repr) => format!("{line}: Cannot encode {repr} character"),
                    None => format!("{line}: ASCII {character}"),
                }
            } else {
                format!("{line}: Attempt to encode UTF8 character sequence (this program only can encode ascii non-control characters and newlines)")
            },
        }
    }
}

impl std::string::ToString for UnknownCharacterError {
    fn to_string(&self) -> String {
        self.descriptor.clone()
    }
}

pub fn zalgo_compress(s: &str) -> Result<String, String> {
    let mut line = 1;
    let mut result: Vec<u8> = vec![b'E'];
    for c in s.bytes() {
        if c == b'\r' {
            return Err("Non-unix line endings detected".into());
        }

        if c == b'\n' {
            line += 1;
        }

        if !is_zalgo_encodable(c) {
            return Err(UnknownCharacterError::new(c, line).to_string());
        }

        let v: u8 = if c == b'\n' { 111 } else { (c - 11) % 133 - 21 };
        result.push((v >> 6) & 1 | 0b11001100);
        result.push((v & 63) | 0b10000000);
    }

    match std::str::from_utf8(&result) {
        Ok(s) => Ok(s.into()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn zalgo_decompress(compressed: &str) -> Result<String, String> {
    let bytes: Vec<u8> = compressed
        .bytes()
        .skip(1)
        .step_by(2)
        .zip(compressed.bytes().skip(2).step_by(2))
        .map(|(h, c)| (((h << 6 & 64 | c & 63) + 22) % 133 + 10))
        .collect();

    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(s.into()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify() {
        const TEST_STRING: &str = "the greatest adventure is going to bed";
        let out_string = std::str::from_utf8(b"E\xcd\x94\xcd\x88\xcd\x85\xcc\x80\xcd\x87\xcd\x92\xcd\x85\xcd\x81\xcd\x94\xcd\x85\xcd\x93\xcd\x94\xcc\x80\xcd\x81\xcd\x84\xcd\x96\xcd\x85\xcd\x8e\xcd\x94\xcd\x95\xcd\x92\xcd\x85\xcc\x80\xcd\x89\xcd\x93\xcc\x80\xcd\x87\xcd\x8f\xcd\x89\xcd\x8e\xcd\x87\xcc\x80\xcd\x94\xcd\x8f\xcc\x80\xcd\x82\xcd\x85\xcd\x84").unwrap();
        assert_eq!(zalgo_compress(TEST_STRING).unwrap(), out_string,)
    }

    #[test]
    fn lossless() {
        const TEST_STRING: &str =
            "I'll have you know I graduated top of my class in the Navy Seals";
        assert_eq!(
            zalgo_decompress(&zalgo_compress(TEST_STRING).unwrap()).unwrap(),
            TEST_STRING
        );
    }

    #[test]
    fn newlines() {
        assert_eq!(&zalgo_compress("\n").unwrap(), "Eͯ",);
    }

    #[test]
    fn check_errors() {
        assert!(zalgo_compress("We got the Ä Ö Å, you aint got the Ä Ö Å").is_err());
        assert!(zalgo_compress("\t").is_err());
    }
}
