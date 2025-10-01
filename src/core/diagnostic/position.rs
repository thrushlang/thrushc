use crate::frontends::classical::lexer::span::Span;

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

pub fn find_line_and_range(code: &str, span: Span) -> Option<CodePosition> {
    let start: usize = span.get_span_start();
    let end: usize = span.get_span_end();

    let mut line_start: usize = 0;
    let mut line_num: usize = 1;

    for (i, c) in code.char_indices() {
        if i >= start {
            break;
        }
        if c == '\n' {
            line_start = i;
            line_num += 1;
        }
    }

    if start >= code.len() || end > code.len() || start > end {
        return None;
    }

    Some(CodePosition::new(
        line_num,
        start.saturating_sub(line_start),
        end.saturating_sub(line_start),
    ))
}
