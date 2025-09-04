use inkwell::{context::Context, types::BasicTypeEnum, values::BasicValueEnum};

use crate::{
    backends::classical::llvm::compiler::{constants, context::LLVMCodeGenContext, typegen},
    frontends::classical::typesystem::{traits::LLVMTypeExtensions, types::Type},
};

pub mod bitcast;
pub mod numeric;
pub mod ptr;

pub fn try_one<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    value_type: &Type,
    cast: &Type,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    match (value_type, cast) {
        (from_ty, cast_ty) if from_ty.is_str_type() && cast_ty.is_ptr_type() => {
            let cast: BasicTypeEnum = typegen::generate_type(llvm_context, cast_ty);

            constants::casts::ptr::const_ptr_cast(context, value, cast)
        }

        (_, cast_ty) if cast_ty.is_ptr_type() || cast_ty.is_mut_type() => {
            let cast: BasicTypeEnum = typegen::generate_type(llvm_context, cast_ty);

            constants::casts::ptr::const_ptr_cast(context, value, cast)
        }

        (_, cast_ty) if cast_ty.is_numeric() => {
            if value_type.llvm_is_same_bit_size(context, cast_ty) {
                constants::casts::bitcast::const_numeric_bitcast_cast(context, value, cast)
            } else {
                let cast: BasicTypeEnum = typegen::generate_subtype_with_all(llvm_context, cast_ty);
                constants::casts::numeric::numeric_cast(
                    value,
                    cast,
                    value_type.is_signed_integer_type(),
                )
            }
        }

        _ => value,
    }
}
