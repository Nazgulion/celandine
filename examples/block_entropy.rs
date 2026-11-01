//! Overlapping-block Shannon entropy, in bits per block.
use celandine::distribution::ngram_probabilities;
use celandine::entropy::{block_entropy, block_entropy_distribution};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Block entropy: uncertainty across overlapping n-byte blocks.");
    println!("H_n = -sum p_g*log2(p_g), p_g = block count / number of observations.");
    println!("Units: bits per block; no division by n, padding or wrapping.");
    for data in [b"ABABA".as_slice(), b"AABBA", b"AAAAA", b""] {
        for n in [1, 2, 5, 6] {
            let d = ngram_probabilities(data, n)?;
            let h = block_entropy(data, n)?;
            assert!((h - block_entropy_distribution(&d)).abs() < 1e-12);
            println!(
                "{data:?}, n={n}: {} observations, {} distinct blocks, H={h:.6} bits/block",
                d.sample_size(),
                d.support_size()
            );
        }
    }
    assert_eq!(block_entropy(b"ABABA", 2)?, 1.0);
    assert_eq!(block_entropy(b"AABBA", 2)?, 2.0);
    assert!(block_entropy(b"", 0).is_err());
    println!("ABABA pairs: AB=2, BA=2 -> 1 bit/block; AABBA has four distinct pairs -> 2.");
    println!(
        "No observations returns zero by convention; this does not estimate a source entropy rate."
    );
    Ok(())
}
