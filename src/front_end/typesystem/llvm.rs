use inkwell::context::Context;
use inkwell::intrinsics::Intrinsic;
use inkwell::targets::TargetData;
use inkwell::types::BasicTypeEnum;

use crate::back_end::llvm;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;

use crate::front_end::typesystem::traits::LLVMTypeExtensions;
use crate::front_end::typesystem::types::Type;

impl LLVMTypeExtensions for Type {
    #[inline]
    fn llvm_is_same_bit_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool {
        let llvm_context: &Context = context.get_llvm_context();

        let lhs: BasicTypeEnum = llvm::compiler::typegen::generate(llvm_context, self);
        let rhs: BasicTypeEnum = llvm::compiler::typegen::generate(llvm_context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&lhs) == target_data.get_bit_size(&rhs)
    }

    #[inline]
    fn llvm_is_intrinsic_available(&self, name: &str) -> bool {
        Intrinsic::find(name).is_some()
    }
}
