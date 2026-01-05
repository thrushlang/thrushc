use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use inkwell::{builder::Builder, values::BasicValueEnum};
use thrushc_ast::Ast;
use thrushc_ast::traits::AstCodeLocation;
use thrushc_ast::traits::AstMemoryExtensions;
use thrushc_span::Span;
use thrushc_typesystem::Type;
use thrushc_typesystem::traits::TypeIsExtensions;
use thrushc_typesystem::traits::TypePointerExtensions;

use crate::context::LLVMCodeGenContext;
use crate::traits::AstLLVMGetType;
use crate::{abort, codegen, memory, typegeneration};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &'ctx [(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let source_type: &Type = source.llvm_get_type();

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
    indexes: &'ctx [(Type, u32)],
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
                    std::path::PathBuf::from(file!()),
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
                    std::path::PathBuf::from(file!()),
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
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    property
}

fn compile_gep_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    indexes: &'ctx [(Type, u32)],
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let span: Span = source.get_span();

    let ptr: PointerValue = codegen::compile_as_ptr(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.llvm_get_type();

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
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            });
    }

    property.into()
}
