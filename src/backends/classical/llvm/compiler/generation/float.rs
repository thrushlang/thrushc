use crate::core::console::logging;
use crate::core::console::logging::LoggingType;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::{context::Context, values::FloatValue};
use std::fmt::Display;

pub fn generate<'ctx>(
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
        Type::FX8680 if signed => context.x86_f80_type().const_float(-iee),
        Type::FX8680 => context.x86_f80_type().const_float(iee),

        what => {
            self::codegen_abort(format!("Unsupported float type: '{}'.", what));
        }
    }
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
