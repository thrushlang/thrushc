use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::expr;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::{EnumFields, ThrushAttributes};
use crate::front_end::typesystem::types::Type;

pub fn build_enum<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Enum,
        "Syntax error".into(),
        "Expected 'enum'.".into(),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected enum name.".into(),
    )?;

    let enum_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let enum_attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut enum_fields: EnumFields = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        if ctx.match_token(TokenType::Identifier)? {
            let field_tk: &Token = ctx.previous();

            let name: &str = field_tk.get_lexeme();
            ctx.consume(
                TokenType::Colon,
                "Syntax error".into(),
                "Expected ':'.".into(),
            )?;

            let field_type: Type = typegen::build_type(ctx)?;

            ctx.consume(TokenType::Eq, "Syntax error".into(), "Expected '='.".into())?;

            let expr: Ast = expr::build_expr(ctx)?;

            ctx.consume(
                TokenType::SemiColon,
                String::from("Syntax error"),
                String::from("Expected ';'."),
            )?;

            enum_fields.push((name, field_type, expr));

            continue;
        }

        return Err(CompilationIssue::Error(
            "Syntax error".into(),
            "Expected identifier in enum field.".into(),
            None,
            ctx.advance()?.get_span(),
        ));
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    ctx.get_mut_symbols().new_enum(
        enum_name,
        (enum_fields.clone(), enum_attributes.clone()),
        span,
    )?;

    Ok(Ast::Enum {
        name: enum_name,
        fields: enum_fields,
        attributes: enum_attributes,
        span,
    })
}
