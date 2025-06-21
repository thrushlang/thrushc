use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::span::Span,
        semantic::typechecker::TypeChecker,
        types::{lexer::ThrushType, parser::stmts::stmt::ThrushStatement},
    },
};

pub fn validate_call<'type_checker>(
    typechecker: &mut TypeChecker<'type_checker>,
    metadata: (&[ThrushType], bool),
    args: &'type_checker [ThrushStatement],
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let (parameter_types, ignore_more_arguments) = metadata;

    let parameter_types_size: usize = parameter_types.len();
    let mut parameter_types_displayed: String = String::with_capacity(100);

    parameter_types.iter().for_each(|parameter_type| {
        parameter_types_displayed.push_str(&format!("{} ", parameter_type));
    });

    if args.len() != parameter_types_size && !ignore_more_arguments {
        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!(
                "Expected {} arguments not {}.",
                parameter_types_size,
                args.len()
            ),
            None,
            *span,
        ));

        typechecker.add_error(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!(
                "Expected arguments in order to '{}'.",
                parameter_types_displayed
            ),
            None,
            *span,
        ));

        return Ok(());
    }

    for (target_type, expr) in parameter_types.iter().zip(args.iter()) {
        let from_type: &ThrushType = expr.get_value_type()?;
        let expr_span: Span = expr.get_span();

        if let Err(error) =
            typechecker.validate_types(target_type, from_type, Some(expr), None, None, &expr_span)
        {
            typechecker.add_error(error);
        }
    }

    args.iter()
        .try_for_each(|arg| typechecker.analyze_stmt(arg))?;

    Ok(())
}
