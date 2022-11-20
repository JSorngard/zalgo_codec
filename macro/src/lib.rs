//! This crate provides the macro part of [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/)
//! by defining the procedural macro [`zalgo_embed!`].

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

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
/// use zalgo_codec_common::{zalgo_encode, UnencodableCharacterError};
/// use zalgo_codec_macro::zalgo_embed;
/// 
/// fn main() -> Result<(), UnencodableCharacterError> {
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
/// This is an incomplete list of the limitations of this macro. There are many more,
/// and as I learn about them I will add them here. Feel free to create a 
/// Pull Request on [Github](https://github.com/JSorngard/zalgo_codec) for adding more notes here if you 
/// know of more limitations.
/// 
/// - Due to ambiguity macros can not deal with variable names inside format string literals. An example of this is that  
///   `println!("{variable_name}")`  
///    will give a compile error if used in a macro,
///    but  
///    `println!("{}", variable_name)`  
///    will work fine. This means that calling `zalgo_embed!` on the encoded 
///    version of the former will not work.  
#[proc_macro]
pub fn zalgo_embed(encoded: TokenStream) -> TokenStream {
    let encoded = parse_macro_input!(encoded as LitStr).value();

    zalgo_codec_common::zalgo_decode(&encoded)
        .unwrap()
        .parse()
        .unwrap()
}
