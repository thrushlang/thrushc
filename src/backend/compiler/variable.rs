use {
    super::{
        super::super::frontend::lexer::DataTypes, binaryop, call, codegen,
        objects::CompilerObjects, types::Variable, unaryop, utils, Instruction,
    },
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicValueEnum, FunctionValue, IntValue, PointerValue},
        AddressSpace, IntPredicate,
    },
};

pub fn compile<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let var_type: &DataTypes = variable.1;

    let ptr: PointerValue<'_> = utils::build_ptr(context, builder, *var_type);

    if var_type.is_integer() {
        compile_integer_var(
            module,
            builder,
            context,
            (variable.0, var_type, variable.2),
            ptr,
            compiler_objects,
        );
    }

    if var_type.is_float() {
        compile_float_var(
            builder,
            context,
            (variable.0, var_type, variable.2),
            ptr,
            compiler_objects,
        );
    }

    if *var_type == DataTypes::String {
        compile_string_var(
            module,
            builder,
            context,
            variable.0,
            variable.2,
            compiler_objects,
        );
    }
}

pub fn compile_mut<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    compiler_objects: &mut CompilerObjects<'ctx>,
    variable: Variable<'ctx>,
    function: FunctionValue<'ctx>,
) {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;
    let var_value: &Instruction<'ctx> = variable.2;

    let variable_ptr: PointerValue<'ctx> = compiler_objects.find_and_get(var_name).unwrap();

    if var_type.is_integer() {
        compile_integer_var(
            module,
            builder,
            context,
            variable,
            variable_ptr,
            compiler_objects,
        );
    }

    if var_type.is_float() {
        compile_float_var(builder, context, variable, variable_ptr, compiler_objects);
    }

    if *var_type == DataTypes::String {
        if let Instruction::String(str, _) = var_value {
            builder
                .build_call(
                    module.get_function("Vec.realloc").unwrap(),
                    &[
                        variable_ptr.into(),
                        context.i64_type().const_int(str.len() as u64, false).into(),
                        context.bool_type().const_int(1, true).into(),
                    ],
                    "",
                )
                .unwrap();

            // HACERLO CON UN LOOP EN EL FUTURO, PARA EMITIR MENOS INSTRUCCIONES

            str.as_bytes().iter().for_each(|byte| {
                builder
                    .build_call(
                        module.get_function("Vec.push_i8").unwrap(),
                        &[
                            variable_ptr.into(),
                            context.i8_type().const_int(*byte as u64, false).into(),
                        ],
                        "",
                    )
                    .unwrap();
            });
        }

        if let Instruction::RefVar {
            name: refvar_name, ..
        } = var_value
        {
            let string_from_mut: PointerValue<'_> =
                compiler_objects.find_and_get(refvar_name).unwrap();

            let new_size: IntValue<'_> = builder
                .build_call(
                    module.get_function("Vec.size").unwrap(),
                    &[string_from_mut.into()],
                    "",
                )
                .unwrap()
                .try_as_basic_value()
                .unwrap_left()
                .into_int_value();

            let alloca_idx: PointerValue<'ctx> =
                builder.build_alloca(context.i64_type(), "").unwrap();

            builder
                .build_store(alloca_idx, context.i64_type().const_zero())
                .unwrap();

            builder
                .build_call(
                    module.get_function("Vec.realloc").unwrap(),
                    &[
                        variable_ptr.into(),
                        new_size.into(),
                        context.bool_type().const_zero().into(),
                    ],
                    "",
                )
                .unwrap();

            let start_block: BasicBlock<'_> = context.append_basic_block(function, "");

            builder.build_unconditional_branch(start_block).unwrap();

            builder.position_at_end(start_block);

            let get_idx: IntValue<'_> = builder
                .build_load(context.i64_type(), alloca_idx, "")
                .unwrap()
                .into_int_value();

            let cmp: IntValue<'_> = builder
                .build_int_compare(IntPredicate::UGT, get_idx, new_size, "")
                .unwrap();

            let then_block: BasicBlock<'_> = context.append_basic_block(function, "");
            let else_block: BasicBlock<'_> = context.append_basic_block(function, "");

            builder
                .build_conditional_branch(cmp, then_block, else_block)
                .unwrap();

            builder.position_at_end(else_block);

            let get_idx: IntValue<'_> = builder
                .build_load(context.i64_type(), alloca_idx, "")
                .unwrap()
                .into_int_value();

            let char: IntValue<'_> = builder
                .build_call(
                    module.get_function("Vec.get_i8").unwrap(),
                    &[string_from_mut.into(), get_idx.into()],
                    "",
                )
                .unwrap()
                .try_as_basic_value()
                .unwrap_left()
                .into_int_value();

            let get_data: PointerValue<'ctx> = builder
                .build_struct_gep(
                    context.struct_type(
                        &[
                            context.i64_type().into(),                        // size
                            context.i64_type().into(),                        // capacity
                            context.i64_type().into(),                        // element_size
                            context.ptr_type(AddressSpace::default()).into(), // data
                            context.i8_type().into(),                         // type
                        ],
                        false,
                    ),
                    variable_ptr,
                    3,
                    "",
                )
                .unwrap();

            let data: PointerValue<'_> = builder
                .build_load(get_data.get_type(), get_data, "")
                .unwrap()
                .into_pointer_value();

            let get_space: PointerValue<'ctx> = unsafe {
                builder
                    .build_in_bounds_gep(context.i8_type(), data, &[get_idx], "")
                    .unwrap()
            };

            builder.build_store(get_space, char).unwrap();

            let get_idx: IntValue<'_> = builder
                .build_load(context.i64_type(), alloca_idx, "")
                .unwrap()
                .into_int_value();

            let new_idx: IntValue<'_> = builder
                .build_int_add(get_idx, context.i64_type().const_int(1, false), "")
                .unwrap();

            builder.build_store(alloca_idx, new_idx).unwrap();

            builder.build_unconditional_branch(start_block).unwrap();

            builder.position_at_end(then_block);
        }
    }
}

