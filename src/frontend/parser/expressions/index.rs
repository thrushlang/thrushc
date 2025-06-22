#![allow(clippy::type_complexity)]

use std::rc::Rc;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expression, expressions::reference},
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn build_index<'instr>(
    parser_context: &mut ParserContext<'instr>,
    reference: Option<&'instr str>,
    expr: Option<ThrushStatement<'instr>>,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let index_to: (
        Option<(&str, Rc<ThrushStatement>)>,
        Option<Rc<ThrushStatement>>,
    ) = if let Some(name) = reference {
        let reference: ThrushStatement = reference::build_reference(parser_context, name, span)?;

        (Some((name, reference.into())), None)
    } else if let Some(expr) = expr {
        (None, Some(expr.into()))
    } else {
        let expr: ThrushStatement = expression::build_expr(parser_context)?;
        (None, Some(expr.into()))
    };

    let index_type: &ThrushType = match index_to {
        (Some(ref any_reference), None) => {
            let reference: &ThrushStatement = &any_reference.1;
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
            let reference: &ThrushStatement = &any_reference.1;
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

    let mut indexes: Vec<ThrushStatement> = Vec::with_capacity(50);

    loop {
        if parser_context.check(TokenType::RBracket) {
            break;
        }

        let indexe: ThrushStatement = expression::build_expr(parser_context)?;

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

    Ok(ThrushStatement::Index {
        index_to,
        indexes,
        kind: index_type,
        is_mutable,
        span,
    })
}
