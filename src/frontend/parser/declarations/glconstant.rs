use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, expr, typegen},
        types::{
            ast::{Ast, metadata::constant::ConstantMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_global_const<'parser>(
    parser_context: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_context.consume(
        TokenType::Const,
        "Syntax error".into(),
        "Expected 'const' keyword.".into(),
    )?;

    let is_lazy: bool = parser_context.match_token(TokenType::LazyThread)?;
    let is_volatile: bool = parser_context.match_token(TokenType::Volatile)?;

    let const_tk: &Token = parser_context.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected name.".into(),
    )?;

    let name: &str = const_tk.get_lexeme();
    let ascii_name: &str = const_tk.get_ascii_lexeme();

    let span: Span = const_tk.get_span();

    parser_context.consume(
        TokenType::Colon,
        "Syntax error".into(),
        "Expected ':'.".into(),
    )?;

    let const_type: Type = typegen::build_type(parser_context)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(parser_context, &[TokenType::Eq])?;

    parser_context.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expression(parser_context)?;

    if declare_forward {
        parser_context.get_mut_symbols().new_global_constant(
            name,
            (const_type.clone(), attributes.clone()),
            span,
        )?;
    }

    Ok(Ast::Const {
        name,
        ascii_name,
        kind: const_type,
        value: value.into(),
        attributes,
        metadata: ConstantMetadata::new(true, is_lazy, is_volatile),
        span,
    })
}
