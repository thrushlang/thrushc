use crate::backend::llvm::compiler::memory::SymbolAllocated;
use crate::backend::llvm::compiler::{binaryop, call, unaryop, utils};
use crate::middle::instruction::Instruction;
use crate::middle::statement::ThrushAttributes;
use crate::middle::statement::traits::AttributesExtensions;

use super::super::super::super::middle::types::Type;

use super::symbols::SymbolsTable;
use super::typegen;

use inkwell::module::{Linkage, Module};
use inkwell::types::BasicTypeEnum;

use inkwell::AddressSpace;
use inkwell::values::{BasicValueEnum, FloatValue, GlobalValue, IntValue};
use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn alloc<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: &Type,
    alloc_in_heap: bool,
) -> PointerValue<'ctx> {
    let llvm_type: BasicTypeEnum = typegen::generate_typed_pointer(context, kind);

    if !alloc_in_heap {
        return builder.build_alloca(llvm_type, "").unwrap();
    }

    builder.build_malloc(llvm_type, "").unwrap()
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

pub fn generate_expression<'ctx>(
    expression: &'ctx Instruction,
    casting_target: &Type,
    symbols: &SymbolsTable<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    let module: &Module = symbols.get_llvm_module();
    let context: &Context = symbols.get_llvm_context();
    let builder: &Builder = symbols.get_llvm_builder();

    if let Instruction::Str(_, str, ..) = expression {
        return utils::build_str_constant(module, context, str).into();
    }

    if let Instruction::Float(kind, num, is_signed, ..) = expression {
        let mut float: FloatValue = float(builder, context, kind, *num, *is_signed);

        if let Some(casted_float) =
            utils::float_autocast(casting_target, kind, None, float.into(), builder, context)
        {
            float = casted_float.into_float_value();
        }

        return float.into();
    }

    if let Instruction::Integer(kind, num, is_signed, ..) = expression {
        let mut integer: IntValue = integer(context, kind, *num as u64, *is_signed);

        if let Some(casted_integer) =
            utils::integer_autocast(casting_target, kind, None, integer.into(), builder, context)
        {
            integer = casted_integer.into_int_value();
        }

        return integer.into();
    }

    if let Instruction::Char(_, byte, ..) = expression {
        return context.i8_type().const_int(*byte as u64, false).into();
    }

    if let Instruction::Boolean(_, bool, ..) = expression {
        return context.bool_type().const_int(*bool as u64, false).into();
    }

    if let Instruction::Write {
        write_to,
        write_type,
        write_value,
        ..
    } = expression
    {
        let write_reference: &str = write_to.0;

        let write_value: BasicValueEnum = generate_expression(write_value, write_type, symbols);

        if let Some(expression) = write_to.1.as_ref() {
            let compiled_expression: PointerValue =
                generate_expression(expression, &Type::Void, symbols).into_pointer_value();

            builder
                .build_store(compiled_expression, write_value)
                .unwrap();

            return context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        let symbol: SymbolAllocated = symbols.get_allocated_symbol(write_reference);

        symbol.store(builder, write_value);

        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Carry {
        name,
        expression,
        carry_type,
        ..
    } = expression
    {
        let carry_type_generated: BasicTypeEnum = typegen::generate_type(context, carry_type);

        if let Some(expression) = expression {
            let compiled_expression: PointerValue<'_> =
                generate_expression(expression, carry_type, symbols).into_pointer_value();

            return builder
                .build_load(carry_type_generated, compiled_expression, "")
                .unwrap();
        }

        return symbols.get_allocated_symbol(name).load(context, builder);
    }

    if let Instruction::Address { name, indexes, .. } = expression {
        let symbol: SymbolAllocated = symbols.get_allocated_symbol(name);

        let mut compiled_indexes: Vec<IntValue> = Vec::with_capacity(10);

        indexes.iter().for_each(|indexe| {
            let mut compiled_indexe: BasicValueEnum =
                generate_expression(indexe, &Type::U32, symbols);

            if let Some(casted_index) = utils::integer_autocast(
                &Type::U32,
                indexe.get_type(),
                None,
                compiled_indexe,
                builder,
                context,
            ) {
                compiled_indexe = casted_index;
            }

            compiled_indexes.push(compiled_indexe.into_int_value());
        });

        return symbol.gep(context, builder, &compiled_indexes).into();
    }

    if let Instruction::LocalRef { name, take, .. } | Instruction::ConstRef { name, take, .. } =
        expression
    {
        let symbol: SymbolAllocated = symbols.get_allocated_symbol(name);

        if *take {
            return symbol.take();
        }

        return symbol.load(context, builder);
    }

    if let Instruction::BinaryOp {
        left,
        operator,
        right,
        kind: binaryop_type,
        ..
    } = expression
    {
        if binaryop_type.is_float_type() {
            return binaryop::float::float_binaryop(
                (left, operator, right),
                casting_target,
                symbols,
            );
        }

        if binaryop_type.is_integer_type() {
            return binaryop::integer::integer_binaryop(
                (left, operator, right),
                casting_target,
                symbols,
            );
        }

        if binaryop_type.is_bool_type() {
            return binaryop::boolean::bool_binaryop(
                (left, operator, right),
                casting_target,
                symbols,
            );
        }

        println!("{:?}", expression);

        unreachable!()
    }

    if let Instruction::UnaryOp {
        operator,
        kind,
        expression,
        ..
    } = expression
    {
        return unaryop::unary_op(builder, context, (operator, kind, expression), symbols);
    }

    if let Instruction::LocalMut {
        name, kind, value, ..
    } = expression
    {
        let symbol: SymbolAllocated = symbols.get_allocated_symbol(name);

        let expression: BasicValueEnum = generate_expression(value, kind, symbols);

        symbol.store(builder, expression);

        return expression;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = expression
    {
        return call::build_call((call_name, call_type, call_arguments), symbols).unwrap();
    }

    if let Instruction::Return(kind, value) = expression {
        if kind.is_void_type() {
            builder.build_return(None).unwrap();

            return context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        builder
            .build_return(Some(&generate_expression(value, kind, symbols)))
            .unwrap();

        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Group { expression, .. } = expression {
        return generate_expression(expression, casting_target, symbols);
    }

    println!("{:?}", expression);
    unreachable!()
}

pub fn alloc_constant<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
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
