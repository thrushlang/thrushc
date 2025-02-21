use {
    super::{
        super::{super::frontend::lexer::DataTypes, instruction::Instruction},
        objects::CompilerObjects,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::{Linkage, Module},
        types::{ArrayType, BasicMetadataTypeEnum, FloatType, FunctionType, IntType, StructType},
        values::{BasicValueEnum, FloatValue, GlobalValue, IntValue, PointerValue},
        AddressSpace,
    },
};

pub fn datatype_integer_to_llvm_type<'ctx>(
    context: &'ctx Context,
    kind: &DataTypes,
) -> IntType<'ctx> {
    match kind {
        DataTypes::I8 | DataTypes::Char => context.i8_type(),
        DataTypes::I16 => context.i16_type(),
        DataTypes::I32 => context.i32_type(),
        DataTypes::I64 => context.i64_type(),
        DataTypes::Bool => context.bool_type(),

        _ => unreachable!(),
    }
}

pub fn datatype_float_to_llvm_type<'ctx>(
    context: &'ctx Context,
    kind: &DataTypes,
) -> FloatType<'ctx> {
    match kind {
        DataTypes::F32 => context.f32_type(),
        DataTypes::F64 | DataTypes::Bool => context.f64_type(),
        _ => unreachable!(),
    }
}

pub fn build_const_float<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    kind: &'ctx DataTypes,
    num: f64,
    is_signed: bool,
) -> FloatValue<'ctx> {
    match kind {
        DataTypes::F32 if is_signed => builder
            .build_float_neg(context.f32_type().const_float(num), "")
            .unwrap(),
        DataTypes::F32 => context.f32_type().const_float(num),
        DataTypes::F64 if is_signed => builder
            .build_float_neg(context.f64_type().const_float(num), "")
            .unwrap(),
        DataTypes::F64 => context.f64_type().const_float(num),
        _ => unreachable!(),
    }
}

pub fn build_const_integer<'ctx>(
    context: &'ctx Context,
    kind: &'ctx DataTypes,
    num: u64,
    is_signed: bool,
) -> IntValue<'ctx> {
    match kind {
        DataTypes::Char => context.i8_type().const_int(num, is_signed).const_neg(),
        DataTypes::I8 if is_signed => context.i8_type().const_int(num, is_signed).const_neg(),
        DataTypes::I8 => context.i8_type().const_int(num, is_signed),
        DataTypes::I16 if is_signed => context.i16_type().const_int(num, is_signed).const_neg(),
        DataTypes::I16 => context.i16_type().const_int(num, is_signed),
        DataTypes::I32 if is_signed => context.i32_type().const_int(num, is_signed).const_neg(),
        DataTypes::I32 => context.i32_type().const_int(num, is_signed),
        DataTypes::I64 if is_signed => context.i64_type().const_int(num, is_signed).const_neg(),
        DataTypes::I64 => context.i64_type().const_int(num, is_signed),
        DataTypes::Bool => context.bool_type().const_int(num, false),
        _ => unreachable!(),
    }
}

pub fn build_alloca_int<'ctx>(builder: &Builder<'ctx>, kind: IntType<'ctx>) -> PointerValue<'ctx> {
    builder.build_alloca(kind, "").unwrap()
}

pub fn build_alloca_float<'ctx>(
    builder: &Builder<'ctx>,
    kind: FloatType<'ctx>,
) -> PointerValue<'ctx> {
    builder.build_alloca(kind, "").unwrap()
}

pub fn datatype_to_fn_type<'ctx>(
    context: &'ctx Context,
    kind: &DataTypes,
    params: &[Instruction<'_>],
) -> FunctionType<'ctx> {
    let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::with_capacity(params.len());

    params.iter().for_each(|param| match param {
        Instruction::Param { kind, .. } => {
            param_types.push(datatype_to_basicmetadata_type_enum(context, kind));
        }

        _ => unreachable!(),
    });

    match kind {
        DataTypes::I8 | DataTypes::Char => context.i8_type().fn_type(&param_types, true),
        DataTypes::I16 => context.i16_type().fn_type(&param_types, true),
        DataTypes::I32 => context.i32_type().fn_type(&param_types, true),
        DataTypes::I64 => context.i64_type().fn_type(&param_types, true),
        DataTypes::Str | DataTypes::Struct | DataTypes::Ptr => context
            .ptr_type(AddressSpace::default())
            .fn_type(&param_types, true),
        DataTypes::Bool => context.bool_type().fn_type(&param_types, true),
        DataTypes::F32 => context.f32_type().fn_type(&param_types, true),
        DataTypes::F64 => context.f64_type().fn_type(&param_types, true),
        DataTypes::Void => context.void_type().fn_type(&param_types, true),
    }
}

