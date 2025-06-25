use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{ast::Ast, lexer::ThrushType},
    },
};

pub fn validate_constant<'type_checker>(
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
            let from_type: &ThrushType = value.get_value_type()?;

            if let Err(mismatch_type_error) =
                typechecker.validate_types(target_type, from_type, Some(value), None, None, span)
            {
                typechecker.add_error(mismatch_type_error);
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
