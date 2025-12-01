use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::memory;
use crate::back_end::llvm_codegen::memory::LLVMAllocationSite;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::parser::stmts::sites::AllocationSite;
use crate::front_end::typesystem::types::Type;

use inkwell::values::BasicValueEnum;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    alloc_type: &Type,
    site: &AllocationSite,
    span: Span,
) -> BasicValueEnum<'ctx> {
    let site: LLVMAllocationSite = site.to_llvm_allocation_site();

    memory::alloc_anon(context, site, alloc_type, span).into()
}
