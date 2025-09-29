pub mod atomic;
pub mod memheap;
pub mod memstack;
pub mod memstatic;

use inkwell::{context::Context, types::BasicTypeEnum, values::PointerValue};

use crate::{
    backends::classical::llvm::compiler::{context::LLVMCodeGenContext, obfuscation, typegen},
    frontends::classical::{
        lexer::span::Span,
        types::parser::stmts::{traits::ThrushAttributesExtensions, types::ThrushAttributes},
        typesystem::types::Type,
    },
};

pub fn alloc<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    kind: &Type,
    attributes: &ThrushAttributes<'ctx>,
    span: Span,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    let formatted_name: String = format!(
        "{}.local.{}",
        obfuscation::generate_obfuscation_name(obfuscation::SHORT_RANGE_OBFUSCATION),
        ascii_name
    );

    match (attributes.has_heap_attr(), attributes.has_stack_attr()) {
        (true, _) => memheap::try_alloc_heap(context, llvm_type, &formatted_name, span),
        (_, true) => memstack::try_alloc_stack(context, llvm_type, &formatted_name, span),
        _ => memstack::try_alloc_stack(context, llvm_type, &formatted_name, span),
    }
}
