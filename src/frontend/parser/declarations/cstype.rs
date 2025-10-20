use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, attributes, checks, typegen},
        types::{
            ast::Ast,
            parser::stmts::{traits::TokenExtensions, types::ThrushAttributes},
        },
        typesystem::types::Type,
    },
};

pub fn build_custom_type<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::Type,
        String::from("Syntax error"),
        String::from("Expected 'type' keyword."),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected type name."),
    )?;

    let custom_type_name: &str = name.get_lexeme();

    let span: Span = name.get_span();

    ctx.consume(
        TokenType::Eq,
        String::from("Syntax error"),
        String::from("Expected '='."),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    let custom_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        String::from("Syntax error"),
        String::from("Expected ';'."),
    )?;

    if declare_forward {
        ctx.get_mut_symbols()
            .new_custom_type(custom_type_name, (custom_type, attributes), span)?;
    }

    Ok(Ast::new_nullptr(span))
}
