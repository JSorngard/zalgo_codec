//! This crate provides the macro part of [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/)
//! by defining the procedural macro [`zalgo_embed!`].

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

use zalgo_codec_common::zalgo_decode;

/// This macro decodes a Unicode string that has been encoded with [`zalgo_encode`](https://docs.rs/zalgo-codec-common/latest/zalgo_codec_common/fn.zalgo_encode.html)
/// and passes the results on to the compiler.
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
/// use zalgo_codec_common::{zalgo_encode, UnencodableByteError};
/// use zalgo_codec_macro::zalgo_embed;
///
/// fn main() -> Result<(), UnencodableByteError> {
///     // This macro expands to
///     // let input = std::env::args().collect::<Vec<_>>()[1..].join(" ");
///     // let output = zalgo_encode(&input)?;
///     // println!("{}", output);
///     zalgo_embed!("E͔͉͎͕͔̝͓͔͎͖͇͓͌̀͐̀̀̈́́͒̈̉̎̓̚̚̚̚ͅͅ͏̶͔̜̜̞̞̻͌͌̓̓̿̈̉̑̎̎̽̎͊̚̚ͅͅ͏̛͉͎͔̈̂̀̂̉ͯ͌̀ͅ͏͕͔͕͔̝͚͇͐̀̀́͌͏͎̿̓ͅ͏̛͉͎͕͔̟͉͎͔͎̼͎̼͎̈́̈̆͐̉ͯ͐͒͌́̈̂͛̂̌̀͝ͅ͏̛͕͔͕͔͐̉");
///     Ok(())
/// }
/// ```
///
/// # Limitations
/// The encoded code must be valid macro output
#[proc_macro]
pub fn zalgo_embed(encoded: TokenStream) -> TokenStream {
    let encoded = parse_macro_input!(encoded as LitStr).value();

    zalgo_decode(&encoded).unwrap().parse().unwrap()
}
