use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Const {
            kind: target_type,
            value,
            span,
            ..
        } => {
            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(value.is_literal(), *span);

            let from_type: &Type = value.get_value_type()?;
            let expression_span: Span = value.get_span();

            if !value.is_constant_value() {
                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected compile-time known value.".into(),
                    None,
                    expression_span,
                ));
            }

            checks::check_types(
                target_type,
                &Type::Const(from_type.clone().into()),
                Some(value),
                None,
                metadata,
            )?;

            typechecker.analyze_stmt(value)?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(ThrushCompilerIssue::FrontEndBug(
                "Expression not caught".into(),
                "Expression could not be caught for processing.".into(),
                span,
                CompilationPosition::TypeChecker,
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
