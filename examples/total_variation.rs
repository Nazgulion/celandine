//! Compare empirical byte composition, independently of sample length or order.

use celandine::distribution::Distribution;
use celandine::divergence::total_variation_distance_distribution;

fn main() {
    println!("Total variation compares byte composition, ignoring positions.");
    println!("TV(P,Q) = sum_i |p_i-q_i| / 2; p_i and q_i are byte relative frequencies.");
    println!("Dimensionless: 0 = equal laws, 1 = disjoint supports. Empty samples are errors.");
    println!("AAAB vs AB: (|3/4-1/2| + |1/4-1/2|)/2 = 0.25.\n");
    for (x, y) in [
        (&b"AAAB"[..], &b"AB"[..]),
        (b"AABB", b"ABAB"),
        (b"AAA", b"BB"),
        (&[0, 255, 255], &[0, 255]),
        (b"", b"AB"),
        (b"", b""),
    ] {
        let p = Distribution::from_bytes(x);
        let q = Distribution::from_bytes(y);
        match total_variation_distance_distribution(&p, &q) {
            Ok(distance) => println!("{x:?} vs {y:?}: {distance:.6}"),
            Err(error) => println!("{x:?} vs {y:?}: {error}"),
        }
    }
}
