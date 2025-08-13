use std::fmt::Display;

use inkwell::{context::Context, values::FloatValue};

use crate::{
    core::console::logging::{self, LoggingType},
    frontend::typesystem::types::Type,
};

pub fn const_float<'ctx>(
    context: &'ctx Context,
    kind: &Type,
    iee: f64,
    signed: bool,
) -> FloatValue<'ctx> {
    match kind {
        Type::F32 if signed => context.f32_type().const_float(-iee),
        Type::F32 => context.f32_type().const_float(iee),
        Type::F64 if signed => context.f64_type().const_float(-iee),
        Type::F64 => context.f64_type().const_float(iee),

        what => {
            self::codegen_abort(format!("Unsupported integer type: '{:#?}'.", what));
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
