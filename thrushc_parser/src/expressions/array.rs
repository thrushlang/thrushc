use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, tokentype::TokenType, traits::TokenExtensions};
use thrushc_typesystem::{Type, traits::TypeArrayEntensions};

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

    if let Some(item) = items.iter().try_fold(None::<&Ast>, |current, item| {
        let current_type: &Type = item.get_value_type()?;

        Ok(match current {
            None => Some(item),
            Some(old_item) => {
                let old_type: &Type = old_item.get_value_type()?;

                if current_type.get_array_type_herarchy() > old_type.get_array_type_herarchy() {
                    Some(item)
                } else {
                    Some(old_item)
                }
            }
        })
    })? {
        let base_type: Type = item.get_value_type()?.clone();

        let size: Result<u32, std::num::TryFromIntError> = u32::try_from(items.len());

        if size.is_err() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected a size in unsigned 32-bit integer bounds. This exceeds the boundaries."
                    .into(),
                None,
                span,
            ));
        }

        let infered_type: Type =
            Type::FixedArray(base_type.clone().into(), size.unwrap_or_default(), span);

        array_type = Type::Array {
            base_type: base_type.into(),
            infered_type: Some((infered_type.into(), 0)),
            span,
        };
    }

    Ok(Ast::Array {
        items,
        kind: array_type,
        span,
    })
}
