#![no_main]

use ::dice::parse;
use ::libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    let _ = parse(data, true);
});
