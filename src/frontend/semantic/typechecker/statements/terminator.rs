use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::frontend::types::ast::Ast;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Return {
            expression, kind, ..
        } => {
            if let Some(expr) = expression {
                let span: Span = expr.get_span();

                let metadata: TypeCheckerExprMetadata =
                    TypeCheckerExprMetadata::new(expr.is_literal(), span);

                checks::check_types(kind, expr.get_value_type()?, Some(expr), None, metadata)?;

                typechecker.analyze_expr(expr)?;
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
