use crate::backend::llvm::compiler::anchors::PointerAnchor;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite};

use crate::backend::llvm::compiler::{typegen, valuegen};
use crate::frontend::types::ast::Ast;
use crate::frontend::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::{builder::Builder, context::Context};

pub fn compile<'ctx>(
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

pub fn compile_fixed_array_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
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
        .map(|item| valuegen::compile(context, item, Some(array_items_type)))
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

fn compile_fixed_array_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &Type,
    items: &'ctx [Ast],
    cast: Option<&Type>,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let base_array_type: &Type = cast.unwrap_or(kind);
    let array_items_type: &Type = base_array_type.get_fixed_array_base_type();

    let array_type: Type = Type::FixedArray(array_items_type.clone().into(), items.len() as u32);

    let array_wrapper_ptr: PointerValue = anchor.get_pointer();

    context.set_pointer_anchor(PointerAnchor::new(array_wrapper_ptr, true));

    let array_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, &array_type);

    let array_wrapper_type: BasicTypeEnum = typegen::generate_type(llvm_context, base_array_type);

    let items: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| valuegen::compile(context, item, Some(array_items_type)))
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

    self::compile_null_ptr(context)
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
