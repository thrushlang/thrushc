use std::fmt::Write;
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
        Ast::Indirect {
            function,
            function_type,
            args,
            span,
            ..
        } => {
            let function_ref: &Type = function.get_value_type()?;

            if !function_ref.is_fnref_type() {
                typechecker.add_error(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Expected function reference 'Fn[..] -> T' type.".into(),
                    None,
                    *span,
                ));
            }

            if let Type::Fn(parameter_types, ..) = function_type {
                let required_size: usize = parameter_types.len();
                let provided_size: usize = args.len();

                let mut types_display: String = String::with_capacity(100);

                parameter_types.iter().for_each(|parameter_type| {
                    let _ = write!(types_display, "{}", parameter_type);
                });

                if required_size != provided_size {
                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        format!(
                            "Expected arguments total '{}', not '{}'.",
                            required_size, provided_size
                        ),
                        None,
                        *span,
                    ));

                    typechecker.add_error(ThrushCompilerIssue::Error(
                        "Type error".into(),
                        format!("Arguments were expected in the order: '{}'.", types_display),
                        None,
                        *span,
                    ));

                    return Ok(());
                }

                parameter_types
                    .iter()
                    .zip(args.iter())
                    .try_for_each(|(target_type, expr)| {
                        let from_type: &Type = expr.get_value_type()?;
                        let span: Span = expr.get_span();

                        let metadata: TypeCheckerExprMetadata =
                            TypeCheckerExprMetadata::new(expr.is_literal(), span);

                        checks::check_types(target_type, from_type, Some(expr), None, metadata)?;

                        Ok(())
                    })?;
            }

            args.iter()
                .try_for_each(|arg| typechecker.analyze_expr(arg))?;

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
