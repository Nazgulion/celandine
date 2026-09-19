//! Explain collision entropy and compare measures using one histogram per sample.

use celandine::distribution::Distribution;
use celandine::entropy::{
    collision_entropy_distribution, hartley_entropy_distribution, shannon_distribution,
};

fn main() {
    // Edit these samples and run `cargo run --example collision`.
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
    println!("Collision entropy: order two of Rényi's entropy family (1961).");
    println!("C = sum(p*p), with p = count / sample length; H2 = -log2(C).");
    println!("C is the probability of equal symbols in two independent draws with replacement.");
    println!("Useful for measuring frequency concentration; results ignore sequence order.");
    println!("H2 is in bits/symbol; C is a dimensionless probability.");
    for data in samples {
        let d = Distribution::from_bytes(data);
        if d.is_empty() {
            println!(
                "{:?}: C=undefined, H2={:.6} (empty convention)",
                String::from_utf8_lossy(data),
                collision_entropy_distribution(&d)
            );
            continue;
        }
        // Direct squares illustrate C for these small samples, not the core algorithm.
        let matching = d.probabilities().map(|p| p * p).sum::<f64>();
        println!(
            "{:?}: C={matching:.6}, H2={:.6}, Shannon={:.6}, Hartley={:.6}",
            String::from_utf8_lossy(data),
            collision_entropy_distribution(&d),
            shannon_distribution(&d),
            hartley_entropy_distribution(&d)
        );
    }
    println!("AAAB: C=(3/4)^2+(1/4)^2=10/16, so H2=log2(8/5)=0.678072.");
    println!("AB: two of four ordered position pairs match, giving C=1/2 and H2=1.");
}
