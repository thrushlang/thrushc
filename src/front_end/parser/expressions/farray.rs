use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::traits::{TypeFixedArrayEntensions, TypeIsExtensions};
use crate::front_end::typesystem::types::Type;

pub fn build_fixed_array<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Fixed,
        CompilationIssueCode::E0001,
        "Expected 'fixed' keyword.".into(),
    )?;

    let array_start_tk: &Token = ctx.consume(
        TokenType::LBracket,
        CompilationIssueCode::E0001,
        "Expected '['.".into(),
    )?;

    let span: Span = array_start_tk.get_span();

    let mut array_type: Type = ctx
        .get_type_ctx()
        .get_infered_type()
        .unwrap_or(Type::Void(span))
        .get_fixed_array_base_type();

    if !array_type.is_fixed_array_type() {
        array_type = Type::FixedArray(array_type.into(), 0, span);
    }

    let mut items: Vec<Ast> = Vec::with_capacity(100);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        let item: Ast = expressions::build_expr(ctx)?;

        items.push(item);

        if ctx.check(TokenType::RBracket) {
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
        TokenType::RBracket,
        CompilationIssueCode::E0001,
        "Expected ']'.".into(),
    )?;

    if let Some(item) = items.iter().try_fold(None::<&Ast>, |acc, item| {
        let item_type: &Type = item.get_value_type()?;

        Ok(match acc {
            None => Some(item),
            Some(current) => {
                let current_type: &Type = current.get_value_type()?;
                if item_type.get_fixed_array_type_herarchy()
                    > current_type.get_fixed_array_type_herarchy()
                {
                    Some(item)
                } else {
                    Some(current)
                }
            }
        })
    })? {
        let size: u32 = u32::try_from(items.len()).map_err(|_| {
            CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "The size limit of an array was exceeded.".into(),
                None,
                span,
            )
        })?;

        array_type = Type::FixedArray(item.get_value_type()?.clone().into(), size, span);
    }

    Ok(Ast::FixedArray {
        items,
        kind: array_type,
        span,
    })
}
