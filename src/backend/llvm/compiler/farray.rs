use std::fmt::Display;

use super::context::LLVMCodeGenContext;
use super::typegen;
use crate::backend::llvm::compiler::anchors::PointerAnchor;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite};

use crate::backend::llvm::compiler::valuegen;
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
    self::fixed_array(context, kind, items, cast_type)
}

fn fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [Ast],
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_fixed_array_with_anchor(context, kind, items, cast_type, anchor)
        } else {
            self::compile_fixed_array_without_anchor(context, kind, items, cast_type)
        }
    } else {
        self::compile_fixed_array_without_anchor(context, kind, items, cast_type)
    }
}

fn compile_fixed_array_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [Ast],
    cast_type: Option<&ThrushType>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let array_ptr: PointerValue = anchor.get_pointer();
    let array_type: &ThrushType = cast_type.unwrap_or(kind);

    context.set_pointer_anchor(PointerAnchor::new(array_ptr, true));

    let array_items_type: &ThrushType = array_type.get_fixed_array_base_type();

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| valuegen::compile(context, item, Some(array_items_type)))
        .collect();

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let array_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, array_type);

        match unsafe {
            llvm_builder.build_gep(
                array_type,
                array_ptr,
                &[llvm_context.i32_type().const_zero(), llvm_idx],
                "",
            )
        } {
            Ok(ptr) => {
                memory::store_anon(context, ptr, *item);
            }
            Err(_) => {
                self::codegen_abort(format!(
                    "Failed to calculate memory address for array element at index '{}'.",
                    idx
                ));

                return self::compile_null_ptr(context);
            }
        }
    }

    self::compile_null_ptr(context)
}

fn compile_fixed_array_without_anchor<'ctx>(
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
        memory::alloc_anon(LLVMAllocationSite::Stack, context, array_type, true);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| valuegen::compile(context, item, Some(array_items_type)))
        .collect();

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let array_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, array_type);

        match unsafe {
            llvm_builder.build_gep(
                array_type,
                array_ptr,
                &[llvm_context.i32_type().const_zero(), llvm_idx],
                "",
            )
        } {
            Ok(ptr) => {
                memory::store_anon(context, ptr, *item);
            }
            Err(_) => {
                self::codegen_abort(format!(
                    "Failed to calculate memory address for array element at index '{}'.",
                    idx
                ));

                return self::compile_null_ptr(context);
            }
        }
    }

    memory::load_anon(context, array_ptr, array_type)
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
