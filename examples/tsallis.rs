//! Explain Tsallis entropy, its fixed scale, and its order-one Shannon limit.

use celandine::distribution::Distribution;
use celandine::entropy::{shannon_distribution, tsallis_entropy_distribution};

fn main() {
    // Edit these samples and orders, then run `cargo run --example tsallis`.
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
    let orders = [0.0, 0.5, 1.0, 2.0, 4.0];
    println!("Tsallis (1988): a generalized entropy built from probability powers.");
    println!("S_q = (1 - sum(p^q)) / (q-1) / ln(2), where p = count / sample length.");
    println!("At q=1 use Shannon. Finite q>=0 only; the fixed scale is 1/ln(2).");
    println!("This is Shannon-bit scaling, not a general average code length or an 8-bit bound.");
    println!("Useful for comparing diversity at different orders; symbol order is ignored.");
    for data in samples {
        let d = Distribution::from_bytes(data);
        print!(
            "{:?}: support={}, Shannon={:.6}",
            String::from_utf8_lossy(data),
            d.support_size(),
            shannon_distribution(&d)
        );
        for q in orders {
            print!(
                ", S({q})={:.6}",
                tsallis_entropy_distribution(&d, q).unwrap()
            );
        }
        println!();
    }
    println!("AAAB: sum(p^2)=10/16, so S_2=(6/16)/ln(2)=0.541011.");
    println!("Uniform AB: S_0=1/ln(2)=1.442695, S_1=1, S_2=0.5/ln(2)=0.721348.");
    let d = Distribution::from_bytes(b"AAAB");
    println!("Near one for AAAB (keeping the supplied q):");
    for q in [1.0 - 1e-8, 1.0, 1.0 + 1e-8] {
        println!(
            "  q={q:.8}: {:.12}",
            tsallis_entropy_distribution(&d, q).unwrap()
        );
    }
    println!("Empty input has no empirical law; all valid orders return zero by convention.");
}
