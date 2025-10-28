use inkwell::context::Context;
use inkwell::targets::TargetData;
use inkwell::types::BasicTypeEnum;

use crate::backend::llvm;
use crate::backend::llvm::compiler::context::LLVMCodeGenContext;

use crate::frontend::typesystem::traits::LLVMTypeExtensions;
use crate::frontend::typesystem::types::Type;

impl LLVMTypeExtensions for Type {
    #[inline]
    fn is_llvm_same_bit_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool {
        let llvm_context: &Context = context.get_llvm_context();

        let lhs: BasicTypeEnum = llvm::compiler::typegen::generate(llvm_context, self);
        let rhs: BasicTypeEnum = llvm::compiler::typegen::generate(llvm_context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&lhs) == target_data.get_bit_size(&rhs)
    }
}
