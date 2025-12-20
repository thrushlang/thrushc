use std::path::PathBuf;

use crate::core::diagnostic::span::Span;
use crate::core::errors::{position::CompilationPosition, standard::CompilationIssue};

use crate::front_end::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstGetType, AstStandardExtensions};
use crate::front_end::typesystem::traits::TypeCodeLocation;
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    node: &'type_checker Ast,
) -> Result<(), CompilationIssue> {
    match node {
        Ast::Const {
            kind: target_type,
            value,
            ..
        } => {
            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(value.is_literal_value(), value.get_span());

            let from_type: &Type = value.get_value_type()?;

            checks::check_types(
                target_type,
                &Type::Const(from_type.clone().into(), from_type.get_span()),
                Some(value),
                None,
                metadata,
            )?;

            typechecker.analyze_expr(value)?;

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
