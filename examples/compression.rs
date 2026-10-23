//! Run with: cargo run --locked --features compression-deflate --example compression
use celandine::compression::{Zlib, compressed_length, compression_ratio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Compression measures a particular encoder's complete output, including overhead.");
    println!("C(x) is zlib bytes; R(x) = C(x) / |x| is compressed bytes per input byte.");
    println!("Fixed miniz_oxide 0.9.1, level 6, fresh 32 KiB window, no dictionary.");
    for input in [b"".as_slice(), b"A", b"ABABAB", &[b'A'; 1024]] {
        let length = compressed_length(input, &Zlib)?;
        match compression_ratio(input, &Zlib) {
            Ok(ratio) => println!(
                "{} input bytes -> {length} zlib bytes, ratio {ratio:.6}",
                input.len()
            ),
            Err(error) => println!(
                "{} input bytes -> {length} zlib bytes; {error}",
                input.len()
            ),
        }
    }
    assert_eq!(compressed_length(b"A", &Zlib)?, 9);
    assert_eq!(compression_ratio(b"A", &Zlib)?, 9.0);
    println!("A: 2 header + 3 DEFLATE + 4 checksum bytes = 9; 9 / 1 = 9.");
    println!(
        "Short inputs may expand. These are compressor-dependent proxies, not Kolmogorov complexity."
    );
    Ok(())
}
