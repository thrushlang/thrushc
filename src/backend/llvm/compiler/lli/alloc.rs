use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, LLVMAllocationSite},
    },
    frontend::types::{
        lexer::{ThrushType, traits::ThrushTypePointerExtensions},
        parser::stmts::sites::LLIAllocationSite,
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    type_to_alloc: &ThrushType,
    site_allocation: &LLIAllocationSite,
) -> BasicValueEnum<'ctx> {
    let site: LLVMAllocationSite = site_allocation.to_llvm_allocation_site();

    memory::alloc_anon(site, context, type_to_alloc, type_to_alloc.is_all_ptr()).into()
}
