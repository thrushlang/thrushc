use std::path::{Path, PathBuf};

use crate::front_end::lexer::span::Span;

#[derive(Debug)]
pub struct PreprocessorIssue {
    path: PathBuf,
    title: String,
    description: String,
    span: Span,
}

impl PreprocessorIssue {
    #[inline]
    pub fn new(path: PathBuf, title: String, description: String, span: Span) -> Self {
        Self {
            path,
            title,
            description,
            span,
        }
    }
}

impl PreprocessorIssue {
    #[inline]
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn get_title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn get_description(&self) -> &str {
        &self.description
    }

    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }
}
