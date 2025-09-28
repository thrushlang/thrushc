use std::fmt::Display;

use inkwell::{AddressSpace, values::BasicValueEnum};

use crate::{
    backends::classical::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self},
        ptr, value,
    },
    core::console::logging::{self, LoggingType},
    frontends::classical::{
        types::ast::Ast,
        typesystem::{traits::TypeExtensions, types::Type},
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx Ast,
) -> BasicValueEnum<'ctx> {
    match expr {
        Ast::Mut { source, value, .. } => {
            let cast: &Type = source.get_type_unwrapped().get_type_with_depth(1);

            let ptr: BasicValueEnum = ptr::compile(context, source, None);
            let value: BasicValueEnum = value::compile(context, value, Some(cast));

            memory::store_anon(context, ptr.into_pointer_value(), value);

            self::compile_null_ptr(context)
        }

        _ => {
            self::codegen_abort("A mutation cannot be executed.");
        }
    }
}

#[inline]
fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
