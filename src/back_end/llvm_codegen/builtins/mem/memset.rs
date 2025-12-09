use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::codegen;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::refptr;
use crate::back_end::llvm_codegen::typegen;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstLLVMGetType;
use crate::front_end::typesystem::types::Type;

use std::path::PathBuf;

use inkwell::{
    builder::Builder,
    context::Context,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{BasicValueEnum, IntValue, PointerValue},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    destination: &'ctx Ast,
    new_size: &'ctx Ast,
    size: &'ctx Ast,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let dest: PointerValue =
        refptr::compile(context, destination, Some(&Type::Ptr(None))).into_pointer_value();

    let new_size: IntValue = codegen::compile(context, new_size, None).into_int_value();
    let size: IntValue = codegen::compile(context, size, None).into_int_value();

    let destination_type: &Type = destination.llvm_get_type(context);

    let target_data: &TargetData = context.get_target_data();

    let dest_type: BasicTypeEnum = typegen::generate(llvm_context, destination_type);

    let dest_alignment: u32 = target_data.get_preferred_alignment(&dest_type);

    llvm_builder
        .build_memset(dest, dest_alignment, new_size, size)
        .unwrap_or_else(|_| {
            abort::abort_codegen(
                context,
                "Failed to compile 'memset' builtin!",
                span,
                PathBuf::from(file!()),
                line!(),
            )
        })
        .into()
}
