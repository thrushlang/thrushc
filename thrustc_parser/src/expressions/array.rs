use thrustc_ast::{Ast, NodeId, traits::AstGetType};
use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;
use thrustc_typesystem::{
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

    let infered_type: Option<Type> = ctx.get_type_ctx().get_infered_type();
    let mut array_type: Type = Type::Void(span);

    let mut items: Vec<Ast> = Vec::with_capacity(u8::MAX as usize);

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
        let item_type: &Type = item.get_value_type()?;

        Ok(match current {
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
        let size: Result<u32, std::num::TryFromIntError> = u32::try_from(items.len());

        if size.is_err() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                format!(
                    "Array size is out of bounds, it is superior to '{}'.'",
                    u32::MAX
                ),
                None,
                span,
            ));
        }

        let base_type: Type = item.get_value_type()?.clone();

        let fixed_type: Type =
            Type::FixedArray(base_type.clone().into(), size.unwrap_or_default(), span);

        array_type = Type::Array {
            base_type: base_type.into(),
            infered_type: Some((fixed_type.into(), 0)),
            span,
        };
    }

    if items.is_empty()
        && array_type.is_void_type()
        && infered_type.as_ref().is_some_and(|ty| ty.is_array_type())
    {
        if let Some(infered_type) = infered_type {
            let base_type: Type = infered_type.get_array_skipping_array_as_base_type();
            let fixed_type: Type = Type::FixedArray(base_type.clone().into(), 0, span);

            array_type = Type::Array {
                base_type: base_type.into(),
                infered_type: Some((fixed_type.into(), 0)),
                span,
            }
        }
    }

    Ok(Ast::Array {
        items,
        kind: array_type,
        span,
        id: NodeId::new(),
    })
}
