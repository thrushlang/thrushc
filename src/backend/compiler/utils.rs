use {
    super::{
        super::{super::frontend::lexer::Type, instruction::Instruction},
        objects::CompilerObjects,
        types::Struct,
    },
    inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::{Linkage, Module},
        types::{
            ArrayType, BasicMetadataTypeEnum, BasicTypeEnum, FloatType, FunctionType, IntType,
            StructType,
        },
        values::{BasicValueEnum, FloatValue, GlobalValue, IntValue, PointerValue},
    },
};

#[inline]
pub fn datatype_integer_to_llvm_type<'ctx>(context: &'ctx Context, kind: &Type) -> IntType<'ctx> {
    match kind {
        Type::I8 | Type::Char => context.i8_type(),
        Type::I16 => context.i16_type(),
        Type::I32 => context.i32_type(),
        Type::I64 => context.i64_type(),
        Type::Bool => context.bool_type(),
        _ => unreachable!(),
    }
}

#[inline]
pub fn datatype_float_to_llvm_type<'ctx>(context: &'ctx Context, kind: &Type) -> FloatType<'ctx> {
    match kind {
        Type::F32 => context.f32_type(),
        Type::F64 | Type::Bool => context.f64_type(),
        _ => unreachable!(),
    }
}

#[inline]
pub fn build_alloca_int<'ctx>(builder: &Builder<'ctx>, kind: IntType<'ctx>) -> PointerValue<'ctx> {
    builder.build_alloca(kind, "").unwrap()
}

#[inline]
pub fn build_alloca_float<'ctx>(
    builder: &Builder<'ctx>,
    kind: FloatType<'ctx>,
) -> PointerValue<'ctx> {
    builder.build_alloca(kind, "").unwrap()
}

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
        Type::I8 if is_signed => context.i8_type().const_int(num, is_signed).const_neg(),
        Type::I8 => context.i8_type().const_int(num, is_signed),
        Type::I16 if is_signed => context.i16_type().const_int(num, is_signed).const_neg(),
        Type::I16 => context.i16_type().const_int(num, is_signed),
        Type::I32 if is_signed => context.i32_type().const_int(num, is_signed).const_neg(),
        Type::I32 => context.i32_type().const_int(num, is_signed),
        Type::I64 if is_signed => context.i64_type().const_int(num, is_signed).const_neg(),
        Type::I64 => context.i64_type().const_int(num, is_signed),
        Type::Bool => context.bool_type().const_int(num, false),
        _ => unreachable!(),
    }
}

pub fn datatype_to_fn_type<'ctx>(
    context: &'ctx Context,
    kind: &Type,
    params: &[Instruction],
) -> FunctionType<'ctx> {
    let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::with_capacity(params.len());

    params.iter().for_each(|param| {
        if let Instruction::FunctionParameter { kind, .. } = param {
            param_types.push(datatype_to_basicmetadata_type_enum(context, kind));
        }
    });

    match kind {
        Type::I8 | Type::Char => context.i8_type().fn_type(&param_types, true),
        Type::I16 => context.i16_type().fn_type(&param_types, true),
        Type::I32 => context.i32_type().fn_type(&param_types, true),
        Type::I64 => context.i64_type().fn_type(&param_types, true),
        Type::Str | Type::Struct | Type::Ptr | Type::Generic => context
            .ptr_type(AddressSpace::default())
            .fn_type(&param_types, true),
        Type::Bool => context.bool_type().fn_type(&param_types, true),
        Type::F32 => context.f32_type().fn_type(&param_types, true),
        Type::F64 => context.f64_type().fn_type(&param_types, true),
        Type::Void => context.void_type().fn_type(&param_types, true),
    }
}

pub fn datatype_to_basicmetadata_type_enum<'ctx>(
    context: &'ctx Context,
    kind: &Type,
) -> BasicMetadataTypeEnum<'ctx> {
    match kind {
        Type::I8 => context.i8_type().into(),
        Type::I16 => context.i16_type().into(),
        Type::I32 => context.i32_type().into(),
        Type::I64 => context.i64_type().into(),
        Type::F32 => context.f32_type().into(),
        Type::F64 => context.f64_type().into(),
        Type::Str | Type::Struct | Type::Ptr => context.ptr_type(AddressSpace::default()).into(),

        _ => unreachable!(),
    }
}

#[inline]
pub fn float_autocast<'ctx>(
    kind: &Type,
    target: &Type,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if *target == Type::Bool && kind.is_float_type() {
        return None;
    }

    if kind == target {
        return None;
    }

    let cast: FloatValue;

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
    kind: &Type,
    target: &Type,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if *target == Type::Bool && kind.is_integer_type() {
        return None;
    }

    if kind == target {
        return None;
    }

    let cast: IntValue;

    if kind != target && from.is_int_value() {
        cast = builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                datatype_integer_to_llvm_type(context, target),
                true,
                "",
            )
            .unwrap()
    } else if kind != target && from.is_pointer_value() {
        let load: IntValue = builder
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

pub fn build_string_constant<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    string: &'ctx [u8],
) -> PointerValue<'ctx> {
    let kind: ArrayType = context.i8_type().array_type(string.len() as u32 + 1);
    let global: GlobalValue = module.add_global(kind, Some(AddressSpace::default()), "");

    global.set_linkage(Linkage::LinkerPrivate);
    global.set_initializer(&context.const_string(string, true));
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
    kind: Type,
) -> PointerValue<'ctx> {
    match kind {
        kind if kind.is_integer_type() => {
            build_alloca_int(builder, datatype_integer_to_llvm_type(context, &kind))
        }
        Type::Bool => build_alloca_int(builder, context.bool_type()),
        Type::F64 | Type::F32 => {
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
    let struct_type: StructType = struct_instr.build_struct_type(context, None, _objects);
    builder.build_malloc(struct_type, "").unwrap()
}

pub fn build_struct_type_from_fields<'ctx>(
    context: &'ctx Context,
    struct_fields: &Struct,
) -> StructType<'ctx> {
    let mut compiled_field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

    struct_fields.iter().for_each(|field| {
        if field.1.is_integer_type() {
            compiled_field_types.push(datatype_integer_to_llvm_type(context, &field.1).into());
        }

        if field.1.is_float_type() {
            compiled_field_types.push(datatype_float_to_llvm_type(context, &field.1).into());
        }

        if field.1.is_bool_type() {
            compiled_field_types.push(context.bool_type().into());
        }

        if field.1.is_ptr_type() {
            compiled_field_types.push(context.ptr_type(AddressSpace::default()).into());
        }
    });

    context.struct_type(&compiled_field_types, false)
}
