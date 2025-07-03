use inkwell::values::BasicValueEnum;

use crate::{
    backend::llvm::compiler::{
        context::LLVMCodeGenContext,
        memory::{self, LLVMAllocationSite},
    },
    frontend::types::{
        lexer::{Type, traits::TypePointerExtensions},
        parser::stmts::sites::AllocationSite,
    },
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    type_to_alloc: &Type,
    site_allocation: &AllocationSite,
) -> BasicValueEnum<'ctx> {
    let site: LLVMAllocationSite = site_allocation.to_llvm_allocation_site();

    memory::alloc_anon(site, context, type_to_alloc, type_to_alloc.is_all_ptr()).into()
}
