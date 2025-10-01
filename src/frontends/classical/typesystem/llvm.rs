use inkwell::{context::Context, targets::TargetData, types::BasicTypeEnum};

use crate::{
    backends::classical::llvm::{self, compiler::context::LLVMCodeGenContext},
    frontends::classical::typesystem::{traits::LLVMTypeExtensions, types::Type},
};

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
    fn llvm_is_ptr_type(&self) -> bool {
        matches!(
            self,
            Type::Ptr(..) | Type::Addr | Type::Array(..) | Type::Fn(..)
        )
    }

    #[inline]
    fn llvm_is_int_type(&self) -> bool {
        self.is_integer_type() || self.is_bool_type() || self.is_char_type()
    }

    #[inline]
    fn llvm_is_float_type(&self) -> bool {
        self.is_float_type()
    }
}
