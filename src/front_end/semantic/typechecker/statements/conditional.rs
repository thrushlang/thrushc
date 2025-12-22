use crate::core::diagnostic::span::Span;
use crate::core::errors::position::CompilationPosition;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::TypeChecker;
use crate::front_end::semantic::typechecker::checks;
use crate::front_end::semantic::typechecker::metadata::TypeCheckerExprMetadata;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeLocation;
use crate::front_end::types::ast::traits::AstGetType;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::If {
            condition,
            block,
            elseif,
            anyway,
            ..
        } => {
            typechecker.analyze_expr(condition)?;

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(condition.is_literal_value());

            let span: Span = condition.get_span();

            checks::check_types(
                &Type::Bool(span),
                condition.get_value_type()?,
                Some(condition),
                None,
                metadata,
                span,
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
            condition, block, ..
        } => {
            typechecker.analyze_expr(condition)?;

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(condition.is_literal_value());

            let span: Span = condition.get_span();

            checks::check_types(
                &Type::Bool(condition.get_span()),
                condition.get_value_type()?,
                Some(condition),
                None,
                metadata,
                span,
            )?;

            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        Ast::Else { block, .. } => {
            typechecker.analyze_stmt(block)?;

            Ok(())
        }

        _ => {
            let span: Span = node.get_span();

            typechecker.add_bug(CompilationIssue::FrontEndBug(
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
