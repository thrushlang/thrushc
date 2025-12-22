use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::core::errors::standard::CompilationIssueCode;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::expressions;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::EnumFields;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;

pub fn build_enum<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Enum,
        CompilationIssueCode::E0001,
        "Expected 'enum'.".into(),
    )?;

    let name: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected enum name.".into(),
    )?;

    let enum_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let enum_attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
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
                CompilationIssueCode::E0001,
                "Expected ':'.".into(),
            )?;

            let field_type: Type = typegen::build_type(ctx, false)?;

            ctx.consume(
                TokenType::Eq,
                CompilationIssueCode::E0001,
                "Expected '='.".into(),
            )?;

            let expr: Ast = expressions::build_expr(ctx)?;

            ctx.consume(
                TokenType::SemiColon,
                CompilationIssueCode::E0001,
                String::from("Expected ';'."),
            )?;

            enum_fields.push((name, field_type, expr));

            continue;
        } else {
            ctx.consume(
                TokenType::Identifier,
                CompilationIssueCode::E0001,
                "Expected identifier.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
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
