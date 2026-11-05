//! Aligned conditional entropy: definitions and history in docs/entropy/conditional.md.
use celandine::{
    distribution::joint_probabilities,
    entropy::{conditional_entropy, conditional_entropy_distribution, conditional_entropy_order},
};
fn main() {
    println!("H(X|Y): uncertainty in the FIRST byte sequence after observing the SECOND.");
    println!(
        "Sum -(c(a,b)/N)*log2(c(a,b)/c_y(b)); c counts pairs, c_y counts Y, N counts observations."
    );
    println!("Units: bits per X observation. Groups are weighted by their frequency.");
    println!("ABCD / XXYY: each Y leaves two equally likely X values → 1 bit; reverse → 0.");
    for (x, y) in [
        (b"".as_slice(), b"".as_slice()),
        (b"A", b"X"),
        (b"ABCD", b"XXYY"),
        (b"AABB", b"XYXY"),
        (b"AAAABC", b"XXXXYY"),
        (b"ABAB", b"ABAB"),
    ] {
        let d = joint_probabilities(x, y).unwrap();
        println!(
            "X={:?}, Y={:?}: H(X|Y)={:.9}, H(Y|X)={:.9}",
            String::from_utf8_lossy(x),
            String::from_utf8_lossy(y),
            conditional_entropy_distribution(&d),
            conditional_entropy(y, x).unwrap()
        );
    }
    println!(
        "Length mismatch: {}",
        conditional_entropy(b"AB", b"X").unwrap_err()
    );
    println!(
        "Separate operation: next-byte H(order=1) for ABABAB = {} bits/predicted byte.",
        conditional_entropy_order(b"ABABAB", 1)
    );
    println!(
        "Empty/empty is zero by convention. Empirical predictability is not causation or a source guarantee."
    );
}
