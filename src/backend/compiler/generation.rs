use {
    super::{
        super::super::frontend::lexer::Type, binaryop, call, instruction::Instruction,
        memory::AllocatedObject, memory::MemoryFlag, objects::CompilerObjects,
        types::CompilerStructure, types::CompilerStructureFields, unaryop, utils,
    },
    inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::Module,
        types::StructType,
        values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
    },
};

pub fn build_expression<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    instruction: &'ctx Instruction,
    casting_target: &Type,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::NullT = instruction {
        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Str(const_str) = instruction {
        return utils::build_string_constant(module, builder, context, const_str).into();
    }

    if let Instruction::Float(kind, num, is_signed) = instruction {
        let mut float: FloatValue =
            utils::build_const_float(builder, context, kind, *num, *is_signed);

        if let Some(casted_float) =
            utils::float_autocast(kind, casting_target, None, float.into(), builder, context)
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

    if let Instruction::Char(byte) = instruction {
        return context.i8_type().const_int(*byte as u64, false).into();
    }

    if let Instruction::Boolean(bool) = instruction {
        return context.bool_type().const_int(*bool as u64, false).into();
    }

    if let Instruction::GEP { name, index } = instruction {
        let local: PointerValue = compiler_objects.get_allocated_object(name).ptr;

        let mut compiled_index: BasicValueEnum = build_expression(
            module,
            builder,
            context,
            index,
            &Type::U64,
            compiler_objects,
        );

        if let Some(casted_index) = utils::integer_autocast(
            &Type::U64,
            index.get_type(),
            None,
            compiled_index,
            builder,
            context,
        ) {
            compiled_index = casted_index;
        }

        return unsafe {
            builder
                .build_in_bounds_gep(
                    context.ptr_type(AddressSpace::default()),
                    local,
                    &[compiled_index.into_int_value()],
                    "",
                )
                .unwrap()
                .into()
        };
    }

    if let Instruction::LocalRef {
        name,
        kind,
        struct_type,
        ..
    } = instruction
    {
        let object: AllocatedObject = compiler_objects.get_allocated_object(name);

        if kind.is_float_type() {
            return object
                .load_from_memory(builder, utils::type_float_to_llvm_float_type(context, kind));
        }

        if kind.is_integer_type() || kind.is_bool_type() {
            return object
                .load_from_memory(builder, utils::type_int_to_llvm_int_type(context, kind));
        }

        if kind.is_str() {
            return object.load_from_memory(builder, context.i8_type());
        }

        if kind.is_struct_type() {
            let structure: &CompilerStructure = compiler_objects.get_struct(struct_type);
            let fields: &CompilerStructureFields = &structure.1;

            let llvm_structure_type: StructType =
                utils::build_struct_type_from_fields(context, fields);

            if object.has_flag(MemoryFlag::HeapAllocated) {
                return object.ptr.into();
            }

            return object.load_from_memory(builder, llvm_structure_type);
        }

        if kind.is_raw_ptr() {
            return object.load_from_memory(builder, context.ptr_type(AddressSpace::default()));
        }

        unreachable!()
    }

    if let Instruction::BinaryOp {
        left,
        op,
        right,
        kind: binary_op_type,
        ..
    } = instruction
    {
        if binary_op_type.is_float_type() {
            return binaryop::float::float_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        if binary_op_type.is_integer_type() {
            return binaryop::integer::compile_integer_binaryop(
                module,
                builder,
                context,
                (left, op, right),
                casting_target,
                compiler_objects,
            );
        }

        if binary_op_type.is_bool_type() {
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
        expression,
        kind,
        ..
    } = instruction
    {
        return unaryop::compile_unary_op(
            builder,
            context,
            (op, expression, kind),
            compiler_objects,
        );
    }

    if let Instruction::LocalMut { name, kind, value } = instruction {
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

    if let Instruction::Return(instruction, kind) = instruction {
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
                instruction,
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
