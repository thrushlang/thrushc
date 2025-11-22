use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::{attributes, checks, typegen};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::attributes::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;

use crate::front_end::types::parser::symbols::types::ParametersTypes;
use crate::front_end::typesystem::types::Type;

pub fn build_intrinsic<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    let intrinsic_tk: &Token = ctx.consume(
        TokenType::Intrinsic,
        "Syntax error".into(),
        "Expected 'intrinsic' keyword.".into(),
    )?;

    let span: Span = intrinsic_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let external_name_tk: &Token = ctx.consume(
        TokenType::Str,
        "Syntax error".into(),
        "Expected string literal.".into(),
    )?;

    let external_name: &str = external_name_tk.get_lexeme().trim();

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected 'identifier'.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameters_types: Vec<Type> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let parameter_name_tk: &Token = ctx.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected 'identifier'.".into(),
        )?;

        let span: Span = parameter_name_tk.get_span();

        ctx.consume(
            TokenType::Colon,
            "Syntax error".into(),
            "Expected ':'.".into(),
        )?;

        let kind: Type = typegen::build_type(ctx)?;

        parameters_types.push(kind.clone());

        parameters.push(Ast::IntrinsicParameter { kind, span });

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    let return_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::SemiColon])?;

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    let has_ignore: bool = attributes.has_ignore_attribute();

    if declare_forward {
        ctx.get_mut_symbols().new_intrinsic(
            name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                has_ignore,
            ),
            span,
        )?;

        return Ok(Ast::new_nullptr(span));
    }

    Ok(Ast::Intrinsic {
        name,
        external_name,
        parameters,
        parameters_types,
        return_type,
        attributes,
        span,
    })
}
