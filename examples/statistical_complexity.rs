//! Frequency-based statistical complexity with an explicit state space.
use celandine::{
    complexity::{
        normalized_jensen_shannon_complexity as complexity,
        normalized_jensen_shannon_complexity_distribution as reuse,
    },
    distribution::Distribution,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Normalized Jensen-Shannon statistical complexity: C = (H/log2 K)*(J/Jmax).");
    println!("H is byte entropy in bits/symbol; J compares to uniform on K states, in bits.");
    println!(
        "Jmax is the point-mass divergence; C is dimensionless. K includes unobserved states."
    );
    for data in [b"AAAA".as_slice(), b"AABB", b"AAAB", b"ABAB"] {
        let p = Distribution::from_bytes(data);
        for k in [2, 4, 256] {
            let c = reuse(&p, k)?;
            assert_eq!(c, complexity(data, k)?);
            println!("{}: K={k:3}, C={c:.9}", String::from_utf8_lossy(data));
        }
    }
    assert!((complexity(b"AAAB", 2)? - 0.127173305349489).abs() < 1e-14);
    assert_eq!(complexity(b"AABB", 2)?, 0.0);
    println!("AABB and ABAB agree: frequency-based complexity does not detect temporal structure.");
    println!("Empty input, K outside 2..=256, or support exceeding K return explicit errors.");
    Ok(())
}
