use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn validate_loop<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::For {
            local,
            cond,
            actions,
            block,
            ..
        } => {
            if let Err(error) = typechecker.analyze_stmt(local) {
                typechecker.add_error(error);
            }

            if let Err(error) = typechecker.analyze_stmt(cond) {
                typechecker.add_error(error);
            }

            if let Err(error) = typechecker.analyze_stmt(actions) {
                typechecker.add_error(error);
            }

            if let Err(error) = typechecker.analyze_stmt(block) {
                typechecker.add_error(error);
            }

            Ok(())
        }

        ThrushStatement::While { cond, block, .. } => {
            if let Err(error) = typechecker.validate_types(
                &ThrushType::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                None,
                &cond.get_span(),
            ) {
                typechecker.add_error(error);
            }

            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        ThrushStatement::Loop { block, .. } => {
            typechecker.analyze_stmt(block)?;

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
