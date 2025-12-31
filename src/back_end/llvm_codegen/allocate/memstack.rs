use crate::back_end::llvm_codegen::abort;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::typegeneration;
use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use std::path::PathBuf;

#[inline]
pub fn try_alloc_at_stack<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    llvm_type: BasicTypeEnum<'ctx>,
    ascii_name: &str,
    span: Span,
) -> PointerValue<'ctx> {
    if let Ok(ptr) = context
        .get_llvm_builder()
        .build_alloca(llvm_type, ascii_name)
    {
        return ptr;
    }

    abort::abort_codegen(
        context,
        "Failed to allocate at stack!",
        span,
        PathBuf::from(file!()),
        line!(),
    );
}

pub fn local_variable<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    kind: &Type,
    value: Option<&Ast>,
    attributes: &ThrushAttributes,
    span: Span,
) -> PointerValue<'ctx> {
    let llvm_type: BasicTypeEnum = typegeneration::compile_local_type(context, kind, value);
    let name: String = format!("local.{}", ascii_name);

    context.mark_dbg_location(span);

    if attributes.has_heap_attr() {
        crate::back_end::llvm_codegen::allocate::memheap::try_alloc_at_heap(
            context, llvm_type, &name, span,
        )
    } else {
        crate::back_end::llvm_codegen::allocate::memstack::try_alloc_at_stack(
            context, llvm_type, &name, span,
        )
    }
}
