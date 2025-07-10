use std::fmt::Display;

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{cast, context::LLVMCodeGenContext, memory, ptrgen},
    core::console::logging::{self, LoggingType},
    frontend::{types::ast::types::AstEitherExpression, typesystem::types::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx AstEitherExpression<'ctx>,
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let value: BasicValueEnum = match source {
        (Some((name, _)), _) => {
            let ptr: PointerValue = context.get_table().get_symbol(name).get_ptr();

            memory::load_anon(context, ptr, kind)
        }
        (_, Some(expr)) => {
            let ptr: PointerValue = ptrgen::compile(context, expr, None).into_pointer_value();

            memory::load_anon(context, ptr, kind)
        }
        _ => {
            self::codegen_abort("Invalid load target in expression");
            self::compile_null_ptr(context)
        }
    };

    cast::try_cast(context, cast, kind, value).unwrap_or(value)
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(LoggingType::BackendBug, &format!("{}", message));
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
