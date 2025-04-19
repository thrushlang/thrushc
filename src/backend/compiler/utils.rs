use super::super::super::frontend::lexer::Type;

use super::{
    instruction::Instruction,
    objects::CompilerObjects,
    traits::CompilerStructureFieldsExtensions,
    types::{Structure, StructureFields},
};

use inkwell::types::BasicType;
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{
        AnyTypeEnum, ArrayType, BasicMetadataTypeEnum, BasicTypeEnum, FloatType, FunctionType,
        IntType, StructType,
    },
    values::{BasicValueEnum, FloatValue, GlobalValue, IntValue, PointerValue},
};

#[inline]
pub fn type_int_to_llvm_int_type<'ctx>(context: &'ctx Context, kind: &Type) -> IntType<'ctx> {
    match kind {
        Type::S8 | Type::U8 | Type::Char => context.i8_type(),
        Type::S16 | Type::U16 => context.i16_type(),
        Type::S32 | Type::U32 => context.i32_type(),
        Type::S64 | Type::U64 => context.i64_type(),
        Type::Bool => context.bool_type(),
        _ => unreachable!(),
    }
}

#[inline]
pub fn type_float_to_llvm_float_type<'ctx>(context: &'ctx Context, kind: &Type) -> FloatType<'ctx> {
    match kind {
        Type::F32 => context.f32_type(),
        Type::F64 => context.f64_type(),
        _ => unreachable!(),
    }
}

#[inline]
fn build_alloca_int<'ctx>(builder: &Builder<'ctx>, kind: IntType<'ctx>) -> PointerValue<'ctx> {
    builder.build_alloca(kind, "").unwrap()
}

#[inline]
fn build_alloca_float<'ctx>(builder: &Builder<'ctx>, kind: FloatType<'ctx>) -> PointerValue<'ctx> {
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

pub fn type_to_function_type<'ctx>(
    context: &'ctx Context,
    compiler_objects: &CompilerObjects,
    kind: &Instruction,
    parameters: &[Instruction],
    ignore_args: bool,
) -> FunctionType<'ctx> {
    let mut parameters_types: Vec<BasicMetadataTypeEnum> = Vec::with_capacity(parameters.len());

    for parameter in parameters.iter() {
        if let Instruction::FunctionParameter { kind, .. } = parameter {
            let parameter_basic_type: &Type = kind.get_basic_type();

            if parameter_basic_type.is_struct_type() {
                let structure_type: &str = kind.get_structure_type();

                let structure: &Structure = compiler_objects.get_struct(structure_type);
                let fields: &StructureFields = &structure.1;

                parameters_types.push(
                    build_struct_type_from_compiler_objects(
                        context,
                        compiler_objects,
                        structure_type,
                        fields,
                    )
                    .into(),
                );

                continue;
            }

            parameters_types.push(type_to_basic_metadata_enum(context, parameter_basic_type));
        }
    }

    if let Instruction::ComplexType(kind, structure_name) = kind {
        return match kind {
            Type::S8 | Type::U8 | Type::Char => {
                context.i8_type().fn_type(&parameters_types, ignore_args)
            }
            Type::S16 | Type::U16 => context.i16_type().fn_type(&parameters_types, ignore_args),
            Type::S32 | Type::U32 => context.i32_type().fn_type(&parameters_types, ignore_args),
            Type::S64 | Type::U64 => context.i64_type().fn_type(&parameters_types, ignore_args),
            Type::Str | Type::T => context
                .ptr_type(AddressSpace::default())
                .fn_type(&parameters_types, ignore_args),

            Type::Struct => {
                let structure: &Structure = compiler_objects.get_struct(structure_name);
                let structure_fields: &StructureFields = &structure.1;

                build_struct_type_from_compiler_objects(
                    context,
                    compiler_objects,
                    structure_name,
                    structure_fields,
                )
                .fn_type(&parameters_types, ignore_args)
            }
            Type::Bool => context.bool_type().fn_type(&parameters_types, ignore_args),
            Type::F32 => context.f32_type().fn_type(&parameters_types, ignore_args),
            Type::F64 => context.f64_type().fn_type(&parameters_types, ignore_args),
            Type::Void => context.void_type().fn_type(&parameters_types, ignore_args),
        };
    }

    unreachable!()
}

pub fn type_to_basic_metadata_enum<'ctx>(
    context: &'ctx Context,
    kind: &Type,
) -> BasicMetadataTypeEnum<'ctx> {
    match kind {
        Type::Bool => context.bool_type().into(),
        Type::S8 | Type::U8 | Type::Char => context.i8_type().into(),
        Type::S16 | Type::U16 => context.i16_type().into(),
        Type::S32 | Type::U32 => context.i32_type().into(),
        Type::S64 | Type::U64 => context.i64_type().into(),
        Type::F32 => context.f32_type().into(),
        Type::F64 => context.f64_type().into(),
        Type::Str | Type::Struct | Type::T => context.ptr_type(AddressSpace::default()).into(),

        _ => unreachable!(),
    }
}

