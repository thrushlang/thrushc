use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::lexer::{Lexer, keywords::KEYWORDS, tokentype::TokenType},
};

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    while lexer.is_identifier_boundary(lexer.peek()) {
        lexer.advance();
    }

    if let Some(keyword) = KEYWORDS.get(lexer.lexeme().as_str()) {
        lexer.make(*keyword);
    } else {
        lexer.make(TokenType::Identifier);
    }

    Ok(())
}
