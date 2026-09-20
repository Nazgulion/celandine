//! Test-only oracles shared by libFuzzer targets and stable seed replay.
#![forbid(unsafe_code)]

use celandine::distribution::{ByteHistogram, Distribution, ngram_counts, ngram_probabilities};
use celandine::entropy::*;
use std::f64::consts::LN_2;

// Short inputs are zero-padded rather than discarded. Every mutation is exercised.
fn word(data: &[u8], start: usize) -> u64 {
    let mut bytes = [0; 8];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = data.get(start + i).copied().unwrap_or(0);
    }
    u64::from_le_bytes(bytes)
}

fn close(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite());
    assert!(
        (actual - expected).abs() <= 1e-11 * expected.abs().max(1.0),
        "actual={actual:e}, expected={expected:e}"
    );
}

fn check_distribution(d: &Distribution, counts: &[usize; 256], order: f64, base: f64) {
    let total = counts.iter().map(|&c| c as u128).sum::<u128>();
    let support = counts.iter().filter(|&&c| c != 0).count();
    assert_eq!(d.sample_size() as u128, total);
    assert_eq!(d.support_size(), support);
    assert_eq!(d.is_empty(), total == 0);
    let p: Vec<_> = counts
        .iter()
        .map(|&c| c as f64 / total.max(1) as f64)
        .collect();
    assert_eq!(d.probabilities().len(), 256);
    for (i, actual) in d.probabilities().enumerate() {
        assert_eq!(actual, p[i]);
        assert_eq!(d.probability(i as u8), p[i]);
    }
    close(p.iter().sum(), if total == 0 { 0.0 } else { 1.0 });
    let h = shannon_distribution(d);
    let h0 = hartley_entropy_distribution(d);
    let h2 = collision_entropy_distribution(d);
    let hi = min_entropy_distribution(d);
    let reference_h: f64 = p
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.ln() / LN_2)
        .sum();
    close(h, reference_h);
    close(h0, (support.max(1) as f64).ln() / LN_2);
    if support > 0 {
        close(h2, -p.iter().map(|p| p * p).sum::<f64>().ln() / LN_2);
        close(hi, -p.iter().copied().fold(0.0, f64::max).ln() / LN_2);
    }
    for value in [h, h0, h2, hi] {
        assert!(value.is_finite() && (0.0..=8.0 + 1e-11).contains(&value));
        if support <= 1 {
            assert_eq!(value.to_bits(), 0.0_f64.to_bits());
        }
    }
    assert!(hi <= h2 + 1e-11 && h2 <= h + 1e-11 && h <= h0 + 1e-11);
    assert_eq!(renyi_entropy_distribution(d, 0.0), Ok(h0));
    assert_eq!(renyi_entropy_distribution(d, 1.0), Ok(h));
    assert_eq!(renyi_entropy_distribution(d, 2.0), Ok(h2));
    assert_eq!(renyi_entropy_distribution(d, f64::INFINITY), Ok(hi));
    assert_eq!(tsallis_entropy_distribution(d, 1.0), Ok(h));
    assert_eq!(
        shannon_distribution_with_base(d, 2.0).unwrap().to_bits(),
        h.to_bits()
    );

    let renyi = renyi_entropy_distribution(d, order);
    assert_eq!(renyi.is_ok(), !order.is_nan() && order >= 0.0);
    if let Ok(value) = renyi {
        assert!(value.is_finite() && value >= 0.0);
        assert!(hi <= value + 1e-11 && value <= h0 + 1e-11);
        if support <= 1 {
            assert_eq!(value.to_bits(), 0.0_f64.to_bits());
        } else if (0.25..=8.0).contains(&order) && (order - 1.0).abs() >= 0.01 {
            // Direct powers/logarithms are an independent, well-conditioned oracle here.
            let sum: f64 = p.iter().filter(|&&p| p > 0.0).map(|p| p.powf(order)).sum();
            close(value, sum.ln() / (1.0 - order) / LN_2);
        }
    }
    let tsallis = tsallis_entropy_distribution(d, order);
    assert_eq!(tsallis.is_ok(), order.is_finite() && order >= 0.0);
    if let Ok(value) = tsallis {
        assert!(value.is_finite() && value >= 0.0);
        assert!(value <= support.saturating_sub(1) as f64 / LN_2 + 1e-11);
        if support <= 1 {
            assert_eq!(value.to_bits(), 0.0_f64.to_bits());
        } else if (0.0..=8.0).contains(&order) && (order - 1.0).abs() >= 0.01 {
            let sum: f64 = p.iter().filter(|&&p| p > 0.0).map(|p| p.powf(order)).sum();
            close(value, (1.0 - sum) / (order - 1.0) / LN_2);
        }
    }
    let scaled = shannon_distribution_with_base(d, base);
    assert_eq!(scaled.is_ok(), base.is_finite() && base > 1.0);
    if let Ok(value) = scaled {
        assert!(value.is_finite() && value >= 0.0);
        // Compare in nats so bases immediately above one do not magnify the tolerance.
        close(value * base.ln(), reference_h * LN_2);
        if support <= 1 {
            assert_eq!(value.to_bits(), 0.0_f64.to_bits());
        }
    }
}

