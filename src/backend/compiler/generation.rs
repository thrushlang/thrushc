use super::super::super::frontend::lexer::Type;

use super::{binaryop, call, instruction::Instruction};

use super::{memory::AllocatedObject, objects::CompilerObjects, typegen, unaryop, utils};

use inkwell::types::BasicTypeEnum;
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
};

pub fn build_expression<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    instruction: &'ctx Instruction,
    casting_target: &Type,
    compiler_objects: &CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::Str(_, str) = instruction {
        return utils::build_str_constant(module, context, str).into();
    }

    if let Instruction::Float(kind, num, is_signed) = instruction {
        let mut float: FloatValue =
            utils::build_const_float(builder, context, kind, *num, *is_signed);

        if let Some(casted_float) =
            utils::float_autocast(casting_target, kind, None, float.into(), builder, context)
        {
            float = casted_float.into_float_value();
        }

        return float.into();
    }

    if let Instruction::Integer(kind, num, is_signed) = instruction {
        let mut integer: IntValue =
            utils::build_const_integer(context, kind, *num as u64, *is_signed);

        if let Some(casted_integer) =
            utils::integer_autocast(casting_target, kind, None, integer.into(), builder, context)
        {
            integer = casted_integer.into_int_value();
        }

        return integer.into();
    }

    if let Instruction::Char(_, byte) = instruction {
        return context.i8_type().const_int(*byte as u64, false).into();
    }

    if let Instruction::Boolean(_, bool) = instruction {
        return context.bool_type().const_int(*bool as u64, false).into();
    }

    if let Instruction::Write {
        write_to,
        write_type,
        write_value,
        ..
    } = instruction
    {
        let write_reference: &str = write_to.0;

        let write_value: BasicValueEnum = build_expression(
            module,
            builder,
            context,
            write_value,
            write_type,
            compiler_objects,
        );

        if let Some(expression) = write_to.1.as_ref() {
            let compiled_expression: PointerValue = build_expression(
                module,
                builder,
                context,
                expression,
                &Type::Void,
                compiler_objects,
            )
            .into_pointer_value();

            builder
                .build_store(compiled_expression, write_value)
                .unwrap();

            return context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        let object: AllocatedObject = compiler_objects.get_allocated_object(write_reference);

        object.build_store(builder, write_value);

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
    } = instruction
    {
        let carry_type_generated: BasicTypeEnum = typegen::generate_type(context, carry_type);

        if let Some(expression) = expression {
            let compiled_expression: PointerValue<'_> = build_expression(
                module,
                builder,
                context,
                expression,
                carry_type,
                compiler_objects,
            )
            .into_pointer_value();

            return builder
                .build_load(carry_type_generated, compiled_expression, "")
                .unwrap();
        }

        let local: AllocatedObject = compiler_objects.get_allocated_object(name);
        return local.load_from_memory(builder, carry_type_generated);
    }

    if let Instruction::Address {
        name,
        indexes,
        kind,
        ..
    } = instruction
    {
        let local: PointerValue = compiler_objects.get_allocated_object(name).ptr;

        let mut compiled_indexes: Vec<IntValue> = Vec::with_capacity(10);

        indexes.iter().for_each(|indexe| {
            let mut compiled_indexe: BasicValueEnum = build_expression(
                module,
                builder,
                context,
                indexe,
                &Type::U32,
                compiler_objects,
            );

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

        return unsafe {
            builder
                .build_in_bounds_gep(
                    typegen::generate_type(context, kind),
                    local,
                    &compiled_indexes,
                    "",
                )
                .unwrap()
                .into()
        };
    }

    if let Instruction::LocalRef {
        name,
        kind: ref_type,
        take,
        ..
    }
    | Instruction::ConstRef {
        name,
        kind: ref_type,
        take,
        ..
    } = instruction
    {
        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        if *take {
            return object.ptr.into();
        }

        let llvm_type: BasicTypeEnum = typegen::generate_type(context, ref_type);

        return object.load_from_memory(builder, llvm_type);
    }

    if let Instruction::BinaryOp {
        left,
        op,
        right,
        kind: binaryop_type,
        ..
    } = instruction
    {
        if binaryop_type.is_float_type() {
            return binaryop::float::float_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        if binaryop_type.is_integer_type() {
            return binaryop::integer::compile_integer_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        if binaryop_type.is_bool_type() {
            return binaryop::boolean::bool_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        println!("{:?}", instruction);

        unreachable!()
    }

    if let Instruction::UnaryOp {
        op,
        kind,
        expression,
        ..
    } = instruction
    {
        return unaryop::compile_unary_op(
            builder,
            context,
            (op, kind, expression),
            compiler_objects,
        );
    }

    if let Instruction::LocalMut {
        name, kind, value, ..
    } = instruction
    {
        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        let expression: BasicValueEnum =
            build_expression(module, builder, context, value, kind, compiler_objects);

        object.build_store(builder, expression);

        return expression;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = instruction
    {
        return call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap();
    }

    if let Instruction::Return(kind, value) = instruction {
        if kind.is_void_type() {
            builder.build_return(None).unwrap();

            return context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        builder
            .build_return(Some(&build_expression(
                module,
                builder,
                context,
                value,
                kind,
                compiler_objects,
            )))
            .unwrap();

        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    println!("{:?}", instruction);
    unreachable!()
}
