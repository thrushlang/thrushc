use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::tokentype::TokenType, types::parser::stmts::traits::TokenExtensions,
    },
};

use super::span::Span;

#[derive(Debug)]
pub struct Token {
    pub lexeme: String,
    pub ascii_lexeme: String,
    pub kind: TokenType,
    pub span: Span,
}

impl TokenExtensions for Token {
    fn scape(&self, span: Span) -> Result<Vec<u8>, ThrushCompilerIssue> {
        let source: &[u8] = self.lexeme.as_bytes();

        let mut parsed_string: Vec<u8> = Vec::with_capacity(source.len());

        let mut idx: usize = 0;

        while idx < source.len() {
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

                        _ => (),
                    }

                    idx += 1;
                    continue;
                }

                parsed_string.push(source[idx]);

                idx += 1;
            } else {
                return Err(ThrushCompilerIssue::FrontEndBug(
                    "Byte not caught".into(),
                    "Unable to get byte for determinate next byte to parse at scape sequence parsing.".into(),
                    span,
                    CompilationPosition::Lexer,
                    line!()
                ));
            }
        }

        Ok(parsed_string)
    }

    #[inline]
    fn get_lexeme(&self) -> &str {
        &self.lexeme
    }

    #[inline]
    fn get_ascii_lexeme(&self) -> &str {
        &self.ascii_lexeme
    }

    #[inline]
    fn get_span(&self) -> Span {
        self.span
    }

    #[inline]
    fn get_type(&self) -> TokenType {
        self.kind
    }

    #[inline]
    fn get_lexeme_first_byte(&self) -> u64 {
        self.lexeme.as_bytes()[0] as u64
    }
}
