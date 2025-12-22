use std::path::PathBuf;

use crate::back_end::llvm_codegen::anchors::PointerAnchor;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::generation::cast;
use crate::back_end::llvm_codegen::memory::LLVMAllocationSite;
use crate::back_end::llvm_codegen::{abort, typegen};
use crate::back_end::llvm_codegen::{codegen, constgen, memory};

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstLLVMGetType;
use crate::front_end::typesystem::traits::TypeFixedArrayEntensions;
use crate::front_end::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let base_type: Type = array_type.get_fixed_array_base_type();
    let array_type: BasicTypeEnum = typegen::generate(context, &base_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| {
            let value_type: &Type = item.llvm_get_type(context);
            let value: BasicValueEnum = constgen::compile(context, item, &base_type);

            cast::try_cast_const(context, value, value_type, &base_type)
        })
        .collect();

    match array_type {
        t if t.is_int_type() => t
            .into_int_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_int_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_float_type() => t
            .into_float_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_float_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_array_type() => t
            .into_array_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_array_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_struct_type() => t
            .into_struct_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_struct_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        t if t.is_pointer_type() => t
            .into_pointer_type()
            .const_array(
                &values
                    .iter()
                    .map(|v| v.into_pointer_value())
                    .collect::<Vec<_>>(),
            )
            .into(),
        _ => {
            abort::abort_codegen(
                context,
                "Failed to compile the constant array!",
                span,
                PathBuf::from(file!()),
                line!(),
            );
        }
    }
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_fixed_array_with_anchor(
                context, items, array_type, span, cast_type, anchor,
            )
        } else {
            self::compile_fixed_array_without_anchor(context, items, array_type, span, cast_type)
        }
    } else {
        self::compile_fixed_array_without_anchor(context, items, array_type, span, cast_type)
    }
}

fn compile_fixed_array_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let anchor: PointerValue = anchor.get_pointer();

    let array_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: Type = array_type.get_fixed_array_base_type();

    let llvm_type: BasicTypeEnum = typegen::generate(context, array_type);

    context.set_pointer_anchor(PointerAnchor::new(anchor, true));

    if items.is_empty() {
        memory::store_anon(context, anchor, llvm_type.const_zero(), span);

        return context
            .get_llvm_context()
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
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
            anchor,
            array_type,
            &[llvm_context.i32_type().const_zero(), index],
            span,
        );

        memory::store_anon(context, ptr, *value, span);
    }

    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn compile_fixed_array_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
    span: Span,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: Type = array_type.get_fixed_array_base_type();

    let llvm_type: BasicTypeEnum = typegen::generate(context, array_type);

    if items.is_empty() {
        return llvm_type.const_zero();
    }

    let array_ptr: PointerValue =
        memory::alloc_anon(context, LLVMAllocationSite::Stack, array_type, span);

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
            array_type,
            &[llvm_context.i32_type().const_zero(), index],
            span,
        );

        memory::store_anon(context, ptr, *value, span);
    }

    memory::load_anon(context, array_ptr, array_type, span)
}
