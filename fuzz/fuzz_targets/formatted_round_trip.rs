#![no_main]

use std::str;

use custos_cpe::Cpe;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = str::from_utf8(data) else {
        return;
    };
    let Ok(parsed) = Cpe::parse_formatted(input) else {
        return;
    };

    let formatted = parsed.to_string();
    let reparsed = Cpe::parse_formatted(&formatted)
        .expect("Display must produce a valid CPE 2.3 formatted string");

    assert_eq!(parsed, reparsed);
});
