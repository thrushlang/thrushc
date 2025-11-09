use crate::{
    core::errors::standard::ThrushCompilerIssue,
    front_end::lexer::{
        Lexer, atomic::ATOMIC, attributes::ATTRIBUTES, builtins::BUILTINS, keywords::KEYWORDS,
        tokentype::TokenType, types::TYPES,
    },
};

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    while lexer.is_identifier_boundary(lexer.peek()) {
        lexer.advance_only();
    }

    let lexeme: String = lexer.lexeme();
    let content: &str = lexeme.as_str();

    if let Some(keyword) = KEYWORDS.get(content) {
        lexer.make(*keyword);
    } else if let Some(atomic_stuff) = ATOMIC.get(content) {
        lexer.make(*atomic_stuff);
    } else if let Some(attribute) = ATTRIBUTES.get(content) {
        lexer.make(*attribute);
    } else if let Some(builtin) = BUILTINS.get(content) {
        lexer.make(*builtin);
    } else if let Some(r#type) = TYPES.get(content) {
        lexer.make(*r#type);
    } else {
        lexer.make(TokenType::Identifier);
    }

    Ok(())
}
