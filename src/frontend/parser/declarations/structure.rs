use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::attributes;
use crate::frontend::parser::builder;
use crate::frontend::parser::checks;
use crate::frontend::parser::typegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::{StructFieldsExtensions, TokenExtensions};
use crate::frontend::types::parser::stmts::types::{StructFields, ThrushAttributes};
use crate::frontend::typesystem::modificators::StructureTypeModificator;
use crate::frontend::typesystem::types::Type;

pub fn build_structure<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::Struct,
        "Syntax error".into(),
        "Expected 'struct' keyword.".into(),
    )?;

    let name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected identifier.".into(),
    )?;

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;
    let modificator: StructureTypeModificator = builder::build_structure_modificator(&attributes);

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let name: &str = name_tk.get_lexeme();
    let span: Span = name_tk.get_span();

    let mut fields_types: StructFields = (name, Vec::with_capacity(10), modificator);
    let mut field_position: u32 = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.check(TokenType::Identifier) {
            let field_tk: &Token = ctx.consume(
                TokenType::Identifier,
                "Syntax error".into(),
                "Expected identifier.".into(),
            )?;

            let field_name: &str = field_tk.get_lexeme();
            let field_span: Span = field_tk.get_span();

            ctx.consume(
                TokenType::Colon,
                "Syntax error".into(),
                "Expected ':'.".into(),
            )?;

            let field_type: Type = typegen::build_type(ctx)?;

            fields_types
                .1
                .push((field_name, field_type, field_position, field_span));

            field_position += 1;

            if ctx.check(TokenType::RBrace) {
                break;
            } else if ctx.match_token(TokenType::Comma)? {
                if ctx.check(TokenType::RBrace) {
                    break;
                }
            } else if ctx.check_to(TokenType::Identifier, 0) {
                ctx.consume(
                    TokenType::Comma,
                    "Syntax error".into(),
                    "Expected ','.".into(),
                )?;
            } else {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected identifier.".into(),
                    None,
                    ctx.previous().get_span(),
                ));
            }
        } else {
            ctx.only_advance()?;

            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "Expected structure fields identifiers.".into(),
                None,
                ctx.previous().get_span(),
            ));
        }
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    if declare_forward {
        ctx.get_mut_symbols().new_struct(
            name,
            (name, fields_types.1, attributes, modificator),
            span,
        )?;

        return Ok(Ast::new_nullptr(span));
    }

    Ok(Ast::Struct {
        name,
        fields: fields_types.clone(),
        kind: fields_types.get_type(),
        attributes,
        span,
    })
}
