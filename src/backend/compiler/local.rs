use {
    super::{
        super::super::frontend::lexer::Type, Instruction, binaryop, call, generation,
        objects::CompilerObjects, types::Variable, unaryop, utils,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        types::StructType,
        values::{BasicValueEnum, PointerValue},
    },
};

pub fn build<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let local_type: &Type = variable.1;

    if *local_type == Type::Ptr {
        return build_local_ptr(
            module,
            builder,
            context,
            (variable.0, local_type, variable.2),
            compiler_objects,
        );
    }

    if *local_type == Type::Str {
        return build_local_static_str(
            module,
            builder,
            context,
            variable.0,
            variable.2,
            compiler_objects,
        );
    }

    if local_type.is_integer_type() {
        let ptr: PointerValue = utils::build_ptr(context, builder, *local_type);

        return build_local_integer(
            module,
            builder,
            context,
            (variable.0, local_type, variable.2),
            ptr,
            compiler_objects,
        );
    }

    if local_type.is_float_type() {
        let ptr: PointerValue = utils::build_ptr(context, builder, *local_type);

        return build_local_float(
            module,
            builder,
            context,
            (variable.0, local_type, variable.2),
            ptr,
            compiler_objects,
        );
    }

    if *local_type == Type::Bool {
        let ptr: PointerValue = utils::build_ptr(context, builder, *local_type);
        return build_local_boolean(module, builder, context, variable, ptr, compiler_objects);
    }

    if *local_type == Type::Struct {
        let ptr: PointerValue =
            utils::build_struct_ptr(context, builder, variable.2, compiler_objects);

        return build_local_structure(
            module,
            builder,
            context,
            (variable.0, local_type, variable.2),
            compiler_objects,
            ptr,
        );
    }

    unreachable!()
}

pub fn build_local_mut<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    compiler_objects: &mut CompilerObjects<'ctx>,
    variable: Variable<'ctx>,
) {
    let local_name: &str = variable.0;
    let local_type: &Type = variable.1;

    let variable_ptr: PointerValue<'ctx> = compiler_objects.get_local(local_name).unwrap();

    if local_type.is_integer_type() {
        build_local_integer(
            module,
            builder,
            context,
            variable,
            variable_ptr,
            compiler_objects,
        );
    }

    if local_type.is_float_type() {
        build_local_float(
            module,
            builder,
            context,
            variable,
            variable_ptr,
            compiler_objects,
        );
    }

    if *local_type == Type::Str {
        todo!()
    }
}

fn build_local_ptr<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let local_name: &str = variable.0;
    let local_value: &Instruction<'ctx> = variable.2;

    if let Instruction::NullPtr = local_value {
        let compiled_str: PointerValue = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            None,
            compiler_objects,
        )
        .into_pointer_value();

        compiler_objects.insert(local_name, compiled_str);
        return compiled_str.into();
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = local_value
    {
        let compiled_call: PointerValue = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        compiler_objects.insert(local_name, compiled_call);
        return compiled_call.into();
    }

    if let Instruction::Str(_) = local_value {
        let compiled_str: PointerValue = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            None,
            compiler_objects,
        )
        .into_pointer_value();

        compiler_objects.insert(local_name, compiled_str);
        return compiled_str.into();
    }

    unreachable!()
}

fn build_local_structure<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
    ptr: PointerValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    let local_name: &str = variable.0;
    let local_value: &Instruction = variable.2;

    if let Instruction::InitStruct { fields, .. } = local_value {
        fields.iter().for_each(|field| {
            let compiled_field: BasicValueEnum = generation::build_expression(
                module,
                builder,
                context,
                &field.1,
                Some(field.2),
                compiler_objects,
            );

            let field_in_struct: PointerValue = builder
                .build_struct_gep(
                    local_value.build_struct_type(context, None, compiler_objects),
                    ptr,
                    field.3,
                    "",
                )
                .unwrap();

            builder
                .build_store(field_in_struct, compiled_field)
                .unwrap();
        });

        compiler_objects.insert(local_name, ptr);
        return ptr.into();
    }

    if let Instruction::LocalRef { name, .. } = local_value {
        let original_ptr: PointerValue = compiler_objects.get_local(name).unwrap();

        builder.build_store(ptr, original_ptr).unwrap();
        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        struct_type: struct_name,
    } = local_value
    {
        let compiled_value: PointerValue = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        builder.build_store(ptr, compiled_value).unwrap();

        let struct_type: StructType =
            local_value.build_struct_type(context, None, compiler_objects);

        if let Some(structure) = compiler_objects.get_struct(struct_name) {
            structure
                .iter()
                .filter(|field| field.1.is_heaped_ptr())
                .for_each(|field| {
                    let field_in_struct: PointerValue<'ctx> = builder
                        .build_struct_gep(struct_type, compiled_value, field.2, "")
                        .unwrap();

                    let loaded_field: PointerValue<'ctx> = builder
                        .build_load(field_in_struct.get_type(), field_in_struct, "")
                        .unwrap()
                        .into_pointer_value();

                    builder.build_free(loaded_field).unwrap();
                });
        };

        builder.build_free(compiled_value).unwrap();
        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    unreachable!()
}

