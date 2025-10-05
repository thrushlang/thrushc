use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::{abort, codegen, memory, ptr, typegen};

use crate::backends::classical::types::LLVMGEPIndexes;
use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::typesystem::traits::LLVMTypeExtensions;
use crate::frontends::classical::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use inkwell::{builder::Builder, values::BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: LLVMGEPIndexes<'ctx>,
) -> BasicValueEnum<'ctx> {
    if source.is_allocated() || source.get_type_unwrapped().llvm_is_ptr_type() {
        return self::compile_gep_property(context, source, indexes);
    }

    self::compile_extract_property(context, source, indexes)
}

fn compile_extract_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: LLVMGEPIndexes<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let span: Span = source.get_span();

    let mut property: BasicValueEnum = {
        let value: BasicValueEnum = codegen::compile(context, source, None);

        llvm_builder
            .build_extract_value(value.into_struct_value(), indexes[0].1, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to extract a value from struct!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            })
    };

    for idx in indexes.iter().skip(1) {
        property = llvm_builder
            .build_extract_value(property.into_struct_value(), idx.1, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to extract a value from struct!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    property
}

fn compile_gep_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: LLVMGEPIndexes<'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let span: Span = source.get_span();

    let ptr: PointerValue = ptr::compile(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.get_type_unwrapped();

    let mut property: PointerValue = memory::gep_struct_anon(context, ptr, ptr_type, indexes[0].1);

    for idx in indexes.iter().skip(1) {
        let idx_type: BasicTypeEnum = typegen::generate(llvm_context, &idx.0);

        property = llvm_builder
            .build_struct_gep(idx_type, property, idx.1, "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to gep a value from pointer!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    property.into()
}
