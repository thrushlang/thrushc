use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::span::Span,
        types::ast::metadata::cast::CastMetadata,
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn check_type_cast(
    cast_type: &Type,
    from_type: &Type,
    metadata: &CastMetadata,
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    let is_allocated: bool = metadata.is_allocated();

    let abort_cast = || {
        Err(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!("Cannot cast '{}' to '{}'.", from_type, cast_type),
            None,
            *span,
        ))
    };

    if from_type.is_integer_type() && cast_type.is_integer_type() {
        return Ok(());
    }

    if from_type.is_float_type() && cast_type.is_float_type() {
        return Ok(());
    }

    if from_type.is_str_type() && cast_type.is_ptr_type() {
        return Ok(());
    }

    if (from_type.is_str_type()
        || from_type.is_float_type()
        || from_type.is_integer_type()
        || from_type.is_struct_type()
        || from_type.is_array_type()
        || from_type.is_fixed_array_type())
        && is_allocated
        && cast_type.is_ptr_type()
    {
        return Ok(());
    }

    if from_type.is_mut_type() && cast_type.is_mut_type() {
        let lhs_type: &Type = from_type.get_type_with_depth(1);
        let rhs_type: &Type = cast_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    if from_type.is_mut_type() && cast_type.is_ptr_type() {
        let lhs_type: &Type = from_type.get_type_with_depth(1);
        let rhs_type: &Type = cast_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    if from_type.is_ptr_type() && cast_type.is_ptr_type() {
        let lhs_type: &Type = from_type.get_type_with_depth(1);
        let rhs_type: &Type = cast_type.get_type_with_depth(1);

        self::check_type_cast(lhs_type, rhs_type, metadata, span)?;

        return Ok(());
    }

    abort_cast()
}
