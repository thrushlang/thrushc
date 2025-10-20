use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub line: usize,
    pub span: (usize, usize),
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.span.0)
    }
}

impl Span {
    #[inline]
    pub fn new(line: usize, span: (usize, usize)) -> Self {
        Self { line, span }
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
