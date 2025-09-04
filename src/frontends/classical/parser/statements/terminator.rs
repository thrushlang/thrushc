use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, expr},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_return<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    let return_tk: &Token = parser_context.consume(
        TokenType::Return,
        String::from("Syntax error"),
        String::from("Expected 'return' keyword."),
    )?;

    let span: Span = return_tk.get_span();

    if parser_context.match_token(TokenType::SemiColon)? {
        if parser_context
            .get_type_ctx()
            .get_function_type()
            .is_void_type()
        {
            return Ok(Ast::Null { span });
        }

        return Ok(Ast::Return {
            expression: None,
            kind: Type::Void,
            span,
        });
    }

    let value: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    Ok(Ast::Return {
        expression: Some(value.into()),
        kind: parser_context.get_type_ctx().get_function_type().clone(),
        span,
    })
}

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)?;
    checks::check_inside_function_state(parser_context)
}
