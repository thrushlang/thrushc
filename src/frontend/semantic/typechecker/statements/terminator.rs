use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
        types::ast::Ast,
    },
};

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
                    TypeCheckerExprMetadata::new(expr.is_literal(), None, span);

                if let Err(error) =
                    checks::type_check(kind, expr.get_value_type()?, Some(expr), None, metadata)
                {
                    typechecker.add_error(error);
                }

                typechecker.analyze_stmt(expr)?;
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
                line!(),
            ));

            Ok(())
        }
    }
}
