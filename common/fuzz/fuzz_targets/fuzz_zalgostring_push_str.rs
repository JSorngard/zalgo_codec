#![no_main]

use libfuzzer_sys::fuzz_target;
use zalgo_codec_common::ZalgoString;

fuzz_target!(|data: &str| {
    let mut zs1 = ZalgoString::new("Zalgo").unwrap();
    if let Ok(zs2) = ZalgoString::new(data) {
        zs1.push_zalgo_str(&zs2);
        assert_eq!(zs1.into_decoded_string(), format!("Zalgo{data}"));
    }
});