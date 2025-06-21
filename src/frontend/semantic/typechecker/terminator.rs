use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span, semantic::typechecker::TypeChecker,
        types::parser::stmts::stmt::ThrushStatement,
    },
};

pub fn validate_terminator<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::Return {
            expression,
            kind,
            span,
        } => {
            if let Some(expr) = expression {
                if let Err(error) = typechecker.validate_types(
                    kind,
                    expr.get_value_type()?,
                    Some(expr),
                    None,
                    None,
                    span,
                ) {
                    typechecker.add_error(error);
                }

                typechecker.analyze_stmt(expr)?;
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