fn compile_string_var<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    name: &str,
    value: &'ctx Instruction<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let ptr: PointerValue<'_> = utils::build_ptr(context, builder, DataTypes::String);

    if let Instruction::Null = value {
        compiler_objects.insert(name.to_string(), ptr);
    }

    if let Instruction::String(_, _) = value {
        compiler_objects.insert(
            name.to_string(),
            codegen::compile_instr_as_basic_value_enum(
                module,
                builder,
                context,
                value,
                &[],
                true,
                compiler_objects,
            )
            .into_pointer_value(),
        );
    }

    if let Instruction::RefVar { .. } = value {
        compiler_objects.insert(
            name.to_string(),
            codegen::compile_instr_as_basic_value_enum(
                module,
                builder,
                context,
                value,
                &[],
                true,
                compiler_objects,
            )
            .into_pointer_value(),
        );
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
    } = value
    {
        compiler_objects.insert(
            name.to_string(),
            call::build_call(
                module,
                builder,
                context,
                (call_name, call_type, args),
                compiler_objects,
            )
            .unwrap()
            .into_pointer_value(),
        );
    }

    if let Instruction::BinaryOp {
        left,
        op,
        right,
        kind,
        ..
    } = value
    {
        todo!()

        /*compiler_objects.insert(
            name.to_string(),
            binaryop::compile_binary_op(
                module, builder, context, left, op, right, kind, compiler_objects, function,
            )
            .into_pointer_value(),
        ); */
    }
}

fn compile_integer_var<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;
    let var_value: &Instruction = variable.2;

    if let Instruction::Null = var_value {
        builder
            .build_store(ptr, utils::build_const_integer(context, var_type, 0, false))
            .unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::Boolean(bool) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, var_type, *bool as u64, false),
            )
            .unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::Char(byte) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, var_type, *byte as u64, false),
            )
            .unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::Indexe {
        origin: from,
        index,
        ..
    } = var_value
    {
        let var: PointerValue<'_> = compiler_objects.find_and_get(from).unwrap();

        let char: IntValue<'_> = builder
            .build_call(
                module.get_function("Vec.get_i8").unwrap(),
                &[
                    var.into(),
                    context.i64_type().const_int(*index, false).into(),
                ],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();

        builder.build_store(ptr, char).unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::Integer(_, num, is_signed) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, var_type, *num as u64, *is_signed),
            )
            .unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::RefVar {
        name: refvar_name,
        kind: kind_refvar,
        ..
    } = var_value
    {
        let var: PointerValue<'ctx> = compiler_objects.find_and_get(refvar_name).unwrap();

        let load: BasicValueEnum<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, kind_refvar),
                var,
                "",
            )
            .unwrap();

        if utils::integer_autocast(kind_refvar, var_type, Some(ptr), load, builder, context)
            .is_none()
        {
            builder.build_store(ptr, load).unwrap();
        }

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = var_value
    {
        let result: BasicValueEnum<'_> = binaryop::integer_binaryop(
            builder,
            context,
            (left, op, right),
            var_type,
            compiler_objects,
        );

        builder.build_store(ptr, result.into_int_value()).unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
    } = var_value
    {
        let result: BasicValueEnum<'_> = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if utils::integer_autocast(call_type, var_type, Some(ptr), result, builder, context)
            .is_none()
        {
            builder.build_store(ptr, result).unwrap();
        };

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::UnaryOp { op, value, kind } = var_value {
        let result =
            unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::Group { instr, .. } = var_value {
        compile_integer_var(
            module,
            builder,
            context,
            (var_name, var_type, instr),
            ptr,
            compiler_objects,
        );
    }
}

fn compile_float_var<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;
    let var_value: &Instruction = variable.2;

    if let Instruction::Null = var_value {
        builder
            .build_store(ptr, utils::build_const_float(context, var_type, 0.0))
            .unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);

        return;
    }

    if let Instruction::Float(_, num, _) = var_value {
        builder
            .build_store(ptr, utils::build_const_float(context, var_type, *num))
            .unwrap();

        compiler_objects.insert(var_name.to_string(), ptr);

        return;
    }

    if let Instruction::RefVar {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = var_value
    {
        let var: PointerValue<'ctx> = compiler_objects.find_and_get(name_refvar).unwrap();

        let load = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, kind_refvar),
                var,
                "",
            )
            .unwrap();

        if utils::float_autocast(
            kind_refvar,
            var_type,
            Some(ptr),
            var.into(),
            builder,
            context,
        )
        .is_none()
        {
            builder.build_store(ptr, load).unwrap();
        }

        compiler_objects.insert(var_name.to_string(), ptr);
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = var_value
    {
        todo!()

        /*let result: FloatValue<'_> = binaryop::compile_binary_op(
            module, builder, context, left, op, right, kind, compiler_objects, function,
        )
        .into_float_value();

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(name.to_string(), ptr);*/
    }

    if let Instruction::Group { instr, .. } = var_value {
        compile_float_var(
            builder,
            context,
            (var_name, var_type, instr),
            ptr,
            compiler_objects,
        );
    }
}
