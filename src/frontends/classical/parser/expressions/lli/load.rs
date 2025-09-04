use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::reference, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_load<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let load_tk: &Token = parser_context.consume(
        TokenType::Load,
        "Syntax error".into(),
        "Expected 'load' keyword.".into(),
    )?;

    let span: Span = load_tk.get_span();

    let load_type: Type = typegen::build_type(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    if parser_context.check(TokenType::Identifier) {
        let identifier_tk: &Token = parser_context.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected 'identifier'.".into(),
        )?;

        let reference_name: &str = identifier_tk.get_lexeme();

        let reference: Ast = reference::build_reference(parser_context, reference_name, span)?;

        return Ok(Ast::Load {
            source: (Some((reference_name, reference.into())), None),
            kind: load_type,
            span,
        });
    }

    let expression: Ast = expr::build_expr(parser_context)?;

    Ok(Ast::Load {
        source: (None, Some(expression.into())),
        kind: load_type,
        span,
    })
}
