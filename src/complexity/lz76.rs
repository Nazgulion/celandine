#[doc = include_str!("../../docs/complexity/lz76.md")]
pub fn lz76_complexity(data: &[u8]) -> usize {
    let mut start = 0;
    let mut phrases = 0;
    while start < data.len() {
        let remaining = data.len() - start;
        let mut longest = 0;
        // Every possible copy source starts strictly before this phrase.
        // Source and destination may overlap: at each offset, the source byte
        // precedes its destination and is already available during reproduction.
        for source in 0..start {
            let mut matched = 0;
            while matched < remaining && data[source + matched] == data[start + matched] {
                matched += 1;
            }
            if matched == remaining {
                // A reproducible final suffix is still one component. No extra
                // empty phrase is appended when reproduction reaches the end.
                return phrases + 1;
            }
            longest = longest.max(matched);
        }
        // All sources failed before the end. Copy the longest match and append
        // one new byte; this is the shortest non-reproducible phrase.
        start += longest + 1;
        phrases += 1;
    }
    phrases
}