pub fn datatype_to_basicmetadata_type_enum<'ctx>(
    context: &'ctx Context,
    kind: &DataTypes,
) -> BasicMetadataTypeEnum<'ctx> {
    match kind {
        DataTypes::I8 => context.i8_type().into(),
        DataTypes::I16 => context.i16_type().into(),
        DataTypes::I32 => context.i32_type().into(),
        DataTypes::I64 => context.i64_type().into(),
        DataTypes::F32 => context.f32_type().into(),
        DataTypes::F64 => context.f64_type().into(),
        DataTypes::Str | DataTypes::Struct | DataTypes::Ptr => {
            context.ptr_type(AddressSpace::default()).into()
        }

        _ => unreachable!(),
    }
}

#[inline]
pub fn float_autocast<'ctx>(
    kind: &DataTypes,
    target: &DataTypes,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if *target == DataTypes::Bool && kind.is_float_type() {
        return None;
    }

    if kind == target {
        return None;
    }

    let cast: FloatValue<'ctx>;

    if kind != target && from.is_float_value() {
        cast = builder
            .build_float_cast(
                from.into_float_value(),
                datatype_float_to_llvm_type(context, target),
                "",
            )
            .unwrap();
    } else if kind != target && from.is_pointer_value() {
        let load: FloatValue<'ctx> = builder
            .build_load(
                datatype_float_to_llvm_type(context, kind),
                from.into_pointer_value(),
                "",
            )
            .unwrap()
            .into_float_value();

        cast = builder
            .build_float_cast(load, datatype_float_to_llvm_type(context, target), "")
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

#[inline]
pub fn integer_autocast<'ctx>(
    kind: &DataTypes,
    target: &DataTypes,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if *target == DataTypes::Bool && kind.is_integer_type() {
        return None;
    }

    if kind == target {
        return None;
    }

    let cast: IntValue<'ctx>;

    if kind != target && from.is_int_value() {
        cast = builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                datatype_integer_to_llvm_type(context, target),
                is_signed_integer(kind),
                "",
            )
            .unwrap()
    } else if kind != target && from.is_pointer_value() {
        let load: IntValue<'_> = builder
            .build_load(
                datatype_integer_to_llvm_type(context, kind),
                from.into_pointer_value(),
                "",
            )
            .unwrap()
            .into_int_value();

        cast = builder
            .build_int_cast_sign_flag(
                load,
                datatype_integer_to_llvm_type(context, target),
                is_signed_integer(kind),
                "",
            )
            .unwrap();
    }
    /* else if kind != target && from.is_struct_value() {
        let extracted_integer: BasicValueEnum<'ctx> = builder
            .build_extract_value(from.into_struct_value(), 0, "")
            .unwrap();

        return integer_autocast(kind, target, ptr, extracted_integer, builder, context);
    } */
    else {
        builder.build_store(ptr.unwrap(), from).unwrap();
        return None;
    }

    if ptr.is_none() {
        return Some(cast.into());
    }

    builder.build_store(ptr.unwrap(), cast).unwrap();

    Some(cast.into())
}

#[inline]
pub fn is_signed_integer(kind: &DataTypes) -> bool {
    matches!(
        kind,
        DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64
    )
}

pub fn build_string_constant<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    string: &str,
) -> PointerValue<'ctx> {
    let kind: ArrayType<'_> = context.i8_type().array_type(string.len() as u32 + 1);
    let global: GlobalValue<'_> = module.add_global(kind, Some(AddressSpace::default()), "");

    global.set_linkage(Linkage::LinkerPrivate);
    global.set_initializer(&context.const_string(string.as_ref(), true));
    global.set_constant(true);
    global.set_unnamed_addr(true);

    builder
        .build_pointer_cast(
            global.as_pointer_value(),
            context.ptr_type(AddressSpace::default()),
            "",
        )
        .unwrap()
}

pub fn build_ptr<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: DataTypes,
) -> PointerValue<'ctx> {
    match kind {
        kind if kind.is_integer_type() => {
            build_alloca_int(builder, datatype_integer_to_llvm_type(context, &kind))
        }
        DataTypes::Bool => build_alloca_int(builder, context.bool_type()),
        DataTypes::F64 | DataTypes::F32 => {
            build_alloca_float(builder, datatype_float_to_llvm_type(context, &kind))
        }

        _ => unreachable!(),
    }
}

pub fn build_struct_ptr<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    struct_instr: &Instruction<'ctx>,
    _objects: &mut CompilerObjects<'ctx>,
) -> PointerValue<'ctx> {
    let struct_type: StructType<'_> = struct_instr.build_struct_type(context, None, _objects);
    builder.build_malloc(struct_type, "").unwrap()
}
