use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontends::classical::{
        lexer::span::Span,
        semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
        types::ast::Ast,
        typesystem::types::Type,
    },
};

pub fn validate<'type_checker>(
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

        Ast::While { cond, block, .. } => {
            let span: Span = cond.get_span();

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(cond.is_literal(), None, span);

            if let Err(error) = checks::type_check(
                &Type::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                metadata,
            ) {
                typechecker.add_error(error);
            }

            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Loop { block, .. } => {
            typechecker.analyze_stmt(block)?;

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
