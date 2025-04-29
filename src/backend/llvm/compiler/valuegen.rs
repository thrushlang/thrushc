use std::rc::Rc;

use crate::backend::llvm::compiler::context::CodeGenContextPosition;
use crate::backend::llvm::compiler::memory::{self, MemoryManagement, SymbolAllocated};
use crate::backend::llvm::compiler::{binaryop, builtins, unaryop, utils};
use crate::middle::instruction::Instruction;
use crate::middle::statement::traits::AttributesExtensions;
use crate::middle::statement::{Function, ThrushAttributes};

use super::super::super::super::middle::types::Type;

use super::context::CodeGenContext;
use super::typegen;

use inkwell::module::{Linkage, Module};
use inkwell::types::BasicTypeEnum;

use inkwell::AddressSpace;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FloatValue, FunctionValue, GlobalValue,
    IntValue,
};
use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn alloc<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: &Type,
    alloc_in_heap: bool,
) -> PointerValue<'ctx> {
    let llvm_type: BasicTypeEnum = typegen::generate_subtype(context, kind);

    if alloc_in_heap {
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

pub fn generate_expression<'ctx>(
    expression: &'ctx Instruction,
    casting_target: &Type,
    context: &mut CodeGenContext<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let Instruction::NullPtr { .. } = expression {
        return llvm_context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Str(_, str, ..) = expression {
        return utils::build_str_constant(llvm_module, llvm_context, str).into();
    }

    if let Instruction::Float(kind, num, is_signed, ..) = expression {
        let mut float: FloatValue = float(llvm_builder, llvm_context, kind, *num, *is_signed);

        if let Some(casted_float) = utils::float_autocast(
            casting_target,
            kind,
            float.into(),
            llvm_builder,
            llvm_context,
        ) {
            float = casted_float.into_float_value();
        }

        return float.into();
    }

    if let Instruction::Integer(kind, num, is_signed, ..) = expression {
        let mut integer: IntValue = integer(llvm_context, kind, *num as u64, *is_signed);

        if let Some(casted_integer) = utils::integer_autocast(
            casting_target,
            kind,
            integer.into(),
            llvm_builder,
            llvm_context,
        ) {
            integer = casted_integer.into_int_value();
        }

        return integer.into();
    }

    if let Instruction::Char(_, byte, ..) = expression {
        return llvm_context.i8_type().const_int(*byte as u64, false).into();
    }

    if let Instruction::Boolean(_, bool, ..) = expression {
        return llvm_context
            .bool_type()
            .const_int(*bool as u64, false)
            .into();
    }

    if let Instruction::Write {
        write_to,
        write_type,
        write_value,
        ..
    } = expression
    {
        let write_reference: &str = write_to.0;

        let write_value: BasicValueEnum = generate_expression(write_value, write_type, context);

        if let Some(expression) = write_to.1.as_ref() {
            let compiled_expression: PointerValue =
                generate_expression(expression, &Type::Void, context).into_pointer_value();

            llvm_builder
                .build_store(compiled_expression, write_value)
                .unwrap();

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        let symbol: SymbolAllocated = context.get_allocated_symbol(write_reference);

        symbol.store(context, write_value);

        return llvm_context
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
        let carry_type_generated: BasicTypeEnum = typegen::generate_type(llvm_context, carry_type);

        if let Some(expression) = expression {
            let compiled_expression: PointerValue<'_> =
                generate_expression(expression, carry_type, context).into_pointer_value();

            return llvm_builder
                .build_load(carry_type_generated, compiled_expression, "")
                .unwrap();
        }

        return context.get_allocated_symbol(name).load(context);
    }

    if let Instruction::Address { name, indexes, .. } = expression {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        let mut compiled_indexes: Vec<IntValue> = Vec::with_capacity(10);

        indexes.iter().for_each(|indexe| {
            let mut compiled_indexe: BasicValueEnum =
                generate_expression(indexe, &Type::U32, context);

            if let Some(casted_index) = utils::integer_autocast(
                &Type::U32,
                indexe.get_type(),
                compiled_indexe,
                llvm_builder,
                llvm_context,
            ) {
                compiled_indexe = casted_index;
            }

            compiled_indexes.push(compiled_indexe.into_int_value());
        });

        return symbol
            .gep(llvm_context, llvm_builder, &compiled_indexes)
            .into();
    }

    if let Instruction::Property {
        name,
        indexes,
        kind,
        ..
    } = expression
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        let mut address: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, indexes[0].1);

        indexes.iter().skip(1).for_each(|indexe| {
            address = memory::gep_struct_from_ptr(
                llvm_builder,
                typegen::generate_type(llvm_context, &indexe.0),
                address,
                indexe.1,
            );
        });

        if kind.is_mut_type() && context.get_position().in_mutation() {
            return address.into();
        }

        if kind.is_mut_type() && casting_target.is_mut_type() && context.get_position().in_call() {
            return address.into();
        }

        return memory::load_anon(llvm_context, llvm_builder, kind, address);
    }

    if let Instruction::LocalRef { name, kind, .. } | Instruction::ConstRef { name, kind, .. } =
        expression
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if kind.is_mut_type() && casting_target.is_mut_type() && context.get_position().in_call() {
            return symbol.take();
        }

        return symbol.load(context);
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
                context,
            );
        }

        if binaryop_type.is_integer_type() {
            return binaryop::integer::integer_binaryop(
                (left, operator, right),
                casting_target,
                context,
            );
        }

        if binaryop_type.is_bool_type() {
            return binaryop::boolean::bool_binaryop(
                (left, operator, right),
                casting_target,
                context,
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
        return unaryop::unary_op(
            llvm_builder,
            llvm_context,
            (operator, kind, expression),
            context,
        );
    }

    if let Instruction::LocalMut {
        source,
        kind,
        target,
        ..
    } = expression
    {
        context.set_position(CodeGenContextPosition::Mutation);

        let source_name: &str = source.0;
        let source_expression: Option<&Rc<Instruction<'_>>> = source.1.as_ref();

        if let Some(expression) = source_expression {
            let compiled_source: BasicValueEnum = generate_expression(expression, kind, context);
            let compiled_target: BasicValueEnum = generate_expression(target, kind, context);

            memory::store_anon(
                llvm_builder,
                compiled_source.into_pointer_value(),
                compiled_target.load_maybe(target.get_type(), context),
            );

            context.set_position_irrelevant();

            return compiled_source;
        }

        let symbol: SymbolAllocated = context.get_allocated_symbol(source_name);

        let expression: BasicValueEnum = generate_expression(target, kind, context);

        symbol.store(context, expression.load_maybe(target.get_type(), context));

        context.set_position_irrelevant();

        return expression;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_args,
        kind: call_type,
        ..
    } = expression
    {
        if *call_name == "sizeof!" {
            return builtins::build_sizeof(
                llvm_context,
                (call_name, call_type, call_args),
                context,
            );
        }

        if *call_name == "is_signed!" {
            return builtins::build_is_signed(
                llvm_context,
                llvm_builder,
                (call_name, call_type, call_args),
                context,
            );
        }

        context.set_position(CodeGenContextPosition::Call);

        let function: Function = context.get_function(call_name);

        let llvm_function: FunctionValue = function.0;

        let target_function_arguments: &[Type] = function.1;
        let function_convention: u32 = function.2;

        let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

        call_args.iter().enumerate().for_each(|instruction| {
            let casting_target: &Type = target_function_arguments
                .get(instruction.0)
                .unwrap_or(&Type::Void);

            compiled_args.push(generate_expression(instruction.1, casting_target, context).into());
        });

        let call: CallSiteValue = llvm_builder
            .build_call(llvm_function, &compiled_args, "")
            .unwrap();

        call.set_call_convention(function_convention);

        if !call_type.is_void_type() {
            let return_value: BasicValueEnum = call.try_as_basic_value().unwrap_left();

            if call_type.is_mut_type()
                && casting_target.is_mut_type()
                && context.get_position().in_call()
            {
                context.set_position_irrelevant();
                return return_value;
            }

            if call_type.is_mut_type() {
                context.set_position_irrelevant();
                return call.try_as_basic_value().unwrap_left();
            }

            if return_value.is_pointer_value() {
                context.set_position_irrelevant();
                return memory::load_anon(
                    llvm_context,
                    llvm_builder,
                    call_type,
                    return_value.into_pointer_value(),
                );
            }

            context.set_position_irrelevant();
            return call.try_as_basic_value().unwrap_left();
        }

        context.set_position_irrelevant();

        return llvm_context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Return(kind, value) = expression {
        let default_return: PointerValue =
            llvm_context.ptr_type(AddressSpace::default()).const_null();

        if kind.is_void_type() {
            llvm_builder.build_return(None).unwrap();
            return default_return.into();
        }

        llvm_builder
            .build_return(Some(&generate_expression(value, kind, context)))
            .unwrap();

        return default_return.into();
    }

    if let Instruction::Group { expression, .. } = expression {
        return generate_expression(expression, casting_target, context);
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
