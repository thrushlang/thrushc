use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::attributes;
use crate::frontend::parser::checks;
use crate::frontend::parser::typegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use crate::frontend::typesystem::types::Type;

pub fn build_custom_type<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

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
