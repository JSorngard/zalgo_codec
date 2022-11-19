//! This crate provides the macro part of [`zalgo-codec`](https://docs.rs/zalgo-codec/latest/zalgo_codec/)
//! by defining the procedural macro `zalgo_embed!`.

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
/// This macro decodes a Unicode string that has been encoded with `zalgo_encode`
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
/// # Limitations
/// Due to ambiguity macros can not deal with variable names inside format string literals.
/// An example of this is that `println!("{variable_name}")` will give a compile error if used in a macro,
/// but `println!("{}", variable_name)` will work fine. This means that calling `zalgo_embed!` on the encoded 
/// version of the former will not work.  
/// 
/// There are many more limitations, and as I learn about more I will add them here. Feel free to create a 
/// Pull Request on Github for adding more notes here it you know of more limitations.
pub fn zalgo_embed(encoded: TokenStream) -> TokenStream {
    let encoded = parse_macro_input!(encoded as LitStr).value();

    zalgo_codec_common::zalgo_decode(&encoded)
        .unwrap()
        .parse()
        .unwrap()
}
