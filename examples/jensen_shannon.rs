//! Compare empirical byte laws through an equal mixture and its metric distance.
use celandine::distribution::Distribution;
use celandine::divergence::{
    jensen_shannon_distance_distribution, jensen_shannon_divergence_distribution,
};

fn main() {
    println!("Jensen-Shannon divergence compares byte composition through M=(P+Q)/2.");
    println!("J=H(M)-(H(P)+H(Q))/2 in bits; distance=sqrt(J) in square-root bits.");
    println!("Both range from 0 (equal laws) to 1 (disjoint supports), up to rounding.");
    println!("Each law has weight 1/2 regardless of sample length; ordering is ignored.");
    println!("A vs AB: M=(3/4,1/4), J=0.811278-0.5=0.311278, distance=0.557923.\n");
    for (x, y) in [
        (&b"A"[..], &b"AB"[..]),
        (b"AAAA", b"AB"),
        (b"AABB", b"ABAB"),
        (b"AAA", b"B"),
        (&[0, 255], &[255]),
        (b"", b"AB"),
    ] {
        let p = Distribution::from_bytes(x);
        let q = Distribution::from_bytes(y);
        match jensen_shannon_divergence_distribution(&p, &q) {
            Ok(j) => println!(
                "{x:?} vs {y:?}: J={j:.6}, distance={:.6}",
                jensen_shannon_distance_distribution(&p, &q).unwrap()
            ),
            Err(error) => println!("{x:?} vs {y:?}: {error}"),
        }
    }
}
