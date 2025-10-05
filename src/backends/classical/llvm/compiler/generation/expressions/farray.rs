use std::fmt::Display;

use crate::backends::classical::llvm::compiler::anchors::PointerAnchor;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::memory::LLVMAllocationSite;
use crate::backends::classical::llvm::compiler::typegen;
use crate::backends::classical::llvm::compiler::{self, codegen, constgen, memory};

use crate::core::console::logging::{self, LoggingType};
use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::traits::TypeFixedArrayEntensions;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};

pub fn compile_const<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    items: &'ctx [Ast],
    array_type: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let base_type: &Type = array_type.get_farray_base_type();
    let array_type: BasicTypeEnum = typegen::generate(llvm_context, base_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| {
            let value_type: &Type = item.llvm_get_type(context);
            let value: BasicValueEnum = constgen::compile(context, item, base_type);

            compiler::generation::cast::try_cast_const(context, value, value_type, base_type)
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
            self::codegen_abort(format!(
                "Incompatible type '{}' for constant array.",
                base_type
            ));
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

    let anchor_ptr: PointerValue = anchor.get_pointer();

    let array_type: &Type = cast_type.unwrap_or(array_type);
    let items_type: &Type = array_type.get_farray_base_type();

    context.set_pointer_anchor(PointerAnchor::new(anchor_ptr, true));

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile(context, item, Some(items_type)))
        .collect();

    for (idx, value) in items.iter().enumerate() {
        let index: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            anchor_ptr,
            array_type,
            &[llvm_context.i32_type().const_zero(), index],
        );

        memory::store_anon(context, ptr, *value, span);
    }

    self::compile_null_ptr(context)
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
    let items_type: &Type = array_type.get_farray_base_type();

    let array_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, array_type);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| codegen::compile(context, item, Some(items_type)))
        .collect();

    for (idx, value) in items.iter().enumerate() {
        let index: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let ptr: PointerValue = memory::gep_anon(
            context,
            array_ptr,
            array_type,
            &[llvm_context.i32_type().const_zero(), index],
        );

        memory::store_anon(context, ptr, *value, span);
    }

    memory::load_anon(context, array_ptr, array_type, span)
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
