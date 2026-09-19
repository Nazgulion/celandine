//! Compare empirical Shannon and Hartley entropy with one histogram per sample.

use celandine::distribution::Distribution;
use celandine::entropy::{hartley_entropy_distribution, shannon_distribution};

fn main() {
    // Edit the samples and run `cargo run --example hartley` to explore.
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

    println!("Hartley (1928): logarithmic counting of distinguishable possibilities.");
    println!("Here H0 = log2(k), where k is the number of observed distinct bytes.");
    println!("Useful as the largest Shannon entropy possible on that support.");
    println!("Shannon (1948): H = -sum(p * log2(p)), with p = count / sample length.");
    println!("Both results are in bits/symbol and ignore symbol order.");
    for data in samples {
        let d = Distribution::from_bytes(data);
        println!(
            "{:?}: support={}, Shannon={:.6}, Hartley={:.6}",
            String::from_utf8_lossy(data),
            d.support_size(),
            shannon_distribution(&d),
            hartley_entropy_distribution(&d),
        );
    }
    println!(
        "AB and AAAB have the same Hartley entropy; unequal frequencies lower Shannon entropy."
    );
    println!("Empty input returns zero by convention; no empirical probability law exists there.");
}
