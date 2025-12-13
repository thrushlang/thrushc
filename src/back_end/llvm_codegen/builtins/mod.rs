use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;

use crate::back_end::llvm_codegen::generation::cast;
use crate::back_end::llvm_codegen::typegen;
use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;

use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

pub mod mem;

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
    AlignOf {
        of: &'ctx Type,
        span: Span,
    },
    SizeOf {
        of: &'ctx Type,
        span: Span,
    },
}

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    builtin: LLVMBuiltin<'ctx>,
    cast_type: Option<&Type>,
) -> BasicValueEnum<'ctx> {
    match builtin {
        LLVMBuiltin::AlignOf { of, span } => mem::alingof::compile(context, of, span, cast_type),
        LLVMBuiltin::MemCpy {
            src,
            dst,
            size,
            span,
        } => mem::memcpy::compile(context, src, dst, size, span),
        LLVMBuiltin::MemMove {
            src,
            dst,
            size,
            span,
        } => mem::memmove::compile(context, src, dst, size, span),
        LLVMBuiltin::MemSet {
            dst,
            new_size,
            size,
            span,
        } => mem::memset::compile(context, dst, new_size, size, span),
        LLVMBuiltin::Malloc { of, span } => mem::malloc::compile(context, of, span),
        LLVMBuiltin::SizeOf { of, span } => mem::sizeof::compile(context, of, cast_type, span),
        LLVMBuiltin::AbiSizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegen::generate(context.get_llvm_context(), of);
            let abi_size: u64 = context.get_target_data().get_abi_size(&llvm_type);
            let size: BasicValueEnum = context
                .get_llvm_context()
                .i64_type()
                .const_int(abi_size, false)
                .into();

            cast::try_cast(context, cast_type, &Type::U64, size, span).unwrap_or(size)
        }
        LLVMBuiltin::BitSizeOf { of, span } => {
            let llvm_type: BasicTypeEnum = typegen::generate(context.get_llvm_context(), of);
            let bit_size: u64 = context.get_target_data().get_bit_size(&llvm_type);
            let size: BasicValueEnum = context
                .get_llvm_context()
                .i64_type()
                .const_int(bit_size, false)
                .into();

            cast::try_cast(context, cast_type, &Type::U64, size, span).unwrap_or(size)
        }
    }
}
