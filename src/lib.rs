/// This is a crate implementing the zalgo encode and decode functions
/// originally written in Python by [Scott Conner](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor).

static UNKNOWN_CHAR_MAP: &[(u8, & str)] = &[
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
    match UNKNOWN_CHAR_MAP.binary_search_by(|(k, _)| k.cmp(&key)).map(|x| UNKNOWN_CHAR_MAP[x].1) {
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
            }
        }
    }
}

impl std::string::ToString for UnknownCharacterError {
    fn to_string(&self) -> String {
        self.descriptor.clone()
    }
}

pub fn zalgo_encode(s: &str) -> Result<String, String> {
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

        let v: u8 = (c - 11) % 133 - 21;
        result.push((v >> 6) & 1 | 0b11001100);
        result.push((v & 63) | 0b10000000);
    }

    match std::str::from_utf8(&result) {
        Ok(s) => Ok(s.into()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn zalgo_decode(b: &str) -> Result<String, String> {
    let bytes: Vec<u8> = b
        .bytes()
        .skip(1)
        .step_by(2)
        .zip(b.bytes().skip(2).step_by(2))
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
    fn it_works() {
        let test_string = "the greatest adventure is going to bed";
        assert_eq!(
            zalgo_decode(&zalgo_encode(test_string).unwrap()).unwrap(),
            test_string
        )
    }
}
