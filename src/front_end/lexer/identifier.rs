use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::Lexer;
use crate::front_end::lexer::atomic::ATOMIC;
use crate::front_end::lexer::attributes::ATTRIBUTES;
use crate::front_end::lexer::builtins::BUILTINS;
use crate::front_end::lexer::keywords::KEYWORDS;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::lexer::types::TYPES;

pub fn lex(lexer: &mut Lexer) -> Result<(), CompilationIssue> {
    while lexer.is_identifier_boundary(lexer.peek()) {
        lexer.advance_only();
    }

    let bytes: Vec<u8> = lexer.lexeme_bytes();

    if let Some(keyword) = KEYWORDS.get(bytes.as_slice()) {
        lexer.make(*keyword);
    } else if let Some(atomic_stuff) = ATOMIC.get(bytes.as_slice()) {
        lexer.make(*atomic_stuff);
    } else if let Some(attribute) = ATTRIBUTES.get(bytes.as_slice()) {
        lexer.make(*attribute);
    } else if let Some(builtin) = BUILTINS.get(bytes.as_slice()) {
        lexer.make(*builtin);
    } else if let Some(r#type) = TYPES.get(bytes.as_slice()) {
        lexer.make(*r#type);
    } else {
        lexer.make(TokenType::Identifier);
    }

    Ok(())
}
