use super::super::super::super::middle::types::Type;

use super::typegen;

use inkwell::types::BasicTypeEnum;

use inkwell::values::{FloatValue, IntValue};
use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn alloc<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: &Type,
    alloc_in_stack: bool,
) -> PointerValue<'ctx> {
    let llvm_type: BasicTypeEnum = typegen::generate_type(context, kind);

    if !alloc_in_stack {
        return builder.build_malloc(llvm_type, "").unwrap();
    }

    builder.build_alloca(llvm_type, "").unwrap()
}

pub fn integer<'ctx>(
    context: &'ctx Context,
    kind: &'ctx Type,
    number: u64,
    is_signed: bool,
) -> IntValue<'ctx> {
    match kind {
        Type::Char => context.i8_type().const_int(number, is_signed).const_neg(),
        Type::S8 if is_signed => context.i8_type().const_int(number, is_signed).const_neg(),
        Type::S8 => context.i8_type().const_int(number, is_signed),
        Type::S16 if is_signed => context.i16_type().const_int(number, is_signed).const_neg(),
        Type::S16 => context.i16_type().const_int(number, is_signed),
        Type::S32 if is_signed => context.i32_type().const_int(number, is_signed).const_neg(),
        Type::S32 => context.i32_type().const_int(number, is_signed),
        Type::S64 if is_signed => context.i64_type().const_int(number, is_signed).const_neg(),
        Type::S64 => context.i64_type().const_int(number, is_signed),
        Type::U8 => context.i8_type().const_int(number, false),
        Type::U16 => context.i16_type().const_int(number, false),
        Type::U32 => context.i32_type().const_int(number, false),
        Type::U64 => context.i64_type().const_int(number, false),
        Type::Bool => context.bool_type().const_int(number, false),
        _ => unreachable!(),
    }
}

#[inline]
pub fn float<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    kind: &'ctx Type,
    number: f64,
    is_signed: bool,
) -> FloatValue<'ctx> {
    match kind {
        Type::F32 if is_signed => builder
            .build_float_neg(context.f32_type().const_float(number), "")
            .unwrap(),
        Type::F32 => context.f32_type().const_float(number),
        Type::F64 if is_signed => builder
            .build_float_neg(context.f64_type().const_float(number), "")
            .unwrap(),
        Type::F64 => context.f64_type().const_float(number),
        _ => unreachable!(),
    }
}
