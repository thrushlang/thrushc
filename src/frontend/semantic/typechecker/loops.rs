use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, bounds},
        types::{ast::Ast, lexer::Type},
    },
};

pub fn validate_loop<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), ThrushCompilerIssue> {
    match node {
        Ast::For {
            local,
            cond,
            actions,
            block,
            ..
        } => {
            if let Err(error) = typechecker.analyze_ast(local) {
                typechecker.add_error(error);
            }

            if let Err(error) = typechecker.analyze_ast(cond) {
                typechecker.add_error(error);
            }

            if let Err(error) = typechecker.analyze_ast(actions) {
                typechecker.add_error(error);
            }

            if let Err(error) = typechecker.analyze_ast(block) {
                typechecker.add_error(error);
            }

            Ok(())
        }

        Ast::While { cond, block, .. } => {
            if let Err(error) = bounds::checking::check(
                &Type::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                None,
                &cond.get_span(),
            ) {
                typechecker.add_error(error);
            }

            typechecker.analyze_ast(block)?;

            Ok(())
        }

        Ast::Loop { block, .. } => {
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
