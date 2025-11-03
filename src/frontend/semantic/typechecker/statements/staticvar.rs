use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Static {
            kind: static_type,
            value,
            span,
            ..
        } => {
            if let Some(value) = value {
                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(value.is_literal(), *span);

                let value_type: &Type = value.get_value_type()?;
                let value_span: Span = value.get_span();

                if !value.is_constant_value() {
                    return Err(ThrushCompilerIssue::Error(
                        "Syntax error".into(),
                        "Expected compile-time sized value.".into(),
                        None,
                        value_span,
                    ));
                }

                checks::check_types(static_type, value_type, Some(value), None, metadata)?;

                typechecker.analyze_expr(value)?;
            }

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
