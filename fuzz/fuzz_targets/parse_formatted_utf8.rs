#![no_main]

use std::str;

use custos_cpe::Cpe;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = str::from_utf8(data) {
        let _ = Cpe::parse_formatted(input);
    }
});
