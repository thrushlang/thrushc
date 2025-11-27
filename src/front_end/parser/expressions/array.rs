use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::{span::Span, token::Token, tokentype::TokenType};
use crate::front_end::parser::{ParserContext, expr};
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::{ast::Ast, parser::stmts::traits::TokenExtensions};
use crate::front_end::typesystem::{traits::TypeArrayEntensions, types::Type};

pub fn build_array<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
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
                if item_type.get_array_type_herarchy() > current_type.get_array_type_herarchy() {
                    Some(item)
                } else {
                    Some(current)
                }
            }
        })
    })? {
        array_type = Type::Array(item.get_value_type()?.clone().into());
    }

    Ok(Ast::Array {
        items,
        kind: array_type,
        span,
    })
}
