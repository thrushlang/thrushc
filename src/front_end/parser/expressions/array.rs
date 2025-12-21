use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::{token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::{traits::TypeArrayEntensions, types::Type};

pub fn build_array<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let tk: &Token = ctx.consume(
        TokenType::LBracket,
        CompilationIssueCode::E0001,
        "Expected '['.".into(),
    )?;

    let span: Span = tk.get_span();

    let mut array_type: Type = Type::Void(span);
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
                if item_type.get_array_type_herarchy() > current_type.get_array_type_herarchy() {
                    Some(item)
                } else {
                    Some(current)
                }
            }
        })
    })? {
        array_type = Type::Array(item.get_value_type()?.clone().into(), span);
    }

    Ok(Ast::Array {
        items,
        kind: array_type,
        span,
    })
}
