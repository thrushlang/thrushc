use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstStandardExtensions;
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    metadata: (&[Type], bool),
    args: &'type_checker [Ast],
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let (parameter_types, ignore_more_arguments) = metadata;

    let required_size: usize = parameter_types.len();
    let provided_size: usize = args.len();

    if required_size != provided_size && !ignore_more_arguments {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!(
                "Expected arguments total '{}', not '{}'. You should try to fill it in.",
                required_size, provided_size
            ),
            None,
            *span,
        ));

        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!(
                "Arguments were expected in the order '{}'. You must reorder it.",
                parameter_types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
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

            let metadata: TypeCheckerExprMetadata =
                TypeCheckerExprMetadata::new(expr.is_literal_value(), expr.get_span());

            checks::check_types(target_type, from_type, Some(expr), None, metadata)?;

            Ok(())
        })?;

    args.iter()
        .try_for_each(|arg| typechecker.analyze_expr(arg))?;

    Ok(())
}
