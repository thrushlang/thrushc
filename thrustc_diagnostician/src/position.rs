use thrustc_span::Span;

#[derive(Debug)]
pub struct CodePosition {
    line: usize,
    start: usize,
    end: usize,
}

impl CodePosition {
    #[inline]
    pub fn new(line: usize, start: usize, end: usize) -> Self {
        Self { line, start, end }
    }
}

impl CodePosition {
    #[inline]
    pub fn get_line(&self) -> usize {
        self.line
    }

    #[inline]
    pub fn get_start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn get_end(&self) -> usize {
        self.end
    }
}
