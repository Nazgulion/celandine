//! Reproducible paired workloads shared by timing and memory probes.
pub fn input(size: usize, shape: &str) -> (Vec<u8>, Vec<u8>) {
    let mut state = 0x1234_5678_9abc_def0u64;
    (0..size)
        .map(|i| match shape {
            "constant" => (0, 0),
            "uniform" => (i as u8, (i >> 8) as u8),
            "skewed" => (u8::from(i % 100 == 0), u8::from(i % 127 == 0)),
            "mixed" => {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                ((state >> 56) as u8, (state >> 48) as u8)
            }
            _ => unreachable!(),
        })
        .unzip()
}
