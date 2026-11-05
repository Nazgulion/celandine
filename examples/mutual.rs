//! Shared information in paired byte observations.
use celandine::{
    distribution::joint_probabilities,
    information::{mutual_information, mutual_information_distribution},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mutual information: I(X;Y)=sum p(a,b)*log2(p(a,b)/(p_x(a)*p_y(b))).");
    println!("p is the empirical joint law; p_x and p_y are its marginals. Units: bits/pair.");
    for (x, y, expected) in [
        (b"AABB".as_slice(), b"XYXY".as_slice(), 0.0),
        (b"AABB", b"XXYY", 1.0),
        (b"ABCD", b"WXYZ", 2.0),
        (b"", b"", 0.0),
    ] {
        let d = joint_probabilities(x, y)?;
        let value = mutual_information_distribution(&d);
        assert!((value - expected).abs() < 1e-12);
        assert_eq!(value.to_bits(), mutual_information(x, y)?.to_bits());
        println!("{x:?} / {y:?}: {value:.6} bits/pair");
    }
    assert!(mutual_information(b"AB", b"X").is_err());
    println!(
        "Zero describes empirical independence; positive information does not establish causation."
    );
    Ok(())
}
