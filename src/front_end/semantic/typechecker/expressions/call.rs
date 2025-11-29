use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::semantic::typechecker::{
    TypeChecker, checks, metadata::TypeCheckerExprMetadata,
};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstGetType, AstStandardExtensions};
use crate::front_end::typesystem::types::Type;

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    metadata: (&[Type], bool),
    args: &'type_checker [Ast],
    span: &Span,
) -> Result<(), CompilationIssue> {
    let (parameter_types, ignore_more_arguments) = metadata;

    let required_count: usize = parameter_types.len();
    let provided_count: usize = args.len();

    if required_count != provided_count && !ignore_more_arguments {
        typechecker.add_error(CompilationIssue::Error(
            "Type error".into(),
            format!(
                "Expected arguments total '{}', not '{}'. You should try to fill it in.",
                required_count, provided_count
            ),
            None,
            *span,
        ));

        let expected_types: String = parameter_types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        typechecker.add_error(CompilationIssue::Error(
            "Type error".into(),
            format!(
                "Arguments were expected in the order '{}'. You must reorder it.",
                expected_types
            ),
            None,
            *span,
        ));

        return Ok(());
    }

    for (target_type, expr) in parameter_types.iter().zip(args.iter()) {
        let from_type = expr.get_value_type()?;
        let expr_metadata = TypeCheckerExprMetadata::new(expr.is_literal_value(), expr.get_span());

        checks::check_types(target_type, from_type, Some(expr), None, expr_metadata)?;
    }

    for arg in args.iter() {
        typechecker.analyze_expr(arg)?;
    }

    Ok(())
}
