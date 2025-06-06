use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{lexer::tokenkind::TokenKind, types::parser::stmts::traits::TokenExtensions},
};

use super::span::Span;

#[derive(Debug)]
pub struct Token<'token> {
    pub lexeme: &'token str,
    pub kind: TokenKind,
    pub span: Span,
}

impl TokenExtensions for str {
    fn to_bytes(&self, span: Span) -> Result<Vec<u8>, ThrushCompilerIssue> {
        let source: &[u8] = self.as_bytes();

        let mut parsed_string: Vec<u8> = Vec::with_capacity(source.len());

        let mut idx: usize = 0;

        while idx < self.len() {
            if let Some(byte) = source.get(idx) {
                if *byte == b'\\' {
                    idx += 1;

                    match source.get(idx) {
                        Some(b'n') => parsed_string.push(b'\n'),
                        Some(b't') => parsed_string.push(b'\t'),
                        Some(b'r') => parsed_string.push(b'\r'),
                        Some(b'\\') => parsed_string.push(b'\\'),
                        Some(b'0') => parsed_string.push(b'\0'),
                        Some(b'\'') => parsed_string.push(b'\''),
                        Some(b'"') => parsed_string.push(b'"'),
                        _ => {
                            return Err(ThrushCompilerIssue::Error(
                                String::from("Syntax Error"),
                                String::from("Invalid escape sequence."),
                                None,
                                span,
                            ));
                        }
                    }

                    idx += 1;

                    continue;
                }

                parsed_string.push(source[idx]);

                idx += 1;
            } else {
                return Err(ThrushCompilerIssue::Bug(
                    "Byte not caught".into(),
                    "Unable to get byte for determinate next byte to parse at scape sequence parser.".into(),
                    span,
                    CompilationPosition::Lexer,
                    line!()
                ));
            }
        }

        Ok(parsed_string)
    }

    fn get_first_byte(&self) -> u64 {
        self.as_bytes()[0] as u64
    }
}
