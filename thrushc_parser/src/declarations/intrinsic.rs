use thrushc_ast::Ast;
use thrushc_attributes::{ThrushAttributes, traits::ThrushAttributesExtensions};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, attributes, entities::ParametersTypes, typegen};

pub fn build_intrinsic<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    let intrinsic_tk: &Token = ctx.consume(
        TokenType::Intrinsic,
        CompilationIssueCode::E0001,
        "Expected 'intrinsic' keyword.".into(),
    )?;

    let span: Span = intrinsic_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let external_name_tk: &Token = ctx.consume(
        TokenType::Str,
        CompilationIssueCode::E0001,
        "Expected string literal.".into(),
    )?;

    let external_name: &str = external_name_tk.get_lexeme().trim();

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected 'identifier'.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
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
            CompilationIssueCode::E0001,
            "Expected 'identifier'.".into(),
        )?;

        let span: Span = parameter_name_tk.get_span();

        ctx.consume(
            TokenType::Colon,
            CompilationIssueCode::E0001,
            "Expected ':'.".into(),
        )?;

        let kind: Type = typegen::build_type(ctx, false)?;

        parameters_types.push(kind.clone());

        parameters.push(Ast::IntrinsicParameter { kind, span });

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    let return_type: Type = typegen::build_type(ctx, false)?;
    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::SemiColon])?;
    let has_ignore: bool = attributes.has_ignore_attribute();

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    if declare_forward {
        ctx.get_mut_symbols().new_intrinsic(
            name,
            (return_type, ParametersTypes(parameters_types), has_ignore),
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
