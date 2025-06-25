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

pub fn build_const<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenType::Const,
        String::from("Syntax error"),
        String::from("Expected 'const' keyword."),
    )?;

    let const_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name."),
    )?;

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Constants are only defined globally."),
            None,
            const_tk.span,
        ));
    }

    let name: &str = const_tk.get_lexeme();
    let span: Span = const_tk.get_span();

    parser_ctx.consume(
        TokenType::Colon,
        String::from("Syntax error"),
        String::from("Expected ':'."),
    )?;

    let const_type: ThrushType = typegen::build_type(parser_ctx)?;

    let const_attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::Eq])?;

    parser_ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let value: Ast = expression::build_expr(parser_ctx)?;

    value.throw_attemping_use_jit(span)?;

    parser_ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare_forward {
        if let Err(error) =
            parser_ctx
                .get_mut_symbols()
                .new_constant(name, (const_type, const_attributes), span)
        {
            parser_ctx.add_error(error);
        }

        return Ok(Ast::Null { span });
    }

    Ok(Ast::Const {
        name,
        kind: const_type,
        value: value.into(),
        attributes: const_attributes,
        span,
    })
}
