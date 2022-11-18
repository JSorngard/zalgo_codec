use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};



#[proc_macro]
pub fn zalgo_embed(encoded: TokenStream) -> TokenStream {
    let encoded = parse_macro_input!(encoded as LitStr).value();

    zalgo_codec_common::zalgo_decode(&encoded)
        .unwrap()
        .parse()
        .unwrap()
}