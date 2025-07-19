use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::reference, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_write<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let write_tk: &Token = parser_context.consume(
        TokenType::Write,
        "Syntax error".into(),
        "Expected 'write' keyword.".into(),
    )?;

    let span: Span = write_tk.span;

    if parser_context.match_token(TokenType::Identifier)? {
        let identifier_tk: &Token = parser_context.previous();
        let name: &str = identifier_tk.get_lexeme();

        let reference: Ast = reference::build_reference(parser_context, name, span)?;

        parser_context.consume(
            TokenType::Comma,
            "Syntax error".into(),
            "Expected ','.".into(),
        )?;

        let write_type: Type = typegen::build_type(parser_context)?;

        let value: Ast = expr::build_expr(parser_context)?;

        return Ok(Ast::Write {
            source: (Some((name, reference.into())), None),
            write_value: value.clone().into(),
            write_type,
            span,
        });
    }

    let expression: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let write_type: Type = typegen::build_type(parser_context)?;
    let value: Ast = expr::build_expr(parser_context)?;

    Ok(Ast::Write {
        source: (None, Some(expression.into())),
        write_value: value.clone().into(),
        write_type,
        span,
    })
}