/// Two little-endian f64 bit patterns followed by at most 4096 input bytes.
pub fn byte_entropy(input: &[u8]) {
    let order = f64::from_bits(word(input, 0));
    let base = f64::from_bits(word(input, 8));
    let data = &input[input.len().min(16)..input.len().min(4112)];
    // Sorting/run counting is independent of the library's direct histogram tally.
    let mut sorted = data.to_vec();
    sorted.sort_unstable();
    let mut counts = [0; 256];
    for run in sorted.chunk_by(|a, b| a == b) {
        counts[run[0] as usize] = run.len();
    }
    let d = Distribution::from_bytes(data);
    assert_eq!(d.histogram().counts(), &counts);
    check_distribution(&d, &counts, order, base);
    for (direct, reused) in [
        (shannon(data), shannon_distribution(&d)),
        (hartley_entropy(data), hartley_entropy_distribution(&d)),
        (collision_entropy(data), collision_entropy_distribution(&d)),
        (min_entropy(data), min_entropy_distribution(&d)),
    ] {
        assert_eq!(direct.to_bits(), reused.to_bits());
    }
    assert_eq!(
        renyi_entropy(data, order),
        renyi_entropy_distribution(&d, order)
    );
    assert_eq!(
        tsallis_entropy(data, order),
        tsallis_entropy_distribution(&d, order)
    );
    assert_eq!(
        shannon_with_base(data, base),
        shannon_distribution_with_base(&d, base)
    );
}

/// Two f64 bit patterns followed by up to 256 little-endian u64 counts.
pub fn count_entropy(input: &[u8]) {
    let order = f64::from_bits(word(input, 0));
    let base = f64::from_bits(word(input, 8));
    let counts = std::array::from_fn(|i| word(input, 16 + 8 * i) as usize);
    let exact_total: u128 = counts.iter().map(|&c| c as u128).sum();
    let result = ByteHistogram::try_from_counts(counts);
    assert_eq!(result.is_ok(), exact_total <= usize::MAX as u128);
    if let Ok(histogram) = result {
        assert_eq!(histogram.counts(), &counts);
        check_distribution(&Distribution::from_counts(histogram), &counts, order, base);
    }
}

/// A little-endian u64 block length followed by at most 512 bytes.
pub fn ngrams(input: &[u8]) {
    let n = word(input, 0) as usize;
    let data = &input[input.len().min(8)..input.len().min(520)];
    let blocks = celandine::transforms::ngrams(data, n);
    let counts = ngram_counts(data, n);
    let distribution = ngram_probabilities(data, n);
    assert_eq!(blocks.is_ok(), n > 0);
    assert_eq!(counts.is_ok(), n > 0);
    assert_eq!(distribution.is_ok(), n > 0);
    if n == 0 {
        return;
    }
    let blocks = blocks.unwrap();
    let counts = counts.unwrap();
    let d = distribution.unwrap();
    let total = if n > data.len() {
        0
    } else {
        data.len() - n + 1
    };
    // Explicit positions and sorted runs, rather than windows plus a tree map.
    let mut expected: Vec<_> = (0..total).map(|i| &data[i..i + n]).collect();
    assert_eq!(blocks.len(), total);
    for (actual, expected) in blocks.clone().zip(&expected) {
        assert_eq!(actual, *expected);
        assert_eq!(actual.as_ptr(), expected.as_ptr());
    }
    assert!(blocks.rev().eq(expected.iter().rev().copied()));
    expected.sort_unstable();
    let runs: Vec<_> = expected
        .chunk_by(|a, b| a == b)
        .map(|run| (run[0], run.len()))
        .collect();
    assert_eq!(counts.total(), total);
    assert_eq!(counts.ngram_len(), n);
    assert_eq!(counts.is_empty(), total == 0);
    assert_eq!(counts.support_size(), runs.len());
    assert_eq!(counts.counts().collect::<Vec<_>>(), runs);
    assert_eq!(d.counts(), &counts);
    assert_eq!(d.sample_size(), total);
    assert_eq!(d.ngram_len(), n);
    assert_eq!(d.support_size(), runs.len());
    assert_eq!(d.is_empty(), total == 0);
    assert_eq!(d.probabilities().len(), runs.len());
    for ((block, p), &(expected, count)) in d.probabilities().zip(&runs) {
        assert_eq!(block, expected);
        assert_eq!(counts.count(block), count);
        assert_eq!(d.probability(block), p);
        close(p, count as f64 / total as f64);
    }
    close(
        d.probabilities().map(|(_, p)| p).sum(),
        if total == 0 { 0.0 } else { 1.0 },
    );
    assert_eq!(counts.count(b""), 0);
    assert_eq!(d.probability(b"").to_bits(), 0.0_f64.to_bits());
    // Query an arbitrary prefix, including absent blocks and wrong lengths.
    let query = &data[..data.len().min(data.first().copied().unwrap_or(0) as usize)];
    let expected_count = expected.iter().filter(|&&block| block == query).count();
    assert_eq!(counts.count(query), expected_count);
    close(
        d.probability(query),
        expected_count as f64 / total.max(1) as f64,
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn replay_committed_seeds() {
        for (target, check) in [
            ("byte_entropy", super::byte_entropy as fn(&[u8])),
            ("count_entropy", super::count_entropy),
            ("ngrams", super::ngrams),
        ] {
            let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("seeds")
                .join(target);
            let mut paths: Vec<_> = std::fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            paths.sort();
            assert!(!paths.is_empty(), "missing seeds for {target}");
            for path in paths {
                eprintln!("replaying {}", path.display());
                check(&std::fs::read(path).unwrap());
            }
        }
    }
}
