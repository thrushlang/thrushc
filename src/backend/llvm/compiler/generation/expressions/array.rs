use crate::backend::llvm::compiler::anchors::PointerAnchor;
use crate::backend::llvm::compiler::codegen;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::memory;
use crate::backend::llvm::compiler::memory::LLVMAllocationSite;

use crate::frontend::lexer::span::Span;
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::traits::TypeArrayEntensions;
use crate::frontend::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_with_anchor(context, items, array_type, span, cast_type, anchor)
        } else {
            self::compile_without_anchor(context, items, array_type, span, cast_type)
        }
    } else {
        self::compile_without_anchor(context, items, array_type, span, cast_type)
    }
}

fn compile_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let base_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: &Type = base_type.get_array_base_type();

    let array_type: Type = Type::FixedArray(items_type.clone().into(), items.len() as u32);
    let array_ptr: PointerValue =
        memory::alloc_anon(context, LLVMAllocationSite::Stack, &array_type, span);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile(context, item, Some(items_type)))
        .collect();

    for (idx, value) in items.iter().enumerate() {
        let index: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            array_ptr,
            &array_type,
            &[llvm_context.i32_type().const_zero(), index],
            span,
        );

        memory::store_anon(context, ptr, *value, span);
    }

    array_ptr.into()
}

fn compile_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let anchor_ptr: PointerValue = anchor.get_pointer();

    let base_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: &Type = base_type.get_array_base_type();

    let array_type: Type = Type::FixedArray(items_type.clone().into(), items.len() as u32);

    context.set_pointer_anchor(PointerAnchor::new(anchor_ptr, true));

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile(context, item, Some(items_type)))
        .collect();

    let ptr: Option<PointerValue> = items
        .iter()
        .enumerate()
        .map(|(idx, value)| {
            let index: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

            let ptr: PointerValue = memory::gep_anon(
                context,
                anchor_ptr,
                &array_type,
                &[llvm_context.i32_type().const_zero(), index],
                span,
            );

            memory::store_anon(context, ptr, *value, span);

            ptr
        })
        .last();

    ptr.unwrap_or(self::compile_null_ptr(context).into_pointer_value())
        .into()
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
