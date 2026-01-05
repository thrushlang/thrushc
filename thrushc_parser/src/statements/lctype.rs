use thrushc_ast::Ast;
use thrushc_attributes::ThrushAttributes;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::Type;

use crate::{ParserContext, attributes, typegen};

pub fn build_custom_type<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Type,
        CompilationIssueCode::E0001,
        "Expected 'type' keyword.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected type name.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();
    let span: Span = name_tk.get_span();

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        String::from("Expected '='."),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    let custom_type: Type = typegen::build_type(ctx, false)?;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    ctx.get_mut_symbols()
        .new_custom_type(name, (custom_type.clone(), attributes), span)?;

    Ok(Ast::CustomType {
        kind: custom_type,
        span,
    })
}