pub fn type_to_any_type_enum<'ctx>(context: &'ctx Context, kind: &Type) -> AnyTypeEnum<'ctx> {
    match kind {
        Type::Bool => context.bool_type().into(),
        Type::S8 | Type::U8 | Type::Char => context.i8_type().into(),
        Type::S16 | Type::U16 => context.i16_type().into(),
        Type::S32 | Type::U32 => context.i32_type().into(),
        Type::S64 | Type::U64 => context.i64_type().into(),
        Type::F32 => context.f32_type().into(),
        Type::F64 => context.f64_type().into(),
        Type::Str | Type::Struct | Type::T => context.ptr_type(AddressSpace::default()).into(),
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
    if target_type.is_bool_type() {
        return None;
    }

    if from_type == target_type {
        return None;
    }

    let cast: IntValue;

    if from_type != target_type && from.is_int_value() {
        cast = builder
            .build_int_cast_sign_flag(
                from.into_int_value(),
                type_int_to_llvm_int_type(context, target_type),
                true,
                "",
            )
            .unwrap()
    } else if from_type != target_type && from.is_pointer_value() {
        let load: IntValue = builder
            .build_load(
                type_int_to_llvm_int_type(context, from_type),
                from.into_pointer_value(),
                "",
            )
            .unwrap()
            .into_int_value();

        cast = builder
            .build_int_cast_sign_flag(
                load,
                type_int_to_llvm_int_type(context, target_type),
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
    kind: &Type,
    target: &Type,
    ptr: Option<PointerValue<'ctx>>,
    from: BasicValueEnum<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
) -> Option<BasicValueEnum<'ctx>> {
    if target.is_bool_type() {
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
                type_float_to_llvm_float_type(context, target),
                "",
            )
            .unwrap();
    } else if kind != target && from.is_pointer_value() {
        let load: FloatValue<'ctx> = builder
            .build_load(
                type_float_to_llvm_float_type(context, kind),
                from.into_pointer_value(),
                "",
            )
            .unwrap()
            .into_float_value();

        cast = builder
            .build_float_cast(load, type_float_to_llvm_float_type(context, target), "")
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
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    const_str: &'ctx [u8],
) -> PointerValue<'ctx> {
    let kind: ArrayType = context.i8_type().array_type(const_str.len() as u32 + 1);
    let global: GlobalValue = module.add_global(kind, Some(AddressSpace::default()), "");

    global.set_linkage(Linkage::LinkerPrivate);
    global.set_initializer(&context.const_string(const_str, true));
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

/* pub fn build_str<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    size: u32,
) -> PointerValue<'ctx> {
    builder
        .build_alloca(context.i8_type().array_type(size), "")
        .unwrap()
} */

pub fn build_ptr<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: &Type,
) -> PointerValue<'ctx> {
    match kind {
        kind if kind.is_integer_type() => {
            build_alloca_int(builder, type_int_to_llvm_int_type(context, kind))
        }
        Type::Bool => build_alloca_int(builder, context.bool_type()),
        Type::F64 | Type::F32 => {
            build_alloca_float(builder, type_float_to_llvm_float_type(context, kind))
        }
        _ => unreachable!(),
    }
}

pub fn build_struct_ptr<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    instruction: &Instruction<'ctx>,
    _objects: &CompilerObjects<'ctx>,
    alloc_in_stack: bool,
) -> PointerValue<'ctx> {
    let struct_type: StructType = instruction.build_struct_type(context, None, _objects);

    if alloc_in_stack {
        return builder.build_alloca(struct_type, "").unwrap();
    }

    builder.build_malloc(struct_type, "").unwrap()
}

pub fn build_struct_type_from_fields<'ctx>(
    context: &'ctx Context,
    fields: &StructureFields,
) -> StructType<'ctx> {
    let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(10);

    fields.iter().for_each(|field| match &field.1 {
        kind if kind.is_integer_type() || kind.is_bool_type() => {
            field_types.push(type_int_to_llvm_int_type(context, field.1.get_basic_type()).into());
        }

        kind if kind.is_float_type() => {
            field_types
                .push(type_float_to_llvm_float_type(context, field.1.get_basic_type()).into());
        }

        kind if kind.is_ptr_type() => {
            field_types.push(context.ptr_type(AddressSpace::default()).into());
        }

        _ => {}
    });

    context.struct_type(&field_types, false)
}

pub fn build_struct_type_from_compiler_objects<'ctx>(
    context: &'ctx Context,
    compiler_objects: &CompilerObjects,
    structure_name: &str,
    compiler_structure_fields: &StructureFields,
) -> BasicTypeEnum<'ctx> {
    if !compiler_structure_fields.contain_recursive_structure_type(compiler_objects, structure_name)
    {
        let structure_type: StructType =
            build_struct_type_from_fields(context, compiler_structure_fields);

        return structure_type.into();
    }

    context.ptr_type(AddressSpace::default()).into()
}
