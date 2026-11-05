//! Explicit LZ76 scales; see docs/complexity/lz76_normalized.md for history.
use celandine::complexity::Lz76Summary;
fn main() {
    println!("LZ76: c exhaustive phrases, N byte symbols, K declared alphabet size.");
    println!("Density = c/N; asymptotic = c*log_K(N)/N; both dimensionless.");
    println!("ABAB = A | B | AB: density 3/4; K=2 gives 1.5, which exceeds one.");
    println!("Empty input is zero by convention. A singleton has density 1, asymptotic 0.");
    println!("Finite scores are not exact entropy rates or randomness certificates.");
    for data in [b"".as_slice(), b"A", b"AAAA", b"ABAB", b"101101011"] {
        let s = Lz76Summary::from_bytes(data);
        println!(
            "{:?}: N={}, c={}, support={}, density={:.6}, K=2: {:.6}, K=256: {:.6}",
            String::from_utf8_lossy(data),
            s.len(),
            s.phrase_count(),
            s.support_size(),
            s.phrase_density(),
            s.asymptotic(2).unwrap(),
            s.asymptotic(256).unwrap()
        );
    }
    println!(
        "ABC with K=2: {}",
        Lz76Summary::from_bytes(b"ABC").asymptotic(2).unwrap_err()
    );
}
