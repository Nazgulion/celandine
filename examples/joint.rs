//! Joint uncertainty of aligned byte observations and reusable marginal laws.
use celandine::{
    distribution::joint_probabilities,
    entropy::{joint_entropy, joint_entropy_distribution, shannon_distribution},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Joint entropy: H(X,Y) = -sum p(a,b)*log2(p(a,b)), in bits per aligned pair.");
    println!("p(a,b) is the pair count / number of positions; marginals sum over one coordinate.");
    for (x, y, expected) in [
        (b"AABB".as_slice(), b"XYXY".as_slice(), 2.0),
        (b"AABB", b"XXYY", 1.0),
        (b"", b"", 0.0),
    ] {
        let d = joint_probabilities(x, y)?;
        println!(
            "{x:?} / {y:?}: {:?}",
            d.counts().counts().collect::<Vec<_>>()
        );
        let h = joint_entropy_distribution(&d);
        assert_eq!(h, expected);
        assert_eq!(joint_entropy(x, y)?, h);
        println!(
            "  H(X,Y)={h:.6} bits/pair; H(X)={:.6}, H(Y)={:.6} bits/coordinate",
            shannon_distribution(&d.marginal_x()),
            shannon_distribution(&d.marginal_y())
        );
    }
    assert!(joint_entropy(b"AB", b"X").is_err());
    println!("Equal marginals can hide different pairings; unequal lengths are rejected.");
    Ok(())
}
