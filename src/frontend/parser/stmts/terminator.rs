use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{ast::Ast, lexer::ThrushType, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_return<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let return_tk: &Token = parser_ctx.consume(
        TokenType::Return,
        String::from("Syntax error"),
        String::from("Expected 'return' keyword."),
    )?;

    let span: Span = return_tk.get_span();

    if !parser_ctx.get_control_ctx().get_inside_function()
        && !parser_ctx.get_control_ctx().get_inside_bind()
    {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Return outside of bind or function."),
            None,
            span,
        ));
    }

    if parser_ctx.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreacheable code."),
            None,
            span,
        ));
    }

    if parser_ctx.match_token(TokenType::SemiColon)? {
        if parser_ctx.get_type_ctx().get_function_type().is_void_type() {
            return Ok(Ast::Null { span });
        }

        return Ok(Ast::Return {
            expression: None,
            kind: ThrushType::Void,
            span,
        });
    }

    let value: Ast = expression::build_expr(parser_ctx)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Return {
        expression: Some(value.into()),
        kind: parser_ctx.get_type_ctx().get_function_type().clone(),
        span,
    })
}
