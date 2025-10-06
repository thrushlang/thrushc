use inkwell::values::BasicValueEnum;

use crate::{
    backends::classical::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, LLVMAllocationSite},
    },
    frontends::classical::{
        lexer::span::Span, types::parser::stmts::sites::AllocationSite, typesystem::types::Type,
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alloc_type: &Type,
    site: &AllocationSite,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let site: LLVMAllocationSite = site.to_llvm_allocation_site();
    memory::alloc_anon(context, site, alloc_type, span).into()
}
