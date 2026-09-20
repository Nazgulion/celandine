#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| celandine_fuzz::byte_entropy(data));
