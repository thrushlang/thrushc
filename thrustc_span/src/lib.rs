use std::fmt::{self, Display};

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: usize,
    pub span: (usize, usize),
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.line, self.span.0, self.span.1)
    }
}

impl Span {
    #[inline]
    pub fn new(line: usize, span: (usize, usize)) -> Self {
        Self { line, span }
    }

    #[inline]
    pub fn void() -> Self {
        Self::new(1, (0, 0))
    }
}

impl Span {
    #[inline]
    pub fn get_line(&self) -> usize {
        self.line
    }

    #[inline]
    pub fn get_span_start(&self) -> usize {
        self.span.0
    }

    #[inline]
    pub fn get_span_end(&self) -> usize {
        self.span.1
    }
}
