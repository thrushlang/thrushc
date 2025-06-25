use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, expression, typegen},
        types::{
            ast::Ast,
            lexer::ThrushType,
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
    },
};

pub fn build_local<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let local_tk: &Token = parser_ctx.consume(
        TokenType::Local,
        String::from("Syntax error"),
        String::from("Expected 'local' keyword."),
    )?;

    let span: Span = local_tk.get_span();

    if parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Locals variables should be contained at local scope."),
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

    let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;

    let local_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    let name: &str = local_tk.get_lexeme();
    let ascii_name: &str = local_tk.get_ascii_lexeme();

    let span: Span = local_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let local_type: ThrushType = typegen::build_type(parser_ctx)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::SemiColon, TokenType::Eq])?;

    if parser_ctx.match_token(TokenType::SemiColon)? {
        parser_ctx.get_mut_symbols().new_local(
            name,
            (local_type.clone(), is_mutable, true, span),
            span,
        )?;

        return Ok(Ast::Local {
            name,
            ascii_name,
            kind: local_type,
            value: Ast::Null { span }.into(),
            attributes,
            undefined: true,
            is_mutable,
            span,
        });
    }

    parser_ctx.get_mut_symbols().new_local(
        name,
        (local_type.clone(), is_mutable, false, span),
        span,
    )?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Ast = expression::build_expr(parser_ctx)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    let local: Ast = Ast::Local {
        name,
        ascii_name,
        kind: local_type,
        value: value.into(),
        attributes,
        undefined: false,
        is_mutable,
        span,
    };

    Ok(local)
}
