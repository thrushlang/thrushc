use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::{
    Type,
    traits::{TypeArrayEntensions, TypeIsExtensions},
};

use crate::{ParserContext, expressions};

pub fn build_array<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let tk: &Token = ctx.consume(
        TokenType::LBracket,
        CompilationIssueCode::E0001,
        "Expected '['.".into(),
    )?;

    let span: Span = tk.get_span();

    let mut array_type: Type = ctx
        .get_type_ctx()
        .get_infered_type()
        .unwrap_or(Type::Void(span))
        .get_array_base_type();

    if !array_type.is_array_type() {
        array_type = Type::Array(array_type.into(), span);
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
