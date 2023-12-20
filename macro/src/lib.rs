//! This crate provides the proc-macro part of the crate [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/)
//! by defining the procedural macro [`zalgo_embed!`].
//!
//! It lets you take source code that's been converted into a single grapheme cluster by the
//! [`zalgo-codec-common`](https://docs.rs/zalgo-codec-common/latest/zalgo_codec_common/) crate
//! and compile it as if it was never zalgo-ified.
//!
//! # Example
//!
//! If we run [`zalgo_encode`] on the text
//! `fn add(x: i32, y: i32) -> i32 {x + y}` we can add the `add` function to our program
//! by putting the resulting grapheme cluster inside [`zalgo_embed!`]:
//! ```
//! # use zalgo_codec_macro::zalgo_embed;
//! zalgo_embed!("E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");
//! assert_eq!(add(10, 20), 30);
//! ```

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::format;
use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned, LitStr};

use zalgo_codec_common::{zalgo_decode, zalgo_encode};

/// This macro decodes a string that has been encoded with [`zalgo_encode`](https://docs.rs/zalgo-codec-common/latest/zalgo_codec_common/fn.zalgo_encode.html)
/// and passes the results on to the compiler.
///
/// To generate the encoded string used as input you can use the provided program. It can be installed with `cargo install zalgo-codec --features binary`.
/// # Examples
/// We can use a function created in encoded source code:
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// # use zalgo_codec_macro::zalgo_embed;
/// // This line expands to the code
/// // `fn add(x: i32, y: i32) -> i32 {x + y}`
/// zalgo_embed!("E͎͉͙͉̞͉͙͆̀́̈́̈́̈̀̓̒̌̀̀̓̒̉̀̍̀̓̒̀͛̀̋̀͘̚̚͘͝");
///
/// // Now the `add` function is available
/// assert_eq!(add(10, 20), 30);
/// ```
///
/// It works on expressions too!
/// ```
/// # use zalgo_codec_common::zalgo_encode;
/// # use zalgo_codec_macro::zalgo_embed;
/// let x = 20;
/// let y = -10;
///
/// // This macro is expanded to the code
/// // `x + y`
/// let z = zalgo_embed!("È͙̋̀͘");
/// assert_eq!(z, x + y);
/// ```
///
/// A more complex example is this program which expands to code that reads the
/// command line input, encodes it, and prints out the result.
/// ```
/// use zalgo_codec_common::{zalgo_encode, Error};
/// use zalgo_codec_macro::zalgo_embed;
///
/// fn main() -> Result<(), Error> {
///     // This macro expands to
///     // let input = std::env::args().collect::<Vec<_>>()[1..].join(" ");
///     // let output = zalgo_encode(&input)?;
///     // println!("{}", output);
///     zalgo_embed!("E͔͉͎͕͔̝͓͔͎͖͇͓͌̀͐̀̀̈́́͒̈̉̎̓̚̚̚̚ͅͅ͏̶͔̜̜̞̞̻͌͌̓̓̿̈̉̑̎̎̽̎͊̚̚ͅͅ͏̛͉͎͔̈̂̀̂̉ͯ͌̀ͅ͏͕͔͕͔̝͚͇͐̀̀́͌͏͎̿̓ͅ͏̛͉͎͕͔̟͉͎͔͎̼͎̼͎̈́̈̆͐̉ͯ͐͒͌́̈̂͛̂̌̀͝ͅ͏̛͕͔͕͔͐̉");
///     Ok(())
/// }
/// ```
/// Do the opposite of [`obfstr`](https://crates.io/crates/obfstr): obfuscate a string while coding and deobfuscate it during compile time
/// ```
/// # use zalgo_codec_macro::zalgo_embed;
/// let secret_string = zalgo_embed!("Ê̤͏͎͔͔͈͉͓͍̇̀͒́̈́̀̀ͅ͏͍́̂");
/// assert_eq!(secret_string, "Don't read this mom!");
/// ```
/// # Limitations
/// The encoded code must be valid macro output.
#[proc_macro]
pub fn zalgo_embed(encoded: TokenStream) -> TokenStream {
    let encoded = parse_macro_input!(encoded as LitStr).value();

    match zalgo_decode(&encoded) {
        Ok(decoded) => match decoded.parse() {
            Ok(token_stream) => token_stream,
            Err(e) => syn::Error::new(encoded.span(), e).to_compile_error().into(),
        },
        Err(e) => syn::Error::new(
            encoded.span(),
            format!("the given string decodes into an {e}"),
        )
        .to_compile_error()
        .into(),
    }
}

/// At compile time this proc-macro converts the given string literal into a string that contains a single grapheme cluster.
///
/// # Example
///
/// Basic usage:
/// ```
/// # use zalgo_codec_macro::zalgofy;
/// const ZS: &str = zalgofy!("Zalgo");
/// assert_eq!(ZS, "É̺͇͌͏");
/// ```
///
/// # Errors
///
/// Gives a compile error if any character in the string is not either a printable ACII or newline character.
///
/// ```compile_fail
/// # use zalgo_codec_macro::zalgofy;
/// // compile error: "line 2 at column 3: byte value 195 does not correspond to an ASCII character"
/// let zs = zalgofy!("a\naeö");
/// ```
#[proc_macro]
pub fn zalgofy(string: TokenStream) -> TokenStream {
    let string = parse_macro_input!(string as LitStr).value();
    match zalgo_encode(&string) {
        Ok(encoded) => {
            let string = format!("\"{encoded}\"");
            match string.parse() {
                Ok(token_stream) => token_stream,
                Err(e) => syn::Error::new(string.span(), e)
                    .into_compile_error()
                    .into(),
            }
        }
        Err(e) => syn::Error::new(string.span(), e).to_compile_error().into(),
    }
}
