#![no_main]

use libfuzzer_sys::fuzz_target;
use zalgo_codec_common::ZalgoString;

fuzz_target!(|data: &str| {
    let mut zs = ZalgoString::new("Zalgo").unwrap();
    if zs.encode_and_push_str(data).is_ok() {
        assert_eq!(zs.into_decoded_string(), format!("Zalgo{data}"));
    }
});