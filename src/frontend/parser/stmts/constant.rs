use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, checks, expr, typegen},
        types::{
            ast::{Ast, metadata::constant::ConstantMetadata},
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_const<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_state(parser_context)?;

    parser_context.consume(
        TokenType::Const,
        "Syntax error".into(),
        "Expected 'const' keyword.".into(),
    )?;

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

    let const_attributes: ThrushAttributes =
        attributes::build_attributes(parser_context, &[TokenType::Eq])?;

    parser_context.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

    let value: Ast = expr::build_expr(parser_context)?;

    let expression_span: Span = value.get_span();

    if !value.is_constant_value() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Expected integer, floating-point, boolean, string, fixed array, or char constant types.".into(),
            None,
            expression_span,
        ));
    }

    parser_context.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    parser_context.get_mut_symbols().new_constant(
        name,
        (const_type.clone(), const_attributes.clone()),
        span,
    )?;

    Ok(Ast::Const {
        name,
        ascii_name,
        kind: const_type,
        value: value.into(),
        attributes: const_attributes,
        metadata: ConstantMetadata::new(false),
        span,
    })
}

fn check_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)?;
    checks::check_inside_function_state(parser_context)
}
