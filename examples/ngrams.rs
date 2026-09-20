//! Overlapping byte blocks, exact counts, and empirical probabilities.

use celandine::distribution::{NgramDistribution, ngram_counts};
use celandine::transforms::{InvalidNgramLength, ngrams};

fn main() -> Result<(), InvalidNgramLength> {
    println!("N-grams are complete n-byte blocks, starting one byte apart.");
    println!("L = input bytes; n = block bytes; m = L - n + 1 occurrences if n <= L, else 0.");
    println!("c(g) counts occurrences of block g; p(g) = c(g)/m is dimensionless.");
    println!("Blocks are printed as byte values; counts are sorted by byte contents.\n");
    for (data, n) in [
        (&b"ABABA"[..], 2),
        (&b"AAAA"[..], 2),
        (&b"AABB"[..], 2),
        (&b"ABAB"[..], 2),
        (&b"ABABA"[..], 1),
        (&b"ABABA"[..], 5),
        (&[0, 255, 0, 255][..], 2),
        (&b"AB"[..], 3),
        (&b""[..], 1),
    ] {
        let blocks: Vec<_> = ngrams(data, n)?.collect();
        println!("data={data:?}, L={}, n={n}; blocks={blocks:?}", data.len());
        let d = NgramDistribution::from_counts(ngram_counts(data, n)?);
        println!(
            "m={}, distinct blocks={}",
            d.sample_size(),
            d.support_size()
        );
        if d.is_empty() {
            println!("No complete blocks: empty empirical state, no probability law.");
        }
        for (block, probability) in d.probabilities() {
            println!(
                "  {block:?}: count={}, p={probability:.6}",
                d.counts().count(block)
            );
        }
        println!();
    }
    println!("Length zero is rejected: {}", ngrams(b"AB", 0).unwrap_err());
    println!("AABB and ABAB have identical byte counts but different two-byte counts.");
    println!("These probabilities describe observed blocks, not a source's entropy rate.");
    Ok(())
}