fn build_local_static_str<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    name: &'ctx str,
    value: &'ctx Instruction<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::Str(_) = value {
        let compiled_str =
            generation::build_expression(module, builder, context, value, None, compiler_objects)
                .into_pointer_value();

        compiler_objects.insert(name, compiled_str);
        return compiled_str.into();
    }

    if let Instruction::LocalRef { .. } = value {
        let compiled_refvar: PointerValue =
            generation::build_expression(module, builder, context, value, None, compiler_objects)
                .into_pointer_value();

        compiler_objects.insert(name, compiled_refvar);
        return compiled_refvar.into();
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = value
    {
        let compiled_call: PointerValue = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        compiler_objects.insert(name, compiled_call);
        return compiled_call.into();
    }

    unreachable!()
}

fn build_local_integer<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let local_name: &str = variable.0;
    let local_type: &Type = variable.1;
    let local_value: &Instruction = variable.2;

    if let Instruction::Null = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, local_type, 0, false),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Char(byte) = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, local_type, *byte as u64, false),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Integer(_, num, is_signed) = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, local_type, *num as u64, *is_signed),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::LocalRef {
        name: reflocal_name,
        kind: reflocal_type,
        ..
    } = local_value
    {
        let var: PointerValue = compiler_objects.get_local(reflocal_name).unwrap();

        let load: BasicValueEnum = builder
            .build_load(
                utils::type_int_to_llvm_int_type(context, reflocal_type),
                var,
                "",
            )
            .unwrap();

        if utils::integer_autocast(reflocal_type, local_type, Some(ptr), load, builder, context)
            .is_none()
        {
            builder.build_store(ptr, load).unwrap();
        }

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let result: BasicValueEnum = binaryop::integer_binaryop(
            builder,
            context,
            (left, op, right),
            local_type,
            compiler_objects,
        );

        builder.build_store(ptr, result.into_int_value()).unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let result: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if utils::integer_autocast(call_type, local_type, Some(ptr), result, builder, context)
            .is_none()
        {
            builder.build_store(ptr, result).unwrap();
        };

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::UnaryOp {
        op, value, kind, ..
    } = local_value
    {
        let result =
            unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Group { instr, .. } = local_value {
        build_local_integer(
            module,
            builder,
            context,
            (local_name, local_type, instr),
            ptr,
            compiler_objects,
        );

        return ptr.into();
    }

    unimplemented!()
}

fn build_local_float<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let local_name: &str = variable.0;
    let local_type: &Type = variable.1;
    let local_value: &Instruction = variable.2;

    if let Instruction::Null = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_float(builder, context, local_type, 0.0, false),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Float(_, num, is_signed) = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_float(builder, context, local_type, *num, *is_signed),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::LocalRef {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = local_value
    {
        let var: PointerValue<'ctx> = compiler_objects.get_local(name_refvar).unwrap();

        let load = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, kind_refvar),
                var,
                "",
            )
            .unwrap();

        if utils::float_autocast(
            kind_refvar,
            local_type,
            Some(ptr),
            var.into(),
            builder,
            context,
        )
        .is_none()
        {
            builder.build_store(ptr, load).unwrap();
        }

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let result: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if utils::float_autocast(call_type, local_type, Some(ptr), result, builder, context)
            .is_none()
        {
            builder.build_store(ptr, result).unwrap();
        };

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let result: BasicValueEnum = binaryop::float_binaryop(
            builder,
            context,
            (left, op, right),
            local_type,
            compiler_objects,
        );

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Group { instr, .. } = local_value {
        build_local_float(
            module,
            builder,
            context,
            (local_name, local_type, instr),
            ptr,
            compiler_objects,
        );
    }

    unimplemented!()
}

fn build_local_boolean<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let local_name: &str = variable.0;
    let local_type: &Type = variable.1;
    let local_value: &Instruction<'ctx> = variable.2;

    if let Instruction::Null = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, local_type, 0, false),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Boolean(bool) = local_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, local_type, *bool as u64, false),
            )
            .unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::LocalRef {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = local_value
    {
        let reflocal_ptr: PointerValue = compiler_objects.get_local(name_refvar).unwrap();

        let load = builder
            .build_load(
                utils::type_float_to_llvm_float_type(context, kind_refvar),
                reflocal_ptr,
                "",
            )
            .unwrap();

        if utils::integer_autocast(
            kind_refvar,
            local_type,
            Some(ptr),
            reflocal_ptr.into(),
            builder,
            context,
        )
        .is_none()
        {
            builder.build_store(ptr, load).unwrap();
        }

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let result: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if utils::integer_autocast(call_type, local_type, Some(ptr), result, builder, context)
            .is_none()
        {
            builder.build_store(ptr, result).unwrap();
        };

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let result: BasicValueEnum = binaryop::bool_binaryop(
            builder,
            context,
            (left, op, right),
            local_type,
            compiler_objects,
        );

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(local_name, ptr);

        return ptr.into();
    }

    if let Instruction::Group { instr, .. } = local_value {
        build_local_boolean(
            module,
            builder,
            context,
            (local_name, local_type, instr),
            ptr,
            compiler_objects,
        );
    }

    unreachable!()
}
