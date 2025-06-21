use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn validate_conditional<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker ThrushStatement,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        ThrushStatement::If {
            cond,
            block,
            elfs,
            otherwise,
            span,
        } => {
            if let Err(error) = typechecker.validate_types(
                &ThrushType::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                None,
                span,
            ) {
                typechecker.add_error(error);
            }

            elfs.iter()
                .try_for_each(|elif| typechecker.analyze_stmt(elif))?;

            if let Some(otherwise) = otherwise {
                typechecker.analyze_stmt(otherwise)?;
            }

            typechecker.analyze_stmt(cond)?;
            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        ThrushStatement::Elif { cond, block, span } => {
            if let Err(error) = typechecker.validate_types(
                &ThrushType::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                None,
                span,
            ) {
                typechecker.add_error(error);
            }

            typechecker.analyze_stmt(cond)?;
            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        ThrushStatement::Else { block, .. } => {
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
