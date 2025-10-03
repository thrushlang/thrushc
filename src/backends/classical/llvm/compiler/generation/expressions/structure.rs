use crate::backends::classical::llvm::compiler::anchors::PointerAnchor;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::memory::LLVMAllocationSite;
use crate::backends::classical::llvm::compiler::typegen;
use crate::backends::classical::llvm::compiler::{codegen, memory};

use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::types::parser::stmts::types::Constructor;
use crate::frontends::classical::typesystem::traits::TypeStructExtensions;
use crate::frontends::classical::typesystem::types::Type;

use inkwell::AddressSpace;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    args: &'ctx Constructor,
    struct_type: &Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    if let Some(anchor) = context.get_pointer_anchor() {
        if !anchor.is_triggered() {
            self::compile_with_anchor(context, args, struct_type, span, anchor)
        } else {
            self::compile_without_anchor(context, args, struct_type, span)
        }
    } else {
        self::compile_without_anchor(context, args, struct_type, span)
    }
}

fn compile_with_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    args: &'ctx Constructor,
    struct_type: &Type,
    span: Span,
    anchor: PointerAnchor<'ctx>,
) -> BasicValueEnum<'ctx> {
    let ptr_type: BasicTypeEnum = typegen::generate(context.get_llvm_context(), struct_type);
    let ptr: PointerValue = anchor.get_pointer();

    context.set_pointer_anchor(PointerAnchor::new(ptr, true));

    let fields_types: &[Type] = struct_type.get_struct_fields();

    let fields: Vec<BasicValueEnum> = args
        .iter()
        .zip(fields_types)
        .map(|((_, field, _, _), kind)| codegen::compile(context, field, Some(kind)))
        .collect();

    for (idx, value) in fields.iter().enumerate() {
        if let Ok(ptr) = context
            .get_llvm_builder()
            .build_struct_gep(ptr_type, ptr, idx as u32, "")
        {
            memory::store_anon(context, ptr, *value, span);
        }
    }

    self::compile_null_ptr(context)
}

fn compile_without_anchor<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    args: &'ctx Constructor,
    struct_type: &Type,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let ptr_type: BasicTypeEnum = typegen::generate(context.get_llvm_context(), struct_type);
    let ptr: PointerValue = memory::alloc_anon(LLVMAllocationSite::Stack, context, struct_type);

    let fields_types: &[Type] = struct_type.get_struct_fields();

    let fields: Vec<BasicValueEnum> = args
        .iter()
        .zip(fields_types)
        .map(|((_, field, _, _), kind)| codegen::compile(context, field, Some(kind)))
        .collect();

    for (idx, value) in fields.iter().enumerate() {
        if let Ok(ptr) = context
            .get_llvm_builder()
            .build_struct_gep(ptr_type, ptr, idx as u32, "")
        {
            memory::store_anon(context, ptr, *value, span);
        }
    }

    memory::load_anon(context, ptr, struct_type, span)
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}
