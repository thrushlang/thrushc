use crate::{
    core::console::logging::{self, LoggingType},
    frontend::types::{
        lexer::ThrushType,
        representations::FunctionCall,
        symbols::types::{Functions, ParametersTypes},
    },
};

use super::{ThrushStatement, cast, context::LLVMCodeGenContext, memory::SymbolAllocated};

use inkwell::{
    FloatPredicate,
    builder::Builder,
    context::Context,
    values::{BasicValueEnum, IntValue},
};

pub fn include(functions: &mut Functions) {
    functions.insert(
        "is_signed!",
        (
            ThrushType::Bool,
            ParametersTypes::new(Vec::from([ThrushType::Ptr(None)])),
            false,
        ),
    );
}

pub fn build_is_signed<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    call: FunctionCall<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let value: &ThrushStatement = &call.2[0];

    if let ThrushStatement::Reference {
        name,
        kind: ref_type,
        span,
        ..
    } = value
    {
        if !ref_type.is_float_type() && !ref_type.is_integer_type() {
            logging::log(
                LoggingType::Panic,
                &format!(
                    "Builtin 'is_signed!' cannot get the signedness of `{}` at {}.",
                    name, span
                ),
            );
        }

        let object: SymbolAllocated = context.get_allocated_symbol(name);

        return if ref_type.is_integer_type() {
            let mut loaded_value: IntValue = object.load(context).into_int_value();

            if let Some(casted_float) =
                cast::integer(context, &ThrushType::S64, ref_type, loaded_value.into())
            {
                loaded_value = casted_float.into_int_value();
            }

            llvm_builder
                .build_int_compare(
                    inkwell::IntPredicate::SLT,
                    loaded_value,
                    llvm_context.i64_type().const_int(0, false),
                    "",
                )
                .unwrap()
                .into()
        } else {
            let mut loaded_value: BasicValueEnum = object.load(context);

            if let Some(casted_float) =
                cast::float(context, &ThrushType::F64, ref_type, loaded_value)
            {
                loaded_value = casted_float;
            }

            llvm_builder
                .build_float_compare(
                    FloatPredicate::OLT,
                    loaded_value.into_float_value(),
                    llvm_context.f64_type().const_float(0.0),
                    "",
                )
                .unwrap()
                .into()
        };
    }

    llvm_context.bool_type().const_zero().into()
}
