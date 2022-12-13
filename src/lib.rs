//! This is a crate implementing the zalgo encoding and decoding functions
//! [originally written in Python](https://github.com/DaCoolOne/DumbIdeas/tree/main/reddit_ph_compressor) by Scott Conner
//! and extends them for Rust by providing a procedural macro that can run encoded source code.
//!
//! With the functions defined in this crate you can transform an ASCII string into a unicode string that is a single
//! "character" wide in a reversible way. The encoded string will be larger than the original in terms of bytes.
//!
//! The crate also provides the [`zalgo_embed!`] macro that can be used to embed encoded source code and
//! pass on the decoded code to the compiler. Imagine the code clarity!
//!
//! Additionally the crate provides functions to encode Python code and wrap the result in a decoder that
//! decodes and executes the encoded string.
//!
//! Can not encode carriage returns, so files written on non-unix operating systems might not work. The file encoding
//! functions will attempt to encode files anyway by ignoring carriage returns, but the string encoding functions will return an error.
//!
//! # Example
//! The cursed character is the result of using [`zalgo_encode`] on the text `fn add(x: i32, y: i32) -> i32 {x + y}`.
//! ```
//! # use zalgo_codec::zalgo_embed;
//! // We can add that text to our code with the macro
//! zalgo_embed!("E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");
//!
//! // The `add` function is now available
//! assert_eq!(add(10, 20), 30);
//! ```
//!   
//! # Explanation
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
//!
//! # Notes
//! The [original post](https://www.reddit.com/r/ProgrammerHumor/comments/yqof9f/the_most_upvoted_comment_picks_the_next_line_of/ivrd9ur/?context=3)
//! where the Python code was first presented together with the above explanation.

#![forbid(unsafe_code)]

pub use zalgo_codec_common::*;
pub use zalgo_codec_macro::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, str};

    #[test]
    fn test_embed_function() {
        let code = "fn add(x: i32, y: i32) -> i32 {x + y}";

        let encoded = zalgo_encode(code).unwrap();
        assert_eq!(encoded, "E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");

        zalgo_embed!("E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");

        // Now the `add` function is available
        assert_eq!(add(10, 20), 30)
    }

    #[test]
    fn test_embed_expression() {
        let x = 20;
        let y = -10;

        let expr = "x + y";

        let encoded = zalgo_encode(expr).unwrap();
        println!("{}", encoded);
        assert_eq!(encoded, "È͙̋̀͘");

        // It works on expressions, too!
        let z = zalgo_embed!("È͙̋̀͘");
        assert_eq!(z, x + y);
    }

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
            lorem_text.replace('\r', "")
        );

        let mut consistency_path = PathBuf::new();
        consistency_path.push("tests");
        consistency_path.push("consistency_check.txt");

        decode_file(&zalgo_path, &consistency_path).unwrap();
        assert!(decode_file(&zalgo_path, &consistency_path).is_err());

        let decoded_text = fs::read_to_string(&consistency_path).unwrap();

        assert_eq!(decoded_text, lorem_text.replace('\r', ""));
        fs::remove_file(zalgo_path).unwrap();
        fs::remove_file(consistency_path).unwrap();
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
