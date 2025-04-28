use super::super::super::super::middle::types::Type;

use super::typegen;

use inkwell::values::StructValue;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    values::{BasicValueEnum, GlobalValue},
};

pub fn integer_autocast<'ctx>(
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if target_type.is_bool_type()
        || target_type.is_void_type()
        || from_type == target_type
        || target_type.is_ptr_type()
    {
        return None;
    }

    Some(
        builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                typegen::type_int_to_llvm_int_type(context, target_type),
                true,
                "",
            )
            .unwrap()
            .into(),
    )
}

pub fn float_autocast<'ctx>(
    target_type: &Type,
    from_type: &Type,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if target_type.is_bool_type()
        || target_type.is_void_type()
        || from_type == target_type
        || target_type.is_ptr_type()
    {
        return None;
    }

    Some(
        builder
            .build_float_cast(
                from.into_float_value(),
                typegen::type_float_to_llvm_float_type(context, target_type),
                "",
            )
            .unwrap()
            .into(),
    )
}

pub fn build_str_constant<'ctx>(
    module: &Module<'ctx>,
    context: &'ctx Context,
    str: &'ctx [u8],
) -> StructValue<'ctx> {
    let fixed_str_size: u32 = if !str.is_empty() {
        str.len() as u32 + 1
    } else {
        str.len() as u32
    };

    let kind: ArrayType = context.i8_type().array_type(fixed_str_size);
    let global: GlobalValue = module.add_global(kind, Some(AddressSpace::default()), "");

    global.set_linkage(Linkage::LinkerPrivate);
    global.set_initializer(&context.const_string(str, true));
    global.set_constant(true);

    context.const_struct(
        &[
            global.as_pointer_value().into(),
            context
                .i64_type()
                .const_int(fixed_str_size as u64, false)
                .into(),
        ],
        false,
    )
}
