use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::span::Span,
        types::lexer::{Type, traits::TypeMutableExtensions},
    },
};

pub fn check_type_cast(
    cast_type: &Type,
    from_type: &Type,
    is_ref: bool,
    span: &Span,
) -> Result<(), ThrushCompilerIssue> {
    if (from_type.is_integer_type() && cast_type.is_integer_type() || cast_type.is_char_type())
        || (from_type.is_float_type() && cast_type.is_float_type())
        || (from_type.is_mut_numeric_type()
            || from_type.is_array_type()
            || from_type.is_fixed_array_type()
            || from_type.is_struct_type() && is_ref && cast_type.is_ptr_type())
        || (from_type.is_mut_type() || from_type.is_ptr_type() && cast_type.is_ptr_type())
        || (from_type.is_str_type() && cast_type.is_ptr_type())
        || (from_type.is_ptr_type() || cast_type.is_mut_type())
    {
        Ok(())
    } else {
        Err(ThrushCompilerIssue::Error(
            "Type error".into(),
            format!("Cannot cast '{}' to '{}'", from_type, cast_type),
            None,
            *span,
        ))
    }
}
