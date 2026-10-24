//! Compare disjoint regions while retaining their incomplete suffix.
use celandine::{entropy::shannon, transforms::blocks};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fixed blocks are adjacent, complete byte slices; size counts bytes.");
    println!("For L bytes and size b: K = floor(L/b) blocks, R = L mod b tail bytes.");
    let iter = blocks(b"ABCDEFGH", 3)?;
    println!(
        "ABCDEFGH, size 3: K={}, R={}",
        iter.len(),
        iter.remainder().len()
    );
    println!(
        "Original tail: {}",
        String::from_utf8_lossy(iter.remainder())
    );
    for block in iter {
        println!("block: {}", String::from_utf8_lossy(block));
    }
    let iter = blocks(b"AAAABABAZ", 4)?;
    assert_eq!(iter.remainder(), b"Z");
    println!("Local Shannon entropy: H = -sum p*log2(p), with p the byte frequency fraction.");
    let mut entropies = Vec::new();
    for block in iter {
        let entropy = shannon(block);
        println!(
            "{}: {entropy:.6} bits/symbol",
            String::from_utf8_lossy(block)
        );
        entropies.push(entropy);
    }
    assert_eq!(entropies, [0.0, 1.0]);
    println!("Tail Z is excluded. This is entropy within each block, not a block-symbol entropy.");
    println!("Zero size is an error; oversized blocks leave the whole input as remainder.");
    Ok(())
}
