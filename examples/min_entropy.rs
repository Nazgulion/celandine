//! Explain min-entropy and compare measures using one histogram per sample.

use celandine::distribution::Distribution;
use celandine::entropy::{
    collision_entropy_distribution, hartley_entropy_distribution, min_entropy_distribution,
    shannon_distribution,
};

fn main() {
    // Edit these samples and run `cargo run --example min_entropy`.
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
        b"AAAAAABBBC",
        b"AAAAAABBCC",
    ];
    println!("Min-entropy: the infinite-order limit of Rényi's entropy family (1961).");
    println!("H_inf = -log2(p_max), where p_max = largest count / sample length.");
    println!("It measures the surprise of the most likely symbol, exposing dominance.");
    println!("p_max is the best fixed-symbol guess probability under the empirical law.");
    println!("Entropies are in bits/symbol; probabilities have no units; order is ignored.");
    for data in samples {
        let d = Distribution::from_bytes(data);
        if d.is_empty() {
            println!(
                "{:?}: p_max=undefined, H_inf={:.6} (empty convention)",
                String::from_utf8_lossy(data),
                min_entropy_distribution(&d)
            );
            continue;
        }
        let maximum = d.probabilities().fold(0.0, f64::max);
        println!(
            "{:?}: p_max={maximum:.6}, H_inf={:.6}, H2={:.6}, Shannon={:.6}, Hartley={:.6}",
            String::from_utf8_lossy(data),
            min_entropy_distribution(&d),
            collision_entropy_distribution(&d),
            shannon_distribution(&d),
            hartley_entropy_distribution(&d)
        );
    }
    println!("AAAB: p_max=3/4, so H_inf=log2(4/3)=0.415037; the best fixed guess is A.");
    println!("The last two samples share p_max=0.6 and H_inf, but differ in H2 and Shannon.");
}
