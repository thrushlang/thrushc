use std::fmt::Display;

use inkwell::{
    builder::Builder,
    context::Context,
    types::BasicTypeEnum,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, SymbolAllocated},
        typegen,
    },
    core::console::logging::{self, LoggingType},
    frontend::{types::ast::types::AstEitherExpression, typesystem::types::Type},
};

pub fn compile_property_value<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx AstEitherExpression<'ctx>,
    indexes: &[(Type, u32)],
    kind: &Type,
) -> BasicValueEnum<'ctx> {
    match source {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_table().get_symbol(name);

            if symbol.is_pointer() {
                self::compile_pointer_property(context, symbol, indexes)
            } else {
                self::compile_extract_value_property(context, symbol, indexes)
            }
        }
        (None, Some(expr)) => {
            todo!()
        }
        _ => {
            self::codegen_abort("Unable to get a value of an structure at memory manipulation.");
        }
    }
}

fn compile_pointer_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    symbol: SymbolAllocated<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let mut ptr: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, indexes[0].1);
    let mut ptr_type: &Type = &indexes[0].0;

    for index in indexes.iter().skip(1) {
        let index_type: BasicTypeEnum = typegen::generate_type(llvm_context, &index.0);

        if let Ok(new_ptr) = llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
            ptr = new_ptr;
            ptr_type = &index.0;
        }
    }

    memory::load_anon(context, ptr, ptr_type)
}

fn compile_extract_value_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    symbol: SymbolAllocated<'ctx>,
    indexes: &[(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let mut value: BasicValueEnum = symbol.extract_value(llvm_builder, indexes[0].1);

    for index in indexes.iter().skip(1) {
        if value.is_struct_value() {
            if let Ok(new_value) =
                llvm_builder.build_extract_value(value.into_struct_value(), index.1, "")
            {
                value = new_value;
            }
        }
    }

    value
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
