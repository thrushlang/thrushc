use inkwell::{builder::Builder, context::Context, values::FloatValue};

use crate::{
    core::console::logging::{self, LoggingType},
    frontend::types::lexer::Type,
};

pub fn float<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    kind: &Type,
    number: f64,
    signed: bool,
) -> FloatValue<'ctx> {
    match kind {
        Type::F32 if signed => builder
            .build_float_neg(context.f32_type().const_float(number), "")
            .unwrap(),
        Type::F32 => context.f32_type().const_float(number),
        Type::F64 if signed => builder
            .build_float_neg(context.f64_type().const_float(number), "")
            .unwrap(),
        Type::F64 => context.f64_type().const_float(number),

        what => {
            logging::log(
                LoggingType::BackendBug,
                &format!("Unsupported integer type: '{:#?}'.", what),
            );

            unreachable!()
        }
    }
}
