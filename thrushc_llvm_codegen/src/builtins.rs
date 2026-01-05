use inkwell::{
    builder::Builder,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValueEnum, IntValue, PointerValue},
};
use thrushc_ast::traits::AstCodeLocation;
use thrushc_llvm_builtins::LLVMBuiltin;
use thrushc_span::Span;
use thrushc_typesystem::Type;

use crate::{
    abort, cast, codegen, context::LLVMCodeGenContext, traits::AstLLVMGetType, typegeneration,
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    builtin: LLVMBuiltin<'ctx>,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match builtin {
        LLVMBuiltin::MemCpy {
            src: source,
            dst: destination,
            size,
            span,
        } => {
            let llvm_builder: &Builder = context.get_llvm_builder();

            let src_span: Span = source.get_span();
            let src: PointerValue =
                codegen::compile_as_ptr(context, source, Some(&Type::Ptr(None, src_span)))
                    .into_pointer_value();

            let dest_span: Span = destination.get_span();
            let dest: PointerValue =
                codegen::compile_as_ptr(context, destination, Some(&Type::Ptr(None, dest_span)))
                    .into_pointer_value();

            let size: IntValue = codegen::compile(context, size, None).into_int_value();

            let source_type: &Type = source.llvm_get_type();
            let destination_type: &Type = destination.llvm_get_type();

            let src_type: BasicTypeEnum = typegeneration::compile_from(context, source_type);
            let dest_type: BasicTypeEnum = typegeneration::compile_from(context, destination_type);

            let src_alignment: u32 = context.get_target_data().get_preferred_alignment(&src_type);
            let dest_alignment: u32 = context
                .get_target_data()
                .get_preferred_alignment(&dest_type);

            llvm_builder
                .build_memcpy(dest, dest_alignment, src, src_alignment, size)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile 'memcpy' builtin!",
                        span,
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                })
                .into()
        }
        LLVMBuiltin::MemMove {
            src: source,
            dst: destination,
            size,
            span,
        } => {
            let llvm_builder: &Builder = context.get_llvm_builder();

            let src_span: Span = source.get_span();
            let src: PointerValue =
                codegen::compile_as_ptr(context, source, Some(&Type::Ptr(None, src_span)))
                    .into_pointer_value();

            let dest_span: Span = destination.get_span();
            let dest: PointerValue =
                codegen::compile_as_ptr(context, destination, Some(&Type::Ptr(None, dest_span)))
                    .into_pointer_value();

            let size: IntValue = codegen::compile(context, size, None).into_int_value();

            let source_type: &Type = source.llvm_get_type();
            let destination_type: &Type = destination.llvm_get_type();

            let src_type: BasicTypeEnum = typegeneration::compile_from(context, source_type);
            let dest_type: BasicTypeEnum = typegeneration::compile_from(context, destination_type);

            let src_alignment: u32 = context.get_target_data().get_preferred_alignment(&src_type);
            let dest_alignment: u32 = context
                .get_target_data()
                .get_preferred_alignment(&dest_type);

            llvm_builder
                .build_memmove(dest, dest_alignment, src, src_alignment, size)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile 'memmove' builtin!",
                        span,
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                })
                .into()
        }
        LLVMBuiltin::MemSet {
            dst: destination,
            new_size,
            size,
            span,
        } => {
            let llvm_builder: &Builder = context.get_llvm_builder();

            let dest_span: Span = destination.get_span();
            let dest: PointerValue =
                codegen::compile_as_ptr(context, destination, Some(&Type::Ptr(None, dest_span)))
                    .into_pointer_value();

            let new_size: IntValue = codegen::compile(context, new_size, None).into_int_value();
            let size: IntValue = codegen::compile(context, size, None).into_int_value();

            let destination_type: &Type = destination.llvm_get_type();

            let dest_type: BasicTypeEnum = typegeneration::compile_from(context, destination_type);
            let dest_alignment: u32 = context
                .get_target_data()
                .get_preferred_alignment(&dest_type);

            llvm_builder
                .build_memset(dest, dest_alignment, new_size, size)
                .unwrap_or_else(|_| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile 'memset' builtin!",
                        span,
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                })
                .into()
        }
        LLVMBuiltin::Malloc { of, span } => context
            .get_llvm_builder()
            .build_malloc(typegeneration::compile_from(context, of), "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile 'halloc' builtin!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
        LLVMBuiltin::AlignOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, of);

            let alignment: u32 = context
                .get_target_data()
                .get_preferred_alignment(&llvm_type);

            let alignment: BasicValueEnum = context
                .get_llvm_context()
                .i32_type()
                .const_int(alignment.into(), false)
                .into();

            cast::try_cast(context, cast_type, &Type::U32(span), alignment, span)
        }
        LLVMBuiltin::SizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, of);

            let sizeof_value: BasicValueEnum = llvm_type
                .size_of()
                .unwrap_or_else(|| {
                    abort::abort_codegen(
                        context,
                        "Failed to compile 'sizeof' builtin!",
                        span,
                        std::path::PathBuf::from(file!()),
                        line!(),
                    )
                })
                .into();

            cast::try_cast(context, cast_type, &Type::USize(span), sizeof_value, span)
        }
        LLVMBuiltin::AbiSizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, of);
            let abi_size: u64 = context.get_target_data().get_abi_size(&llvm_type);
            let size: BasicValueEnum = context
                .get_llvm_context()
                .i64_type()
                .const_int(abi_size, false)
                .into();

            cast::try_cast(context, cast_type, &Type::U64(span), size, span)
        }
        LLVMBuiltin::BitSizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, of);
            let bit_size: u64 = context.get_target_data().get_bit_size(&llvm_type);
            let size: BasicValueEnum = context
                .get_llvm_context()
                .i64_type()
                .const_int(bit_size, false)
                .into();

            cast::try_cast(context, cast_type, &Type::U64(span), size, span)
        }
        LLVMBuiltin::AbiAlignOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::compile_from(context, of);
            let abi_align: u32 = context.get_target_data().get_abi_alignment(&llvm_type);

            let align: BasicValueEnum = context
                .get_llvm_context()
                .i32_type()
                .const_int(abi_align.into(), false)
                .into();

            cast::try_cast(context, cast_type, &Type::U32(span), align, span)
        }
    }
}
