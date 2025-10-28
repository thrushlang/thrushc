use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::frontend::lexer::span::Span;
use crate::frontend::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

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
            typechecker.analyze_stmt(local)?;
            typechecker.analyze_stmt(cond)?;
            typechecker.analyze_stmt(actions)?;
            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        Ast::While { cond, block, .. } => {
            let span: Span = cond.get_span();

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(cond.is_literal(), span);

            checks::check_types(
                &Type::Bool,
                cond.get_value_type()?,
                Some(cond),
                None,
                metadata,
            )?;

            typechecker.analyze_stmt(cond)?;
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
                PathBuf::from(file!()),
                line!(),
            ));

            Ok(())
        }
    }
}
