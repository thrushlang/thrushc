use {
    crate::{
        core::errors::standard::ThrushCompilerIssue,
        frontends::classical::{
            lexer::span::Span,
            semantic::typechecker::{TypeChecker, checks, metadata::TypeCheckerExprMetadata},
            types::ast::Ast,
            typesystem::types::Type,
        },
    },
    std::fmt::Write,
};

pub fn validate<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    metadata: (&[Type], bool),
    args: &'type_checker [Ast],
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let (parameter_types, ignore_more_arguments) = metadata;

    let required_size: usize = parameter_types.len();
    let provided_size: usize = args.len();

    let mut types_display: String = String::with_capacity(100);

    parameter_types.iter().for_each(|parameter_type| {
        let _ = write!(types_display, "{}", parameter_type);
    });

    if required_size != provided_size && !ignore_more_arguments {
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

    args.iter()
        .try_for_each(|arg| typechecker.analyze_stmt(arg))?;

    Ok(())
}
