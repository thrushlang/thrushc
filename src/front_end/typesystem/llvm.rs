use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;

use crate::front_end::typesystem::traits::LLVMTypeExtensions;
use crate::front_end::typesystem::types::Type;

use inkwell::targets::TargetData;
use inkwell::types::BasicTypeEnum;

impl LLVMTypeExtensions for Type {
    #[inline]
    fn llvm_is_same_bit_size(
        &self,
        context: &mut LLVMCodeGenContext<'_, '_>,
        other: &Type,
    ) -> bool {
        let lhs: BasicTypeEnum = crate::back_end::llvm_codegen::typegen::generate(context, self);
        let rhs: BasicTypeEnum = crate::back_end::llvm_codegen::typegen::generate(context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&lhs) == target_data.get_bit_size(&rhs)
    }
}
