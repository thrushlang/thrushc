use crate::backend::llvm::compiler::anchors::PointerAnchor;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite};

use crate::backend::llvm::compiler::valuegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    self::fixed_array(context, kind, items, cast)
}

fn fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_fixed_array_with_anchor(context, kind, items, cast, anchor)
        } else {
            self::compile_fixed_array_without_anchor(context, kind, items, cast)
        }
    } else {
        self::compile_fixed_array_without_anchor(context, kind, items, cast)
    }
}

fn compile_fixed_array_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
    cast: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_ptr: PointerValue = anchor.get_pointer();
    let array_type: &Type = cast.unwrap_or(kind);

    context.set_pointer_anchor(PointerAnchor::new(array_ptr, true));

    let array_items_type: &Type = array_type.get_fixed_array_base_type();

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| valuegen::compile(context, item, Some(array_items_type)))
        .collect();

    for (idx, item) in items.iter().enumerate() {
        let idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            array_ptr,
            array_type,
            &[llvm_context.i32_type().const_zero(), idx],
        );

        memory::store_anon(context, ptr, *item);
    }

    self::compile_null_ptr(context)
}

fn compile_fixed_array_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
    cast: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_type: &Type = cast.unwrap_or(kind);
    let array_items_type: &Type = array_type.get_fixed_array_base_type();

    let array_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, array_type);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| valuegen::compile(context, item, Some(array_items_type)))
        .collect();

    for (idx, item) in items.iter().enumerate() {
        let idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            array_ptr,
            array_type,
            &[llvm_context.i32_type().const_zero(), idx],
        );

        memory::store_anon(context, ptr, *item);
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
