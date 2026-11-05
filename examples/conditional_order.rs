//! Uncertainty of the next byte given a preceding context.
use celandine::{
    distribution::ngram_probabilities,
    entropy::{conditional_entropy_order, conditional_entropy_order_distribution},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Conditional entropy: uncertainty of a byte given its preceding k bytes.");
    println!("H = -sum [N(c,a)/m]*log2[N(c,a)/N(c)], in bits per predicted byte.");
    println!("N counts context c followed by byte a; m counts complete observations.");
    for data in [b"ABABAB".as_slice(), b"AAB", b"AABA", b"AAAA", b""] {
        for order in 0..=3 {
            let d = ngram_probabilities(data, order + 1)?;
            let h = conditional_entropy_order(data, order);
            assert!((h - conditional_entropy_order_distribution(&d)).abs() < 1e-12);
            println!(
                "{data:?}, order={order}: {} observations, H={h:.6} bits/predicted byte",
                d.sample_size()
            );
        }
    }
    assert_eq!(conditional_entropy_order(b"ABABAB", 1), 0.0);
    assert_eq!(conditional_entropy_order(b"AAB", 1), 1.0);
    println!("ABABAB: each context predicts one next byte -> 0 bits at order one.");
    println!("AAB: A predicts A/B equally -> 1 bit; AABA: weighted contexts give 2/3 bit.");
    println!(
        "Empty or insufficient observations return zero by convention, without a source-rate claim."
    );
    Ok(())
}
