use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use inkwell::{builder::Builder, values::BasicValueEnum};
use thrushc_ast::Ast;
use thrushc_ast::data::PropertyData;
use thrushc_ast::traits::AstMemoryExtensions;
use thrushc_ast::traits::{
    AstCodeLocation, AstPropertyDataExtensions, AstPropertyDataFieldExtensions,
};
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
    data: &'ctx PropertyData,
) -> BasicValueEnum<'ctx> {
    let source_type: &Type = source.llvm_get_type();

    if (source.is_allocated() && source_type.is_struct_type())
        || source_type.is_ptr_composite_type()
    {
        self::compile_gep_property(context, source, data)
    } else {
        self::compile_extract_property(context, source, data)
    }
}

fn compile_extract_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    source: &'ctx Ast<'ctx>,
    data: &'ctx PropertyData,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let span: Span = source.get_span();

    let mut property: BasicValueEnum = {
        let value: BasicValueEnum = codegen::compile(context, source, None);

        let index: u32 = data
            .get_first_property()
            .unwrap_or_else(|| {
                abort::abort_codegen(
                    context,
                    "Failed to extract a value from a struct!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .get_index();

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

    for field in data.iter().skip(1) {
        let index: u32 = field.get_index();

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
    data: &'ctx PropertyData,
) -> BasicValueEnum<'ctx> {
    let llvm_builder: &Builder = context.get_llvm_builder();

    let span: Span = source.get_span();

    let ptr: PointerValue = codegen::compile_as_ptr(context, source, None).into_pointer_value();
    let ptr_type: &Type = source.llvm_get_type();

    let index: u32 = data
        .get_first_property()
        .unwrap_or_else(|| {
            abort::abort_codegen(
                context,
                "Failed to gep a value from pointer!",
                span,
                std::path::PathBuf::from(file!()),
                line!(),
            )
        })
        .get_index();

    let mut property: PointerValue = memory::gep_struct_anon(context, ptr, ptr_type, index, span);

    for field in data.iter().skip(1) {
        let base_type: Type = field.get_base_type();
        let index: u32 = field.get_index();

        let pointee_ty: BasicTypeEnum = typegeneration::compile_gep_type(context, &base_type);

        property = llvm_builder
            .build_struct_gep(pointee_ty, property, index, "")
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
