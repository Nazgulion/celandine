//! Empirical overlapping-block entropy at several byte lengths.
use celandine::entropy::{block_entropy, block_entropy_profile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Block-entropy profiles: H_1 through H_min(max_order,input length).");
    println!("H_n = -sum p_n(g)*log2(p_n(g)), where p_n(g) is a block's occurrence fraction.");
    println!("Units: bits per n-byte block, without division by n.");
    for data in [b"AABB".as_slice(), b"ABABA", b"AAAA", b""] {
        let profile = block_entropy_profile(data, 4);
        println!("{data:?}, maximum 4: {} entries", profile.len());
        for (index, h) in profile.enumerate() {
            let n = index + 1;
            assert_eq!(h, block_entropy(data, n)?);
            println!("  n={n}: {h:.6} bits/block");
        }
    }
    let mut profile = block_entropy_profile(b"AABB", usize::MAX);
    assert_eq!(profile.next_back(), Some(0.0));
    assert_eq!(profile.next(), Some(1.0));
    assert_eq!(block_entropy_profile(b"AABB", 0).count(), 0);
    println!("AABB: [1, log2(3), 1, 0]; declining entries reflect finite sampling.");
    println!(
        "Adjacent differences are not generally conditional entropy or a source entropy rate."
    );
    Ok(())
}
