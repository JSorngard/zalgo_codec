//! This crate provides the macro part of [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/)
//! by defining the procedural macro [`zalgo_embed!`].

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, spanned::Spanned, LitStr};

use zalgo_codec_common::zalgo_decode;

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
            format!("the given string decodes into an {}", e),
        )
        .to_compile_error()
        .into(),
    }
}
