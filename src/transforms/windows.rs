use std::{fmt, iter::FusedIterator};

/// A zero window size or step, which cannot define this finite-window traversal.
///
/// Both parameters count bytes and must be positive, even for empty input.
/// For example, `windows(b"AB", 1, 0)` returns `ZeroStep`. If both are zero,
/// `ZeroSize` takes precedence. No iterator is created on error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidWindowParameters {
    /// Window size is zero; every yielded slice must contain at least one byte.
    ZeroSize,
    /// Step is zero; successive start positions must advance.
    ZeroStep,
}

impl fmt::Display for InvalidWindowParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroSize => "window size must be greater than zero",
            Self::ZeroStep => "window step must be greater than zero",
        })
    }
}
impl std::error::Error for InvalidWindowParameters {}

#[doc = include_str!("../../docs/transforms/windows.md")]
pub fn windows(
    data: &[u8],
    size: usize,
    step: usize,
) -> Result<
    impl ExactSizeIterator<Item = &[u8]> + DoubleEndedIterator + FusedIterator + Clone + fmt::Debug,
    InvalidWindowParameters,
> {
    if size == 0 {
        return Err(InvalidWindowParameters::ZeroSize);
    }
    if step == 0 {
        return Err(InvalidWindowParameters::ZeroStep);
    }
    let count = if size > data.len() {
        0
    } else {
        1 + (data.len() - size) / step
    };
    Ok(WindowIter {
        data,
        size,
        step,
        front: 0,
        back: count,
    })
}

// Ordinals describe the selected forward grid; reverse traversal shares it.
// Bound skips by remaining count BEFORE offset arithmetic. StepBy::nth can spend
// enormous time resolving multiplication overflow even on a tiny exhausted slice.
#[derive(Clone, Debug)]
struct WindowIter<'a> {
    data: &'a [u8],
    size: usize,
    step: usize,
    front: usize,
    back: usize,
}
impl<'a> WindowIter<'a> {
    fn block(&self, ordinal: usize) -> &'a [u8] {
        // ordinal < initial count implies ordinal*step <= data.len()-size.
        // Thus both arithmetic operations below fit and describe a valid slice.
        let start = ordinal * self.step;
        &self.data[start..start + self.size]
    }
}
impl<'a> Iterator for WindowIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let ordinal = self.front;
        self.front += 1;
        Some(self.block(ordinal))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.front = self.back;
            return None;
        }
        self.front += n;
        self.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
    fn count(self) -> usize {
        self.len()
    }
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}
impl DoubleEndedIterator for WindowIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        Some(self.block(self.back))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.front = self.back;
            return None;
        }
        self.back -= n;
        self.next_back()
    }
}
impl ExactSizeIterator for WindowIter<'_> {
    fn len(&self) -> usize {
        self.back - self.front
    }
}
impl FusedIterator for WindowIter<'_> {}
