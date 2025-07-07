use std::fmt::Display;

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self},
        ptrgen, valuegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::types::ast::Ast,
    frontend::typesystem::types::Type,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Mut { source, value, .. } => {
            let cast_type: &Type = expr.get_type_unwrapped();

            let ptr: BasicValueEnum = ptrgen::compile(context, source, None);
            let value: BasicValueEnum = valuegen::compile(context, value, Some(cast_type));

            memory::store_anon(context, ptr.into_pointer_value(), value);

            self::compile_null_ptr(context)
        }

        _ => {
            self::codegen_abort("A mutation cannot be executed.");
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
