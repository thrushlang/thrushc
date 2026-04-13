/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

use inkwell::{
    builder::Builder,
    types::BasicTypeEnum,
    values::{BasicValueEnum, IntValue, PointerValue},
};
use thrustc_ast::traits::AstCodeLocation;
use thrustc_span::Span;
use thrustc_typesystem::Type;

use thrustc_ast::{Ast, builitins::ThrustBuiltin};

use crate::{
    abort, cast, codegen, context::LLVMCodeGenContext, traits::AstLLVMGetType, typegeneration,
};

#[derive(Debug, Clone)]
pub enum LLVMBuiltin<'ctx> {
    Malloc {
        of: &'ctx Type,
        span: Span,
    },
    MemCpy {
        src: &'ctx Ast<'ctx>,
        dst: &'ctx Ast<'ctx>,
        size: &'ctx Ast<'ctx>,
        span: Span,
    },
    MemMove {
        src: &'ctx Ast<'ctx>,
        dst: &'ctx Ast<'ctx>,
        size: &'ctx Ast<'ctx>,
        span: Span,
    },
    MemSet {
        dst: &'ctx Ast<'ctx>,
        new_size: &'ctx Ast<'ctx>,
        size: &'ctx Ast<'ctx>,
        span: Span,
    },
    AbiSizeOf {
        of: &'ctx Type,
        span: Span,
    },
    BitSizeOf {
        of: &'ctx Type,
        span: Span,
    },
    AbiAlignOf {
        of: &'ctx Type,
        span: Span,
    },
    AlignOf {
        of: &'ctx Type,
        span: Span,
    },
    SizeOf {
        of: &'ctx Type,
        span: Span,
    },
}

pub fn into_llvm_builtin<'ctx>(thrust_builtin: &'ctx ThrustBuiltin) -> LLVMBuiltin<'ctx> {
    match thrust_builtin {
        ThrustBuiltin::Halloc { of, span } => LLVMBuiltin::Malloc { of, span: *span },
        ThrustBuiltin::MemCpy {
            src,
            dst,
            size,
            span,
        } => LLVMBuiltin::MemCpy {
            src: src.as_ref(),
            dst,
            size,
            span: *span,
        },
        ThrustBuiltin::MemMove {
            src,
            dst,
            size,
            span,
        } => LLVMBuiltin::MemMove {
            src,
            dst,
            size,
            span: *span,
        },
        ThrustBuiltin::MemSet {
            dst,
            new_size,
            size,
            span,
        } => LLVMBuiltin::MemSet {
            dst,
            new_size,
            size,
            span: *span,
        },
        ThrustBuiltin::AlignOf { of, span } => LLVMBuiltin::AlignOf { of, span: *span },
        ThrustBuiltin::SizeOf { of, span } => LLVMBuiltin::SizeOf { of, span: *span },
        ThrustBuiltin::BitSizeOf { of, span } => LLVMBuiltin::BitSizeOf { of, span: *span },
        ThrustBuiltin::AbiSizeOf { of, span } => LLVMBuiltin::AbiSizeOf { of, span: *span },
        ThrustBuiltin::AbiAlignOf { of, span } => LLVMBuiltin::AbiAlignOf { of, span: *span },
    }
}

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
                codegen::compile_as_ptr_value(context, source, Some(&Type::Ptr(None, src_span)))
                    .into_pointer_value();

            let dest_span: Span = destination.get_span();
            let dest: PointerValue = codegen::compile_as_ptr_value(
                context,
                destination,
                Some(&Type::Ptr(None, dest_span)),
            )
            .into_pointer_value();

            let size: IntValue = codegen::compile_as_value(context, size, None).into_int_value();

            let source_type: &Type = source.llvm_get_type();
            let destination_type: &Type = destination.llvm_get_type();

            let src_type: BasicTypeEnum = typegeneration::generate_type(context, source_type);
            let dest_type: BasicTypeEnum = typegeneration::generate_type(context, destination_type);

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
                codegen::compile_as_ptr_value(context, source, Some(&Type::Ptr(None, src_span)))
                    .into_pointer_value();

            let dest_span: Span = destination.get_span();
            let dest: PointerValue = codegen::compile_as_ptr_value(
                context,
                destination,
                Some(&Type::Ptr(None, dest_span)),
            )
            .into_pointer_value();

            let size: IntValue = codegen::compile_as_value(context, size, None).into_int_value();

            let source_type: &Type = source.llvm_get_type();
            let destination_type: &Type = destination.llvm_get_type();

            let src_type: BasicTypeEnum = typegeneration::generate_type(context, source_type);
            let dest_type: BasicTypeEnum = typegeneration::generate_type(context, destination_type);

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
            let dest: PointerValue = codegen::compile_as_ptr_value(
                context,
                destination,
                Some(&Type::Ptr(None, dest_span)),
            )
            .into_pointer_value();

            let new_size: IntValue =
                codegen::compile_as_value(context, new_size, None).into_int_value();
            let size: IntValue = codegen::compile_as_value(context, size, None).into_int_value();

            let destination_type: &Type = destination.llvm_get_type();

            let dest_type: BasicTypeEnum = typegeneration::generate_type(context, destination_type);
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
            .build_malloc(typegeneration::generate_type(context, of), "")
            .unwrap_or_else(|_| {
                abort::abort_codegen(
                    context,
                    "Failed to compile 'malloc' builtin!",
                    span,
                    std::path::PathBuf::from(file!()),
                    line!(),
                )
            })
            .into(),
        LLVMBuiltin::AlignOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::generate_type(context, of);

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
            let type_layout_result: either::Either<
                thrustc_typesystem::type_layout::TypeLayout,
                thrustc_typesystem::type_layout::StructTypeLayout,
            > = context.get_mut_target_info().get_type_layout(of);

            let size_of: u32 = match type_layout_result {
                either::Either::Left(t) => t.into_layout().sizeof,
                either::Either::Right(t) => t.into_layout().sizeof,
            };

            let sizeof_value: BasicValueEnum = context
                .get_llvm_context()
                .i32_type()
                .const_int(size_of.into(), false)
                .into();

            cast::try_cast(context, cast_type, &Type::USize(span), sizeof_value, span)
        }
        LLVMBuiltin::AbiSizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::generate_type(context, of);
            let abi_size: u64 = context.get_target_data().get_abi_size(&llvm_type);
            let size: BasicValueEnum = context
                .get_llvm_context()
                .i64_type()
                .const_int(abi_size, false)
                .into();

            cast::try_cast(context, cast_type, &Type::U64(span), size, span)
        }
        LLVMBuiltin::BitSizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::generate_type(context, of);
            let bit_size_bits: u64 = context.get_target_data().get_bit_size(&llvm_type);
            let bit_size_bytes: u64 = bit_size_bits.saturating_div(8);

            let size: BasicValueEnum = context
                .get_llvm_context()
                .i64_type()
                .const_int(bit_size_bytes, false)
                .into();

            cast::try_cast(context, cast_type, &Type::U64(span), size, span)
        }
        LLVMBuiltin::AbiAlignOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegeneration::generate_type(context, of);
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
