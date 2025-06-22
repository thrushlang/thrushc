use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expressions::reference, typegen},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn build_sizeof<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let sizeof_tk: &Token = parser_context.consume(
        TokenType::SizeOf,
        String::from("Syntax error"),
        String::from("Expected 'sizeof' keyword."),
    )?;

    parser_context.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let sizeof_span: Span = sizeof_tk.get_span();

    if parser_context.match_token(TokenType::Identifier)? {
        let identifier_tk: &Token = parser_context.previous();

        let name: &str = identifier_tk.get_lexeme();
        let span: Span = identifier_tk.get_span();

        let reference: ThrushStatement = reference::build_reference(parser_context, name, span)?;

        let reference_type: &ThrushType = reference.get_value_type()?;

        parser_context.consume(
            TokenType::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        return Ok(ThrushStatement::SizeOf {
            sizeof: reference_type.clone(),
            kind: ThrushType::U64,
            span: sizeof_span,
        });
    }

    let sizeof_type: ThrushType = typegen::build_type(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ThrushStatement::SizeOf {
        sizeof: sizeof_type,
        kind: ThrushType::U64,
        span: sizeof_span,
    })
}
