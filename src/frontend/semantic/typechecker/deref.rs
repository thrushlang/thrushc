use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn validate_dereference<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::Deref { value, .. } => {
            let value_type: &ThrushType = value.get_value_type()?;
            let value_span: Span = value.get_span();

            if !value_type.is_ptr_type() && !value_type.is_mut_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected 'ptr<T>', 'ptr', or 'mut T' type for dereference.".into(),
                    None,
                    value_span,
                ));
            }

            typechecker.analyze_stmt(value)?;

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
