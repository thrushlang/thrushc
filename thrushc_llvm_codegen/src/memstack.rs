use inkwell::types::BasicTypeEnum;
use inkwell::values::PointerValue;
use thrushc_ast::Ast;
use thrushc_attributes::ThrushAttributes;
use thrushc_attributes::traits::ThrushAttributesExtensions;
use thrushc_span::Span;
use thrushc_typesystem::Type;

use std::path::PathBuf;

use crate::context::LLVMCodeGenContext;
use crate::{abort, memheap, typegeneration};

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
        memheap::try_alloc_at_heap(context, llvm_type, &name, span)
    } else {
        self::try_alloc_at_stack(context, llvm_type, &name, span)
    }
}

#[inline]
fn try_alloc_at_stack<'ctx>(
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
