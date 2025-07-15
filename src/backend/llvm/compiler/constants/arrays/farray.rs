use std::fmt::Display;

use inkwell::{AddressSpace, context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{
        constants::casts,
        constgen::{self},
        context::LLVMCodeGenContext,
        typegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::{types::ast::Ast, typesystem::types::Type},
};

pub fn constant_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    kind: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let item_type: &Type = kind.get_fixed_array_base_type();
    let array_type: BasicTypeEnum = typegen::generate_type(llvm_context, item_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| {
            let value_type: &Type = item.get_type_unwrapped();
            let value: BasicValueEnum = constgen::compile(context, item, item_type);

            casts::try_one(context, value, value_type, item_type)
        })
        .collect();

    match array_type {
        t if t.is_int_type() => t
            .into_int_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_int_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_float_type() => t
            .into_float_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_float_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_array_type() => t
            .into_array_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_array_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_struct_type() => t
            .into_struct_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_struct_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_pointer_type() => t
            .into_pointer_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_pointer_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        _ => {
            self::codegen_abort(format!(
                "Incompatible type '{}' for constant array.",
                item_type
            ));

            self::compile_null_ptr(context)
        }
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}
