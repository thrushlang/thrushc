use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, bounds},
        types::ast::Ast,
    },
};

pub fn validate_terminator<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::Return {
            expression,
            kind,
            span,
        } => {
            if let Some(expr) = expression {
                if let Err(error) = bounds::checking::check(
                    kind,
                    expr.get_value_type()?,
                    Some(expr),
                    None,
                    None,
                    span,
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_ast(expr)?;
            }

            Ok(())
        }
        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(ThrushCompilerIssue::Bug(
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
