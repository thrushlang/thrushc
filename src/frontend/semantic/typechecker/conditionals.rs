use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, bounds},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn validate_conditional<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::If {
            cond,
            block,
            elfs,
            otherwise,
            span,
        } => {
            if let Err(error) = bounds::checking::check(
                &Type::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                None,
                span,
            ) {
                typechecker.add_error(error);
            }

            elfs.iter()
                .try_for_each(|elif| typechecker.analyze_ast(elif))?;

            if let Some(otherwise) = otherwise {
                typechecker.analyze_ast(otherwise)?;
            }

            typechecker.analyze_ast(cond)?;
            typechecker.analyze_ast(block)?;

            Ok(())
        }

        Ast::Elif { cond, block, span } => {
            if let Err(error) = bounds::checking::check(
                &Type::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                None,
                span,
            ) {
                typechecker.add_error(error);
            }

            typechecker.analyze_ast(cond)?;
            typechecker.analyze_ast(block)?;

            Ok(())
        }

        Ast::Else { block, .. } => {
            typechecker.analyze_ast(block)?;

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
