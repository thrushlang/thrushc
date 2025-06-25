use std::fmt::Display;

use super::context::LLVMCodeGenContext;
use super::typegen;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite};

use crate::backend::llvm::compiler::{rawgen, valuegen};
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::ast::Ast;
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::lexer::traits::{
    ThrushTypeMutableExtensions, ThrushTypePointerExtensions,
};

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
    if items
        .iter()
        .all(|item| item.is_integer() || item.is_float() || item.is_bool())
    {
        self::constant_fixed_array(context, kind, items, cast_type)
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

    let array_type: &ThrushType = self::get_fixed_array_type(kind, cast_type);
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

fn constant_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [Ast],
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_item_type: &ThrushType =
        self::get_fixed_array_type(kind, cast_type).get_fixed_array_base_type();
    let array_type: BasicTypeEnum = typegen::generate_type(llvm_context, array_item_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| {
            let item_type = item.get_type_unwrapped();
            if item_type.is_ptr_type() || item_type.is_mut_type() {
                rawgen::compile(context, item, Some(array_item_type))
            } else {
                valuegen::compile(context, item, Some(array_item_type))
            }
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
                "Incompatible type '{}' for constant array",
                array_item_type
            ));

            self::compile_null_ptr(context)
        }
    }
}

pub fn compile_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [Ast],
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let base_array_type: &ThrushType = self::get_array_type(kind, cast_type);
    let array_items_type: &ThrushType = base_array_type.get_array_type();

    let array_type: ThrushType =
        ThrushType::FixedArray(array_items_type.clone().into(), items.len() as u32);

    let array_wrapper_ptr: PointerValue =
        if let Some(preferred_site_allocation) = context.get_site_allocation() {
            memory::alloc_anon(preferred_site_allocation, context, base_array_type, true)
        } else {
            memory::alloc_anon(LLVMAllocationSite::Stack, context, base_array_type, true)
        };

    let array_ptr: PointerValue =
        if let Some(preferred_site_allocation) = context.get_site_allocation() {
            memory::alloc_anon(preferred_site_allocation, context, &array_type, true)
        } else {
            memory::alloc_anon(LLVMAllocationSite::Stack, context, &array_type, true)
        };

    let array_wrapper_type: BasicTypeEnum = typegen::generate_type(llvm_context, base_array_type);
    let array_ptr_type: BasicTypeEnum = typegen::generate_type(llvm_context, &array_type);

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        match unsafe {
            llvm_builder.build_in_bounds_gep(
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
                    "Failed to calculate memory address for array element at index '{}'",
                    idx
                ));

                return self::compile_null_ptr(context);
            }
        }
    }

    let array_ptr_gep: PointerValue = llvm_builder
        .build_struct_gep(array_wrapper_type, array_wrapper_ptr, 0, "")
        .unwrap();

    memory::store_anon(context, array_ptr_gep, array_ptr.into());

    let array_size_gep: PointerValue = llvm_builder
        .build_struct_gep(array_wrapper_type, array_wrapper_ptr, 1, "")
        .unwrap();

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

fn get_fixed_array_type<'ctx>(
    kind: &'ctx ThrushType,
    cast_type: Option<&'ctx ThrushType>,
) -> &'ctx ThrushType {
    if let Some(cast_type) = cast_type {
        if cast_type.is_mut_fixed_array_type()
            || cast_type.is_ptr_fixed_array_type()
            || cast_type.is_fixed_array_type()
        {
            cast_type
        } else {
            kind
        }
    } else {
        kind
    }
}

fn get_array_type<'ctx>(
    kind: &'ctx ThrushType,
    cast_type: Option<&'ctx ThrushType>,
) -> &'ctx ThrushType {
    if let Some(cast_type) = cast_type {
        if cast_type.is_mut_array_type()
            || cast_type.is_ptr_array_type()
            || cast_type.is_array_type()
        {
            cast_type
        } else {
            kind
        }
    } else {
        kind
    }
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(
        LoggingType::Bug,
        &format!("CODE GENERATION: '{}'.", message),
    );
}
