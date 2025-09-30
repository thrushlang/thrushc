use std::fmt::Display;

use crate::backends::classical::llvm::compiler::anchors::PointerAnchor;
use crate::backends::classical::llvm::compiler::codegen;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::memory;
use crate::backends::classical::llvm::compiler::memory::LLVMAllocationSite;
use crate::backends::classical::llvm::compiler::typegen;

use crate::core::console::logging::{self, LoggingType};
use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::traits::TypeArrayEntensions;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::{builder::Builder, context::Context};

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_with_anchor(context, items, kind, cast, anchor)
        } else {
            self::compile_without_anchor(context, items, kind, cast)
        }
    } else {
        self::compile_without_anchor(context, items, kind, cast)
    }
}

fn compile_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    kind: &Type,
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let base_array_type: &Type = cast.unwrap_or(kind);
    let array_items_type: &Type = base_array_type.get_array_base_type();

    let array_type: Type = Type::FixedArray(array_items_type.clone().into(), items.len() as u32);

    let array_wrapper_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, base_array_type);

    let array_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, &array_type);

    let array_wrapper_type: BasicTypeEnum = typegen::generate_type(llvm_context, base_array_type);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile_expr(context, item, Some(array_items_type)))
        .collect();

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            array_ptr,
            &array_type,
            &[llvm_context.i32_type().const_zero(), llvm_idx],
        );

        memory::store_anon(context, ptr, *item);
    }

    let array_ptr_gep: PointerValue = llvm_builder
        .build_struct_gep(array_wrapper_type, array_wrapper_ptr, 0, "")
        .unwrap_or_else(|_| self::codegen_abort("Failed to build pointer array gep."));

    let array_size_gep: PointerValue = llvm_builder
        .build_struct_gep(array_wrapper_type, array_wrapper_ptr, 1, "")
        .unwrap_or_else(|_| self::codegen_abort("Failed to build pointer array gep."));

    memory::store_anon(context, array_ptr_gep, array_ptr.into());

    memory::store_anon(
        context,
        array_size_gep,
        llvm_context
            .i64_type()
            .const_int(items.len() as u64, false)
            .into(),
    );

    memory::load_anon(context, array_wrapper_ptr, base_array_type)
}

fn compile_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    kind: &Type,
    cast: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let anchor_ptr: PointerValue = anchor.get_pointer();

    let base_array_type: &Type = cast.unwrap_or(kind);
    let array_items_type: &Type = base_array_type.get_array_base_type();

    let array_type: Type = Type::FixedArray(array_items_type.clone().into(), items.len() as u32);

    context.set_pointer_anchor(PointerAnchor::new(anchor_ptr, true));

    let array_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, &array_type);

    let array_wrapper_type: BasicTypeEnum = typegen::generate_type(llvm_context, base_array_type);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile_expr(context, item, Some(array_items_type)))
        .collect();

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            array_ptr,
            &array_type,
            &[llvm_context.i32_type().const_zero(), llvm_idx],
        );

        memory::store_anon(context, ptr, *item);
    }

    let array_ptr_gep: PointerValue = llvm_builder
        .build_struct_gep(array_wrapper_type, anchor_ptr, 0, "")
        .unwrap_or_else(|_| self::codegen_abort("Failed to build pointer array gep."));

    let array_size_gep: PointerValue = llvm_builder
        .build_struct_gep(array_wrapper_type, anchor_ptr, 1, "")
        .unwrap_or_else(|_| self::codegen_abort("Failed to build pointer array gep."));

    memory::store_anon(context, array_ptr_gep, array_ptr.into());

    memory::store_anon(
        context,
        array_size_gep,
        llvm_context
            .i64_type()
            .const_int(items.len() as u64, false)
            .into(),
    );

    self::compile_null_ptr(context)
}

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
