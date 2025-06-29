use std::fmt::Display;

use super::context::LLVMCodeGenContext;
use super::typegen;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite};

use crate::backend::llvm::compiler::{constgen, valuegen};
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::ast::Ast;
use crate::frontend::types::lexer::ThrushType;

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::{builder::Builder, context::Context};

pub fn compile_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [Ast],
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    if items.iter().all(|item| item.is_constant_value()) {
        constgen::constant_fixed_array(context, cast_type.unwrap_or(kind), items)
    } else {
        self::fixed_array(context, kind, items, cast_type)
    }
}

fn fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [Ast],
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let array_type: &ThrushType = cast_type.unwrap_or(kind);
    let array_items_type: &ThrushType = array_type.get_fixed_array_base_type();

    let array_ptr: PointerValue =
        if let Some(preferred_site_allocation) = context.get_site_allocation() {
            memory::alloc_anon(preferred_site_allocation, context, array_type, true)
        } else {
            memory::alloc_anon(LLVMAllocationSite::Stack, context, array_type, true)
        };

    let array_ptr_type: BasicTypeEnum = typegen::generate_type(llvm_context, array_type);

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        match unsafe {
            llvm_builder.build_gep(
                array_ptr_type,
                array_ptr,
                &[llvm_context.i32_type().const_zero(), llvm_idx],
                "",
            )
        } {
            Ok(element_ptr) => {
                let value: BasicValueEnum =
                    valuegen::compile(context, item, Some(array_items_type));

                memory::store_anon(context, element_ptr, value);
            }
            Err(_) => {
                self::codegen_abort(format!(
                    "Failed to calculate memory address for array element at index {}",
                    idx
                ));

                return self::compile_null_ptr(context);
            }
        }
    }

    memory::load_anon(context, array_ptr, kind)
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
