use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;

use crate::back_end::llvm_codegen::helpertypes::LLVMGEPIndexes;
use crate::back_end::llvm_codegen::{abort, codegen, memory, typegeneration};

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstLLVMGetType, AstMemoryExtensions};
use crate::front_end::typesystem::traits::TypeIsExtensions;
use crate::front_end::typesystem::traits::TypePointerExtensions;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use inkwell::{builder::Builder, values::BasicValueEnum};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: LLVMGEPIndexes<'ctx>,
) -> BasicValueEnum<'ctx> {
    let source_type: &Type = source.llvm_get_type(context);

    if (source.is_allocated() && source_type.is_struct_type())
        || source_type.is_ptr_composite_type()
    {
        self::compile_gep_property(context, source, indexes)
    } else {
        self::compile_extract_property(context, source, indexes)
    }
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
        let index: u32 = indexes
            .first()
            .unwrap_or_else(|| {
                abort::abort_codegen(
                    context,
                    "Failed to extract the from struct!",
                    span,
                    PathBuf::from(file!()),
                    line!(),
                )
            })
            .1;

        llvm_builder
            .build_extract_value(value.into_struct_value(), index, "")
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

    for n in indexes.iter().skip(1) {
        let index: u32 = n.1;

        property = llvm_builder
            .build_extract_value(property.into_struct_value(), index, "")
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
    let llvm_builder: &Builder = context.get_llvm_builder();

    let span: Span = source.get_span();

    let ptr: PointerValue = codegen::compile_as_ptr(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.llvm_get_type(context);

    let mut property: PointerValue =
        memory::gep_struct_anon(context, ptr, ptr_type, indexes[0].1, span);

    for n in indexes.iter().skip(1) {
        let index: u32 = n.1;
        let index_type: &Type = &n.0;

        let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, index_type);

        property = llvm_builder
            .build_struct_gep(llvm_type, property, index, "")
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
