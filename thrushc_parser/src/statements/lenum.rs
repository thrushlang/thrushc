use thrushc_ast::{Ast, data::EnumData};
use thrushc_attributes::ThrushAttributes;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, attributes, expressions, typegen};

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
        "Expected identifier.".into(),
    )?;

    let enum_name: &str = name.get_lexeme();
    let span: Span = name.get_span();

    let enum_attributes: ThrushAttributes =
        attributes::build_compiler_attributes(ctx, &[TokenType::LBrace])?;

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let mut data: EnumData = Vec::with_capacity(10);

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

            data.push((name, field_type, expr));

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

    if !ctx.is_main_scope() {
        ctx.get_mut_symbols()
            .new_enum(enum_name, (data.clone(), enum_attributes.clone()), span)?;

        Ok(Ast::Enum {
            name: enum_name,
            data,
            attributes: enum_attributes,
            kind: Type::Void(span),
            span,
        })
    } else {
        Ok(Ast::invalid_ast(span))
    }
}
