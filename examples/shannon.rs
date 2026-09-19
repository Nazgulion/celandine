//! Run with `cargo run --example shannon` to explore empirical byte entropy.

use celandine::entropy::shannon;

fn main() {
    // Edit these samples and rerun to inspect different byte sequences.
    let samples: &[&[u8]] = &[
        b"",
        b"A",
        b"AAAA",
        b"AB",
        b"ABAB",
        b"ABCD",
        b"ABCABCABC",
        b"0123456789",
    ];

    println!("Shannon (1948): average symbol surprise, weighted by observed frequencies.");
    println!("H = -sum(p * log2(p)); p is count / sample length. Units: bits/symbol.");
    println!("Useful for comparing frequency balance; this measure ignores symbol order.");
    for data in samples {
        println!(
            "{:?}: {:.6} bits/symbol",
            String::from_utf8_lossy(data),
            shannon(data),
        );
    }
    println!("ABCD: four probabilities of 1/4, each contributing 0.5 bits, total 2 bits/symbol.");
    println!("Empty input returns zero by project convention.");
}
