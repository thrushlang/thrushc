use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

pub fn build_custom_type<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Type,
        "Syntax error".into(),
        "Expected 'type' keyword.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected type name.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();
    let span: Span = name_tk.get_span();

    ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    let custom_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        "Syntax error".into(),
        "Expected ';'.".into(),
    )?;

    if declare_forward {
        ctx.get_mut_symbols()
            .new_global_custom_type(name, (custom_type, attributes), span)?;

        return Ok(Ast::new_nullptr(span));
    }

    Ok(Ast::CustomType {
        kind: custom_type,
        span,
    })
}
