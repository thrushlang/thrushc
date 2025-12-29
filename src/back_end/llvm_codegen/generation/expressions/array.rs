use std::path::PathBuf;

use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::localanchor::PointerAnchor;
use crate::back_end::llvm_codegen::memory::LLVMAllocationSite;
use crate::back_end::llvm_codegen::{abort, memory};
use crate::back_end::llvm_codegen::{codegen, typegeneration};

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::traits::TypeArrayEntensions;
use crate::front_end::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

#[inline]
pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match context.get_pointer_anchor() {
        Some(anchor) if !anchor.is_triggered() => {
            self::compile_array_with_anchor(context, items, array_type, span, cast_type, anchor)
        }

        _ => self::compile_array_without_anchor(context, items, array_type, span, cast_type),
    }
}

fn compile_array_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let base_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: Type = base_type.get_array_base_type();

    let array_size: u32 = u32::try_from(items.len()).unwrap_or_else(|_| {
        abort::abort_codegen(
            context,
            "Failed to parse the size!",
            span,
            PathBuf::from(file!()),
            line!(),
        )
    });

    let array_type: Type = Type::FixedArray(items_type.clone().into(), array_size, span);

    let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, &array_type);

    let array_ptr: PointerValue =
        memory::alloc_anon(context, LLVMAllocationSite::Stack, &array_type, span);

    if items.is_empty() {
        memory::store_anon(context, array_ptr, llvm_type.const_zero(), span);
        return array_ptr.into();
    }

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile(context, item, Some(&items_type)))
        .collect();

    for (n, value) in items.iter().enumerate() {
        let idx: u64 = u64::try_from(n).unwrap_or_else(|_| {
            abort::abort_codegen(
                context,
                "Failed to parse the build index!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        });

        let index: IntValue = llvm_context.i32_type().const_int(idx, false);

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

fn compile_array_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let anchor: PointerValue = anchor.get_pointer();

    let array_size: u32 = u32::try_from(items.len()).unwrap_or_else(|_| {
        abort::abort_codegen(
            context,
            "Failed to parse the size!",
            span,
            PathBuf::from(file!()),
            line!(),
        )
    });

    let base_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: Type = base_type.get_array_base_type();

    let array_type: Type = Type::FixedArray(items_type.clone().into(), array_size, span);
    let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, &array_type);

    context.set_pointer_anchor(PointerAnchor::new(anchor, true));

    if items.is_empty() {
        memory::store_anon(context, anchor, llvm_type.const_zero(), span);
        return anchor.into();
    }

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile(context, item, Some(&items_type)))
        .collect();

    let ptr: Option<PointerValue> = items
        .iter()
        .enumerate()
        .map(|(n, value)| {
            let idx: u64 = u64::try_from(n).unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to parse the build index!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            });

            let index: IntValue = llvm_context.i32_type().const_int(idx, false);

            let ptr: PointerValue = memory::gep_anon(
                context,
                anchor,
                &array_type,
                &[llvm_context.i32_type().const_zero(), index],
                span,
            );

            memory::store_anon(context, ptr, *value, span);

            ptr
        })
        .last();

    ptr.unwrap_or(
        context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null(),
    )
    .into()
}
