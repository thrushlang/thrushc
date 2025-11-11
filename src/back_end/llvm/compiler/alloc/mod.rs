pub mod atomic;
pub mod memheap;
pub mod memstack;
pub mod memstatic;

use inkwell::{context::Context, types::BasicTypeEnum, values::PointerValue};

use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::typegen;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;
use crate::front_end::typesystem::types::Type;

pub fn local_variable<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    ascii_name: &str,
    kind: &Type,
    value: Option<&Ast>,
    attributes: &ThrushAttributes<'ctx>,
    span: Span,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let llvm_type: BasicTypeEnum = typegen::generate_local(llvm_context, kind, value);

    let formatted_ascii_name: String = format!("local.{}", ascii_name);

    if attributes.has_heap_attr() {
        memheap::try_alloc_heap(context, llvm_type, &formatted_ascii_name, span)
    } else {
        memstack::try_alloc_stack(context, llvm_type, &formatted_ascii_name, span)
    }
}
