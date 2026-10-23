//! Explore exact counts of distinct contiguous blocks at several byte lengths.
use celandine::complexity::{subword_complexity, subword_complexity_profile};

fn main() {
    println!("Subword complexity p_x(n) counts distinct contiguous n-byte blocks in x.");
    println!("Repeated occurrences count once; units are blocks, not bits.");
    println!("For ABABA at n=2: AB, BA, AB, BA -> {{AB, BA}} -> p(2)=2.");
    println!("Profiles below cover n=0..=L; p(0)=1 counts the empty word.");
    for (name, data) in [
        ("empty", &b""[..]),
        ("constant", b"AAAA"),
        ("alternating", b"ABABA"),
        ("grouped", b"AABB"),
        ("interleaved", b"ABAB"),
        ("distinct", b"ABCD"),
        ("binary", &[0, 255, 0, 255]),
    ] {
        let profile: Vec<_> = subword_complexity_profile(data).collect();
        println!(
            "{name:12} p(2)={}: {profile:?}",
            subword_complexity(data, 2)
        );
    }
    println!(
        "AABB and ABAB share byte counts but have different p(2). Profiles need not increase."
    );
    println!("A bounded prefix computes only requested lengths:");
    for (n, count) in subword_complexity_profile(b"ABABA").take(3).enumerate() {
        println!("  n={n} bytes: {count} distinct blocks");
    }
    println!("Oversized length: p(6)={}", subword_complexity(b"ABABA", 6));
    println!("This describes the sample, not an unknown source or its algorithmic complexity.");
}
