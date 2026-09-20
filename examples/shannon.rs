//! Run with `cargo run --example shannon` to explore empirical byte entropy.

use celandine::distribution::Distribution;
use celandine::entropy::{InvalidLogBase, shannon_distribution_with_base};

fn main() -> Result<(), InvalidLogBase> {
    // Edit these samples and rerun to inspect different byte sequences.
    let samples: &[&[u8]] = &[
        b"",
        b"A",
        b"AAAA",
        b"AB",
        b"ABAB",
        b"AAAB",
        b"ABCD",
        b"ABCABCABC",
        b"0123456789",
    ];

    println!("Shannon (1948): average symbol surprise, weighted by observed frequencies.");
    println!("H_b = -sum(p * log_b(p)) = H_2/log2(b); p is count / sample length.");
    println!("b is the logarithm base: 2 gives bits, e gives nats, 10 gives decimal units.");
    println!("Every result is per symbol. A base must be finite and greater than one.");
    println!("Useful for comparing frequency balance; this measure ignores symbol order.");
    for data in samples {
        let d = Distribution::from_bytes(data);
        println!(
            "{:?}: {:.6} bits/symbol | {:.6} nats/symbol | {:.6} decimal units/symbol",
            String::from_utf8_lossy(data),
            shannon_distribution_with_base(&d, 2.0)?,
            shannon_distribution_with_base(&d, std::f64::consts::E)?,
            shannon_distribution_with_base(&d, 10.0)?,
        );
    }
    println!("ABCD: four probabilities of 1/4, each contributing 0.5 bits, total 2 bits/symbol.");
    println!(
        "Those 2 bits/symbol equal approximately 1.386294 nats/symbol or 0.602060 decimal units/symbol."
    );
    println!(
        "Changing bases changes units, not probabilities. Empty input returns zero after base validation."
    );
    Ok(())
}
