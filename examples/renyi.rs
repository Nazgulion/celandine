//! Compare Rényi orders on one reusable empirical distribution per sample.

use celandine::distribution::Distribution;
use celandine::entropy::{InvalidRenyiOrder, renyi_entropy_distribution};

fn main() -> Result<(), InvalidRenyiOrder> {
    // Edit samples and orders, then run `cargo run --example renyi`.
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
    let orders = [0.0, 0.5, 1.0, 2.0, f64::INFINITY];
    println!("Rényi (1961): entropy with an order parameter controlling frequency emphasis.");
    println!("H_alpha = log2(sum(p^alpha)) / (1-alpha), with p = count / sample length.");
    println!("Below one, rare symbols matter more; above one, common symbols matter more.");
    println!("Order 0 = Hartley; 1 = Shannon; infinity = -log2(largest probability).");
    println!("All results are bits/symbol; symbol order is ignored.");
    for data in samples {
        let d = Distribution::from_bytes(data);
        print!(
            "{:?}: support={}",
            String::from_utf8_lossy(data),
            d.support_size()
        );
        for alpha in orders {
            print!(", H({alpha})={:.6}", renyi_entropy_distribution(&d, alpha)?);
        }
        println!();
    }
    println!("AAAB: p=(3/4,1/4), so H(2)=-log2(10/16)=0.678072 bits/symbol.");
    println!("Uniform samples have equal entropy at every order; uneven samples decrease.");
    println!("Empty input returns zero by convention; it has no empirical probability law.");
    Ok(())
}
