use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::lexer::{Lexer, keywords::KEYWORDS, tokentype::TokenType},
};

pub fn lex(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    while lexer.is_identifier_boundary(lexer.peek()) {
        lexer.advance();
    }

    let mut lexeme: String = String::with_capacity(100);

    lexeme.extend(&lexer.code[lexer.start..lexer.current]);

    if let Some(keyword) = KEYWORDS.get(lexeme.as_str()) {
        lexer.make(*keyword);
    } else {
        lexer.make(TokenType::Identifier);
    }

    Ok(())
}
