use super::Compressor;
use miniz_oxide::deflate::core::{
    CompressorOxide, TDEFLFlush, TDEFLStatus, compress, create_comp_flags_from_zip_params,
};
use std::io;

/// Fresh zlib streams using miniz_oxide 0.9.1, level 6, default strategy.
///
/// Available with `compression-deflate`. Uses a 32 KiB LZ77 window, no preset
/// dictionary, RFC 1950 framing and Adler-32 checksum. Settings are fixed and
/// every call starts a new stream. The exact dependency version is pinned because
/// encoder changes can alter measured sizes. See the module for history and limits.
///
/// Time is O(n) for n input bytes at these fixed search/window limits; auxiliary
/// memory is O(1), including compressor state and a 32 KiB output scratch buffer.
/// Output is counted and discarded. Initialization is included in every call.
///
/// ```
/// use celandine::compression::{Zlib, compressed_length, compression_ratio};
/// assert_eq!(compressed_length(b"A", &Zlib).unwrap(), 9);
/// assert_eq!(compression_ratio(b"A", &Zlib).unwrap(), 9.0);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Zlib;

impl Compressor for Zlib {
    type Error = io::Error;

    /// # Errors
    /// Returns an I/O error if the backend fails or the byte count exceeds usize.
    fn compressed_len(&self, input: &[u8]) -> io::Result<usize> {
        encode(input, |_| {})
    }
}

// The observer lets tests inspect the exact stream counted in production without
// making encoding or stream export part of the public measurement API.
fn encode(mut input: &[u8], mut observe: impl FnMut(&[u8])) -> io::Result<usize> {
    let flags = create_comp_flags_from_zip_params(6, 15, 0);
    let mut state = CompressorOxide::new(flags);
    let mut output = [0u8; 32768];
    let mut length = 0usize;
    loop {
        let (status, consumed, written) =
            compress(&mut state, input, &mut output, TDEFLFlush::Finish);
        length = length
            .checked_add(written)
            .ok_or_else(|| io::Error::other("compressed byte count exceeds usize"))?;
        observe(&output[..written]);
        input = &input[consumed..];
        match status {
            TDEFLStatus::Done if input.is_empty() => return Ok(length),
            TDEFLStatus::Okay if consumed != 0 || written != 0 => {}
            _ => {
                return Err(io::Error::other(
                    "zlib compressor failed to finish the stream",
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compression_ratio;

    /// Emit actual counted streams for Python's independent C-zlib decoder.
    #[test]
    #[ignore = "run through scripts/reference_compression.py --check"]
    fn export_reference_streams() {
        let mut samples = vec![vec![], b"A".to_vec(), b"ABABAB".to_vec()];
        for size in [2, 16, 256, 32767, 32768, 32769, 65536, 131073] {
            samples.push(vec![b'A'; size]);
            samples.push((0..size).map(|i| i as u8).collect());
            let mut state = 0x1234_5678_9abc_def0u64;
            samples.push(
                (0..size)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        (state >> 56) as u8
                    })
                    .collect(),
            );
        }
        fn hex(bytes: &[u8]) -> String {
            use std::fmt::Write;
            let mut text = String::new();
            for byte in bytes {
                write!(text, "{byte:02x}").unwrap();
            }
            text
        }
        for input in samples {
            let mut stream = Vec::new();
            let length = encode(&input, |bytes| stream.extend_from_slice(bytes)).unwrap();
            assert_eq!(length, Zlib.compressed_len(&input).unwrap());
            println!(
                "CELANDINE_COMPRESSION_REFERENCE\t{}\t{}\t{}\t{}",
                hex(&input),
                hex(&stream),
                length,
                compression_ratio(&input, &Zlib).map_or_else(|_| "empty".into(), |v| v.to_string())
            );
        }
    }
    #[test]
    #[ignore = "run through scripts/reference_ncd.py --check"]
    fn export_ncd_streams() {
        use crate::compression::{
            compression_distance_components, normalized_compression_distance,
        };
        let samples = [
            vec![],
            b"A".to_vec(),
            b"ABABAB".to_vec(),
            vec![0; 32769],
            (0..32769).map(|i| i as u8).collect(),
            (0..65536).map(|i| (i * 7) as u8).collect(),
            b"\0\xff\0".to_vec(),
        ];
        fn hex(bytes: &[u8]) -> String {
            use std::fmt::Write;
            let mut result = String::new();
            for byte in bytes {
                write!(result, "{byte:02x}").unwrap();
            }
            result
        }
        for i in 0..samples.len() {
            for j in [i, (i + 1) % samples.len()] {
                let x = &samples[i];
                let y = &samples[j];
                let xy = [x.as_slice(), y.as_slice()].concat();
                let stream = |data: &[u8]| {
                    let mut out = Vec::new();
                    encode(data, |bytes| out.extend_from_slice(bytes)).unwrap();
                    hex(&out)
                };
                let parts = compression_distance_components(x, y, &Zlib).unwrap();
                println!(
                    "CELANDINE_NCD_REFERENCE\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    hex(x),
                    hex(y),
                    stream(x),
                    stream(y),
                    stream(&xy),
                    parts.x,
                    parts.y,
                    parts.xy,
                    normalized_compression_distance(x, y, &Zlib).unwrap()
                );
            }
        }
    }
}
