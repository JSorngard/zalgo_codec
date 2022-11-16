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

        if !(32..=126).contains(&c) && c != b'\n' {
            return Err(format!("Non-ascii character byte {c} on line {line}"));
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
