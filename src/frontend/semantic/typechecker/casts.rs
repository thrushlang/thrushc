use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, bounds},
        types::{ast::Ast, lexer::Type},
    },
};

pub fn validate_cast_as<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::As {
            from,
            cast: cast_type,
            span,
            ..
        } => {
            let from_type: &Type = from.get_value_type()?;

            if let Err(error) = bounds::cast::check_type_cast(
                cast_type,
                from_type,
                from.is_allocated_reference(),
                span,
            ) {
                typechecker.add_error(error);
            }

            typechecker.analyze_ast(from)?;

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
