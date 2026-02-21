use std::fmt::{self, Display};

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub line: u32,
    pub span: (u32, u32),
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.line, self.span.0, self.span.1)
    }
}

impl Span {
    #[inline]
    pub fn new(span: (u32, (u32, u32))) -> Self {
        let line: u32 = span.0;
        let start: u32 = span.1.0;
        let end: u32 = span.1.1;

        Self {
            line,
            span: (start, end),
        }
    }

    #[inline]
    pub fn nothing() -> Self {
        Self::new((1, (0, 0)))
    }
}

impl Span {
    #[inline]
    pub fn get_line(&self) -> u32 {
        self.line
    }

    #[inline]
    pub fn get_span_start(&self) -> u32 {
        self.span.0
    }

    #[inline]
    pub fn get_span_end(&self) -> u32 {
        self.span.1
    }
}
