use std::path::PathBuf;

use crate::{
    core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue},
    frontend::{
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
        Ast::If {
            condition,
            block,
            elseif,
            anyway,
            span,
        } => {
            typechecker.analyze_stmt(condition)?;

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(condition.is_literal(), *span);

            checks::check_types(
                &Type::Bool,
                condition.get_value_type()?,
                Some(condition),
                None,
                metadata,
            )?;

            elseif
                .iter()
                .try_for_each(|elif| typechecker.analyze_stmt(elif))?;

            if let Some(otherwise) = anyway {
                typechecker.analyze_stmt(otherwise)?;
            }

            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Elif {
            condition,
            block,
            span,
        } => {
            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(condition.is_literal(), *span);

            checks::check_types(
                &Type::Bool,
                condition.get_value_type()?,
                Some(condition),
                None,
                metadata,
            )?;

            typechecker.analyze_stmt(condition)?;
            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Else { block, .. } => {
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
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
