#![no_main]
libfuzzer_sys::fuzz_target!(|input: &[u8]| celandine_fuzz::blocks(input));
