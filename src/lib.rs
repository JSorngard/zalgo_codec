/// This is a crate implementing the zalgo encode and decode functions
/// originally written in Python by [Scott Conner](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor).
/// Explanation by them:
/// Characters U+0300–U+036F are the combining characters for unicode Latin.
/// The fun thing about combining characters is that you can add as many of these characters
/// as you like to the original character and it does not create any new symbols,
/// it only adds symbols on top of the character. It's supposed to be used in order to
/// create characters such as á by taking a normal a and adding another character
/// to give it the mark (U+301, in this case). Fun fact, Unicode doesn't specify
/// any limit on the number of these characters.
/// Conveniently, this gives us 112 different characters we can map to,
/// which nicely maps to the ASCII character range 0x20 -> 0x7F, aka all the non-control characters.
/// The only issue is that we can't have new lines in this system, so to fix that,
/// we can simply map 0x7F (DEL) to 0x0A (LF). Since we want to avoid if elses (for golfing purposes),
/// this can be represented as (CHARACTER - 11) % 133 - 21, and decoded with (CHARACTER + 22) % 133 + 10.

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

pub fn zalgo_compress(string_to_compress: &str) -> Result<String, String> {
    let mut line = 1;
    let mut result: Vec<u8> = vec![b'E'];

    for c in string_to_compress.bytes() {
        if c == b'\r' {
            return Err("Non-unix line endings detected".into());
        }

        if c == b'\n' {
            line += 1;
        }

        if !(32..=126).contains(&c) && c != b'\n' {
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
        .map(|(odds, evens)| (((odds << 6 & 64 | evens & 63) + 22) % 133 + 10))
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
        const TEST_STRING_1: &str = "the greatest adventure is going to bed";
        let out_string = std::str::from_utf8(b"E\xcd\x94\xcd\x88\xcd\x85\xcc\x80\xcd\x87\xcd\x92\xcd\x85\xcd\x81\xcd\x94\xcd\x85\xcd\x93\xcd\x94\xcc\x80\xcd\x81\xcd\x84\xcd\x96\xcd\x85\xcd\x8e\xcd\x94\xcd\x95\xcd\x92\xcd\x85\xcc\x80\xcd\x89\xcd\x93\xcc\x80\xcd\x87\xcd\x8f\xcd\x89\xcd\x8e\xcd\x87\xcc\x80\xcd\x94\xcd\x8f\xcc\x80\xcd\x82\xcd\x85\xcd\x84").unwrap();
        assert_eq!(zalgo_compress(TEST_STRING_1).unwrap(), out_string);

        const TEST_STRING_2: &str =
            "I'll have you know I graduated top of my class in the Navy Seals";
        assert_eq!(
            zalgo_decompress(&zalgo_compress(TEST_STRING_2).unwrap()).unwrap(),
            TEST_STRING_2
        );
    }

    #[test]
    fn newlines() {
        assert_eq!(&zalgo_compress("\n").unwrap(), "Eͯ",);
        const TEST_STRING: &str = "The next sentence is true.\nThe previous sentence is false.";
        assert_eq!(
            zalgo_decompress(&zalgo_compress(TEST_STRING).unwrap()).unwrap(),
            TEST_STRING,
        );
    }

    #[test]
    fn check_errors() {
        assert!(zalgo_compress("We got the Ä Ö Å, you aint got the Ä Ö Å").is_err());
        assert!(zalgo_compress("\t").is_err());
    }
}
