use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;

use std::path::PathBuf;

pub fn parse_scapes(content: &str, span: Span) -> Result<Vec<u8>, CompilationIssue> {
    let source: &[u8] = content.as_bytes();

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
            return Err(CompilationIssue::FrontEndBug(
                "Byte not caught".into(),
                "Unable to get byte for determinate next byte to parse at scape sequence parsing."
                    .into(),
                span,
                CompilationPosition::Lexer,
                PathBuf::from(file!()),
                line!(),
            ));
        }
    }

    Ok(parsed_string)
}
