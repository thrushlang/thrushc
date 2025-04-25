use super::super::super::frontend::lexer::Type;

use super::traits::AttributesExtensions;

use super::types::ThrushAttributes;

use super::typegen;

use inkwell::values::{BasicValue, StructValue};

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::ArrayType,
    types::BasicType,
    values::{BasicValueEnum, FloatValue, GlobalValue, IntValue, PointerValue},
};

#[inline]
pub fn build_const_float<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    kind: &'ctx Type,
    num: f64,
    is_signed: bool,
) -> FloatValue<'ctx> {
    match kind {
        Type::F32 if is_signed => builder
            .build_float_neg(context.f32_type().const_float(num), "")
            .unwrap(),
        Type::F32 => context.f32_type().const_float(num),
        Type::F64 if is_signed => builder
            .build_float_neg(context.f64_type().const_float(num), "")
            .unwrap(),
        Type::F64 => context.f64_type().const_float(num),
        _ => unreachable!(),
    }
}

pub fn build_const_integer<'ctx>(
    context: &'ctx Context,
    kind: &'ctx Type,
    num: u64,
    is_signed: bool,
) -> IntValue<'ctx> {
    match kind {
        Type::Char => context.i8_type().const_int(num, is_signed).const_neg(),
        Type::S8 if is_signed => context.i8_type().const_int(num, is_signed).const_neg(),
        Type::S8 => context.i8_type().const_int(num, is_signed),
        Type::S16 if is_signed => context.i16_type().const_int(num, is_signed).const_neg(),
        Type::S16 => context.i16_type().const_int(num, is_signed),
        Type::S32 if is_signed => context.i32_type().const_int(num, is_signed).const_neg(),
        Type::S32 => context.i32_type().const_int(num, is_signed),
        Type::S64 if is_signed => context.i64_type().const_int(num, is_signed).const_neg(),
        Type::S64 => context.i64_type().const_int(num, is_signed),
        Type::U8 => context.i8_type().const_int(num, false),
        Type::U16 => context.i16_type().const_int(num, false),
        Type::U32 => context.i32_type().const_int(num, false),
        Type::U64 => context.i64_type().const_int(num, false),
        Type::Bool => context.bool_type().const_int(num, false),
        _ => unreachable!(),
    }
}

pub fn integer_autocast<'ctx>(
    target_type: &Type,
    from_type: &Type,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if target_type.is_bool_type() || target_type.is_void_type() || from_type == target_type {
        return None;
    }

    let cast: IntValue;

    if from_type != target_type && from.is_int_value() {
        cast = builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                typegen::type_int_to_llvm_int_type(context, target_type),
                true,
                "",
            )
            .unwrap()
    } else if from_type != target_type && from.is_pointer_value() {
        let load: IntValue = builder
            .build_load(
                typegen::type_int_to_llvm_int_type(context, from_type),
                from.into_pointer_value(),
                "",
            )
            .unwrap()
            .into_int_value();

        cast = builder
            .build_int_cast_sign_flag(
                load,
                typegen::type_int_to_llvm_int_type(context, target_type),
                true,
                "",
            )
            .unwrap();
    } else {
        builder.build_store(ptr.unwrap(), from).unwrap();
        return None;
    }

    if ptr.is_none() {
        return Some(cast.into());
    }

    builder.build_store(ptr.unwrap(), cast).unwrap();

    Some(cast.into())
}

pub fn float_autocast<'ctx>(
    target_type: &Type,
    from_type: &Type,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if target_type.is_bool_type() || target_type.is_void_type() || from_type == target_type {
        return None;
    }

    let cast: FloatValue;

    if from_type != target_type && from.is_float_value() {
        cast = builder
            .build_float_cast(
                from.into_float_value(),
                typegen::type_float_to_llvm_float_type(context, target_type),
                "",
            )
            .unwrap();
    } else if from_type != target_type && from.is_pointer_value() {
        let load: FloatValue<'ctx> = builder
            .build_load(
                typegen::type_float_to_llvm_float_type(context, target_type),
                from.into_pointer_value(),
                "",
            )
            .unwrap()
            .into_float_value();

        cast = builder
            .build_float_cast(
                load,
                typegen::type_float_to_llvm_float_type(context, target_type),
                "",
            )
            .unwrap();
    } else {
        builder.build_store(ptr.unwrap(), from).unwrap();
        return None;
    }

    if ptr.is_none() {
        return Some(cast.into());
    }

    builder.build_store(ptr.unwrap(), cast).unwrap();

    Some(cast.into())
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

pub fn build_global_constant<'ctx, Type: BasicType<'ctx>, Value: BasicValue<'ctx>>(
    module: &Module<'ctx>,
    name: &str,
    llvm_type: Type,
    llvm_value: Value,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let global: GlobalValue = module.add_global(llvm_type, Some(AddressSpace::default()), name);

    if !attributes.contain_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate)
    }

    global.set_initializer(&llvm_value);
    global.set_constant(true);

    global.as_pointer_value()
}
