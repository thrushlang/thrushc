use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::traits::TypeFixedArrayEntensions;
use crate::front_end::typesystem::types::Type;

pub fn build_fixed_array<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    ctx.consume(
        TokenType::Fixed,
        "Syntax error".into(),
        "Expected 'fixed' keyword.".into(),
    )?;

    let array_start_tk: &Token = ctx.consume(
        TokenType::LBracket,
        "Syntax error".into(),
        "Expected '['.".into(),
    )?;

    let span: Span = array_start_tk.get_span();

    let mut array_type: Type = Type::Void;
    let mut items: Vec<Ast> = Vec::with_capacity(100);

    loop {
        if ctx.check(TokenType::RBracket) {
            break;
        }

        let item: Ast = expr::build_expr(ctx)?;

        items.push(item);

        if ctx.check(TokenType::RBracket) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RBracket,
        "Syntax error".into(),
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
            ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "The size limit of an array was exceeded.".into(),
                None,
                span,
            )
        })?;

        array_type = Type::FixedArray(item.get_value_type()?.clone().into(), size);
    }

    Ok(Ast::FixedArray {
        items,
        kind: array_type,
        span,
    })
}
