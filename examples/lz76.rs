//! Explore exhaustive LZ76 phrase counts on bytes.
use celandine::complexity::lz76_complexity;

fn main() {
    println!("LZ76 c_76(x) counts exhaustive production phrases in byte sequence x.");
    println!("Copy the longest match starting earlier, then add one byte if any remains.");
    println!("Overlapping copies are allowed; a reproducible final suffix counts once.");
    println!("For ABABAB: A | B | ABAB => 3 phrases. Units are phrases, not bits.");
    for (data, parse) in [
        (&b""[..], "(empty)"),
        (b"A", "A"),
        (b"AAAA", "A | AAA"),
        (b"ABABAB", "A | B | ABAB"),
        (b"ABCD", "A | B | C | D"),
        (b"AAB", "A | AB"),
        (b"BAA", "B | A | A"),
        (b"101101011", "1 | 0 | 11 | 010 | 11"),
        (b"010011101101100", "0 | 1 | 00 | 11 | 101 | 101100"),
    ] {
        println!(
            "{:16} c_76={}: {parse}",
            String::from_utf8_lossy(data),
            lz76_complexity(data)
        );
    }
    println!(
        "Binary [0,255,0,255]: {} phrases",
        lz76_complexity(&[0, 255, 0, 255])
    );
    println!("AAB and BAA share byte counts but differ in LZ76; reversal need not preserve it.");
    println!("Parses above are worked examples, not output from a public parser API.");
    println!("No normalization, compressed size, or unknown-source entropy rate is computed.");
}
