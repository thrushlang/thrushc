use thrushc_ast::{
    Ast,
    traits::{AstCodeLocation, AstGetType, AstStandardExtensions},
};
use thrushc_attributes::traits::ThrushAttributesExtensions;
use thrushc_entities::typechecker::TypeCheckerFunction;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_typesystem::{Type, traits::VoidTypeExtensions};

use crate::{TypeChecker, checking, metadata::TypeCheckerExpressionMetadata};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    metadata: TypeCheckerFunction<'type_checker>,
    args: &'type_checker [Ast],
    span: &Span,
) -> Result<(), CompilationIssue> {
    let (return_type, parameter_types, attributes) = metadata;

    let required_count: usize = parameter_types.len();
    let provided_count: usize = args.len();

    let var_args: bool = attributes.has_ignore_attribute();

    if return_type.contains_void_type() {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
            None,
            *span,
        ));
    }

    if parameter_types.iter().any(|ty| ty.contains_void_type()) {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0019,
            "The void type is not a value. It cannot contain a value. The type it represents contains it. Remove it.".into(),
            None,
            *span,
        ));
    }

    if required_count != provided_count && !var_args {
        typechecker.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0022,
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
            CompilationIssueCode::E0023,
            format!(
                "Arguments were expected in the order '{}'. You must reorder it.",
                expected_types
            ),
            None,
            *span,
        ));

        return Ok(());
    }

    {
        for (target_type, expr) in parameter_types.iter().zip(args.iter()) {
            let from_type: &Type = expr.get_value_type()?;
            let expr_metadata: TypeCheckerExpressionMetadata =
                TypeCheckerExpressionMetadata::new(expr.is_literal_value());

            checking::check_types(
                target_type,
                from_type,
                Some(expr),
                None,
                expr_metadata,
                expr.get_span(),
            )?;
        }
    }

    {
        for arg in args.iter() {
            typechecker.analyze_expr(arg)?;
        }
    }

    Ok(())
}
