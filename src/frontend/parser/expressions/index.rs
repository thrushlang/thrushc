use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expression, expressions::reference},
        types::{
            ast::{Ast, types::AstEitherExpression},
            lexer::ThrushType,
        },
    },
};

pub fn build_index<'parser>(
    parser_context: &mut ParserContext<'parser>,
    reference: Option<&'parser str>,
    expr: Option<Ast<'parser>>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let index_to: AstEitherExpression = if let Some(name) = reference {
        let reference: Ast = reference::build_reference(parser_context, name, span)?;

        (Some((name, reference.into())), None)
    } else if let Some(expr) = expr {
        (None, Some(expr.into()))
    } else {
        let expr: Ast = expression::build_expr(parser_context)?;
        (None, Some(expr.into()))
    };

    let index_type: &ThrushType = match index_to {
        (Some(ref any_reference), None) => {
            let reference: &Ast = &any_reference.1;
            reference.get_value_type()?
        }
        (None, Some(ref expr)) => expr.get_value_type()?,
        _ => {
            return Err(ThrushCompilerIssue::Bug(
                String::from("Index not caught"),
                String::from("Expected a expression or reference."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }
    };

    let is_mutable: bool = match index_to {
        (Some(ref any_reference), None) => {
            let reference: &Ast = &any_reference.1;
            reference.is_mutable()
        }
        (None, Some(ref expr)) => expr.is_mutable(),
        _ => {
            return Err(ThrushCompilerIssue::Bug(
                String::from("Index not caught"),
                String::from("Expected a expression or reference."),
                span,
                CompilationPosition::Parser,
                line!(),
            ));
        }
    };

    let mut indexes: Vec<Ast> = Vec::with_capacity(50);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let indexe: Ast = expression::build_expr(parser_context)?;

        indexes.push(indexe);

        if parser_context.check(TokenType::RBracket) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RBracket,
        String::from("Syntax error"),
        String::from("Expected ']'."),
    )?;

    let index_type: ThrushType = if index_type.is_ptr_type() {
        ThrushType::Ptr(Some(
            index_type.get_type_with_depth(indexes.len()).clone().into(),
        ))
    } else {
        ThrushType::Mut(index_type.get_type_with_depth(indexes.len()).clone().into())
    };

    Ok(Ast::Index {
        index_to,
        indexes,
        kind: index_type,
        is_mutable,
        span,
    })
}
