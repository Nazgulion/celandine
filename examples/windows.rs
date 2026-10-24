//! Compose existing metrics with borrowed local windows.
use celandine::{entropy::shannon, transforms::windows};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Sliding windows select complete byte slices; size is width, step is start-to-start distance."
    );
    println!("Count: 0 when size > L, otherwise 1 + floor((L-size)/step), for L input bytes.");
    let data = b"ABCDEFGH";
    for block in windows(data, 3, 2)? {
        println!("size 3, step 2: {}", String::from_utf8_lossy(block));
    }
    println!("Starts are 0, 2, 4; incomplete GH at start 6 is omitted.");
    let data = b"AAAABABA";
    for (i, block) in windows(data, 4, 2)?.enumerate() {
        println!(
            "offset {}: {} -> {:.6} bits/symbol",
            i * 2,
            String::from_utf8_lossy(block),
            shannon(block)
        );
    }
    assert_eq!(
        windows(data, 4, 2)?.map(shannon).collect::<Vec<_>>(),
        vec![0.0, shannon(b"AABA"), 1.0]
    );
    println!("Zero width or step is an error; empty or undersized input yields no windows.");
    println!(
        "Extraction borrows without allocation. Metric costs are additional; overlapping windows share bytes."
    );
    Ok(())
}
