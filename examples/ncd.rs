//! Run: cargo run --locked --features compression-deflate --example ncd
use celandine::compression::{Zlib, compression_distance_components};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NCD compares compressed sizes: [C(xy) - min(C(x), C(y))] / max(C(x), C(y)).");
    println!(
        "C counts complete zlib bytes; xy is left then right without a separator. NCD is dimensionless."
    );
    println!(
        "Fixed miniz_oxide 0.9.1 level 6; headers retained. No symmetry, range, or zero-self-distance correction."
    );
    let repeated = vec![b'A'; 4096];
    let other = vec![b'B'; 4096];
    for (label, x, y) in [
        ("both empty", b"".as_slice(), b"".as_slice()),
        ("empty / A", b"", b"A"),
        ("A / A", b"A", b"A"),
        (
            "same repeated input",
            repeated.as_slice(),
            repeated.as_slice(),
        ),
        (
            "different repeated inputs",
            repeated.as_slice(),
            other.as_slice(),
        ),
    ] {
        let parts = compression_distance_components(x, y, &Zlib)?;
        println!(
            "{label}: C(x)={}, C(y)={}, C(xy)={}, NCD={:.9}",
            parts.x,
            parts.y,
            parts.xy,
            parts.normalized_distance()?
        );
    }
    let parts = compression_distance_components(b"", b"A", &Zlib)?;
    assert_eq!((parts.x, parts.y, parts.xy), (8, 9, 9));
    assert_eq!(parts.normalized_distance()?, 1.0 / 9.0);
    println!("Empty / A: (9 - 8) / 9 = 1/9. Shared framing affects short inputs.");
    Ok(())
}
