#![allow(clippy::type_complexity)]

use std::{fmt::Display, rc::Rc};

use inkwell::{
    AddressSpace,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{cast, context::LLVMCodeGenContext, memory, ptrgen},
    core::console::logging::{self, LoggingType},
    frontend::types::{ast::Ast, lexer::Type},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: &'ctx (Option<(&'ctx str, Rc<Ast<'ctx>>)>, Option<Rc<Ast<'ctx>>>),
    kind: &Type,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let mut value: BasicValueEnum = match value {
        (Some((name, _)), _) => {
            let ptr: PointerValue = context.get_symbol(name).get_ptr();

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

    if let Some(cast_type) = cast_type {
        if let Some(casted_value) = cast::try_cast(context, cast_type, kind, value) {
            value = casted_value;
        }
    }

    value
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
