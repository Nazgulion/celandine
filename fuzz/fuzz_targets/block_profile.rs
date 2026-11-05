#![no_main]
libfuzzer_sys::fuzz_target!(|data: &[u8]| celandine_fuzz::block_profile(data));
