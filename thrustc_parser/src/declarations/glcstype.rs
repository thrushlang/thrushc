use thrustc_ast::Ast;
use thrustc_attributes::ThrustAttributes;
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::Type;

use crate::{ParserContext, attributes, typegen};

pub fn build_custom_type<'parser>(
    ctx: &mut ParserContext<'parser>,
    parse_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Type,
        CompilationIssueCode::E0001,
        "Expected 'type' keyword.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();
    let span: Span = name_tk.get_span();

    let attributes: ThrustAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::LBrace])?;

    ctx.consume(
        TokenType::Eq,
        CompilationIssueCode::E0001,
        "Expected '='.".into(),
    )?;

    let custom_type: Type = typegen::build_type(ctx, false)?;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        "Expected ';'.".into(),
    )?;

    if parse_forward {
        ctx.get_mut_symbols()
            .new_global_custom_type(name, (custom_type, attributes), span)?;

        Ok(Ast::new_nullptr(span))
    } else {
        Ok(Ast::CustomType {
            kind: custom_type,
            span,
        })
    }
}
