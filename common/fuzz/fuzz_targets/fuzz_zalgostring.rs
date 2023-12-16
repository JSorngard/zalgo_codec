#![no_main]

use libfuzzer_sys::fuzz_target;
use zalgo_codec_common::ZalgoString;

fuzz_target!(|data: &str| {
    if let Ok(zs) = ZalgoString::new(data) {
        assert_eq!(zs.into_decoded_string(), data);
    }
});
