use std::path::PathBuf;

use crate::core::errors::{position::CompilationPosition, standard::ThrushCompilerIssue};

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
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
            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(value.is_literal_value(), *span);

            let from_type: &Type = value.get_value_type()?;

            checks::check_types(
                target_type,
                &Type::Const(from_type.clone().into()),
                Some(value),
                None,
                metadata,
            )?;

            typechecker.analyze_expr(value)?;

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
