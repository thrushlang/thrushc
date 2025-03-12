use {
    super::{
        super::super::frontend::lexer::DataTypes, binaryop, call, generation,
        objects::CompilerObjects, types::Variable, unaryop, utils, Instruction,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        types::StructType,
        values::{BasicValueEnum, PointerValue},
    },
};

pub fn compile<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let var_type: &DataTypes = variable.1;

    if *var_type == DataTypes::Ptr {
        return compile_ptr_var(
            module,
            builder,
            context,
            (variable.0, var_type, variable.2),
            compiler_objects,
        );
    }

    if *var_type == DataTypes::Str {
        return compile_str_var(
            module,
            builder,
            context,
            variable.0,
            variable.2,
            compiler_objects,
        );
    }

    if var_type.is_integer_type() {
        let ptr: PointerValue<'_> = utils::build_ptr(context, builder, *var_type);

        return compile_integer_var(
            module,
            builder,
            context,
            (variable.0, var_type, variable.2),
            ptr,
            compiler_objects,
        );
    }

    if var_type.is_float_type() {
        let ptr: PointerValue<'_> = utils::build_ptr(context, builder, *var_type);

        return compile_float_var(
            builder,
            context,
            (variable.0, var_type, variable.2),
            ptr,
            compiler_objects,
        );
    }

    if *var_type == DataTypes::Bool {
        let ptr: PointerValue<'_> = utils::build_ptr(context, builder, *var_type);

        return compile_boolean_var(builder, context, variable, ptr, compiler_objects);
    }

    if *var_type == DataTypes::Struct {
        let ptr: PointerValue<'_> =
            utils::build_struct_ptr(context, builder, variable.2, compiler_objects);

        return compile_struct_var(
            module,
            builder,
            context,
            (variable.0, var_type, variable.2),
            compiler_objects,
            ptr,
        );
    }

    unreachable!()
}

pub fn compile_mut<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    compiler_objects: &mut CompilerObjects<'ctx>,
    variable: Variable<'ctx>,
) {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;

    let variable_ptr: PointerValue<'ctx> = compiler_objects.get_local(var_name).unwrap();

    if var_type.is_integer_type() {
        compile_integer_var(
            module,
            builder,
            context,
            variable,
            variable_ptr,
            compiler_objects,
        );
    }

    if var_type.is_float_type() {
        compile_float_var(builder, context, variable, variable_ptr, compiler_objects);
    }

    if *var_type == DataTypes::Str {
        todo!()
    }
}

fn compile_ptr_var<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let var_name: &str = variable.0;
    let var_value: &Instruction<'ctx> = variable.2;

    if let Instruction::NullPtr = var_value {
        let compiled_str: PointerValue = generation::build_basic_value_enum(
            module,
            builder,
            context,
            var_value,
            None,
            compiler_objects,
        )
        .into_pointer_value();

        compiler_objects.insert(var_name, compiled_str);

        return compiled_str.into();
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = var_value
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

        compiler_objects.insert(var_name, compiled_call);
        return compiled_call.into();
    }

    if let Instruction::Str(_) = var_value {
        let compiled_str: PointerValue = generation::build_basic_value_enum(
            module,
            builder,
            context,
            var_value,
            None,
            compiler_objects,
        )
        .into_pointer_value();

        compiler_objects.insert(var_name, compiled_str);

        return compiled_str.into();
    }

    unreachable!()
}

fn compile_struct_var<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
    ptr: PointerValue<'ctx>,
) -> BasicValueEnum<'ctx> {
    let var_name: &str = variable.0;
    let var_value: &Instruction<'ctx> = variable.2;

    if let Instruction::InitStruct { fields, .. } = var_value {
        fields.iter().for_each(|field| {
            let compiled_field: BasicValueEnum = generation::build_basic_value_enum(
                module,
                builder,
                context,
                &field.1,
                Some(field.2),
                compiler_objects,
            );

            let field_in_struct: PointerValue<'ctx> = builder
                .build_struct_gep(
                    var_value.build_struct_type(context, None, compiler_objects),
                    ptr,
                    field.3,
                    "",
                )
                .unwrap();

            builder
                .build_store(field_in_struct, compiled_field)
                .unwrap();
        });

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::RefVar { name, .. } = var_value {
        let original_ptr: PointerValue<'ctx> = compiler_objects.get_local(name).unwrap();

        builder.build_store(ptr, original_ptr).unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        struct_type: struct_name,
    } = var_value
    {
        let compiled_value: PointerValue<'_> = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        builder.build_store(ptr, compiled_value).unwrap();

        let struct_type: StructType<'_> =
            var_value.build_struct_type(context, None, compiler_objects);

        if let Some(structure) = compiler_objects.get_struct(struct_name) {
            structure
                .iter()
                .filter(|field| field.1.is_ptr_heaped())
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

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    unreachable!()
}

fn compile_str_var<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    name: &'ctx str,
    value: &'ctx Instruction<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::Str(_) = value {
        let compiled_str = generation::build_basic_value_enum(
            module,
            builder,
            context,
            value,
            None,
            compiler_objects,
        )
        .into_pointer_value();

        compiler_objects.insert(name, compiled_str);

        return compiled_str.into();
    }

    if let Instruction::RefVar { .. } = value {
        let compiled_refvar = generation::build_basic_value_enum(
            module,
            builder,
            context,
            value,
            None,
            compiler_objects,
        )
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
        let compiled_call: PointerValue<'_> = call::build_call(
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

fn compile_integer_var<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;
    let var_value: &Instruction = variable.2;

    if let Instruction::Null = var_value {
        builder
            .build_store(ptr, utils::build_const_integer(context, var_type, 0, false))
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Char(byte) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, var_type, *byte as u64, false),
            )
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Integer(_, num, is_signed) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, var_type, *num as u64, *is_signed),
            )
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::RefVar {
        name: refvar_name,
        kind: refvar_type,
        ..
    } = var_value
    {
        let var: PointerValue<'ctx> = compiler_objects.get_local(refvar_name).unwrap();

        let load: BasicValueEnum<'_> = builder
            .build_load(
                utils::datatype_integer_to_llvm_type(context, refvar_type),
                var,
                "",
            )
            .unwrap();

        if utils::integer_autocast(refvar_type, var_type, Some(ptr), load, builder, context)
            .is_none()
        {
            builder.build_store(ptr, load).unwrap();
        }

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
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

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
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

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::UnaryOp { op, value, kind } = var_value {
        let result =
            unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
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

        return ptr.into();
    }

    unimplemented!()
}

fn compile_float_var<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;
    let var_value: &Instruction = variable.2;

    if let Instruction::Null = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_float(builder, context, var_type, 0.0, false),
            )
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Float(_, num, is_signed) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_float(builder, context, var_type, *num, *is_signed),
            )
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::RefVar {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = var_value
    {
        let var: PointerValue<'ctx> = compiler_objects.get_local(name_refvar).unwrap();

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

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = var_value
    {
        let result: BasicValueEnum<'_> = binaryop::float_binaryop(
            builder,
            context,
            (left, op, right),
            var_type,
            compiler_objects,
        );

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
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

    unimplemented!()
}

fn compile_boolean_var<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    variable: Variable<'ctx>,
    ptr: PointerValue<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    let var_name: &str = variable.0;
    let var_type: &DataTypes = variable.1;
    let var_value: &Instruction<'ctx> = variable.2;

    if let Instruction::Null = var_value {
        builder
            .build_store(ptr, utils::build_const_integer(context, var_type, 0, false))
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Boolean(bool) = var_value {
        builder
            .build_store(
                ptr,
                utils::build_const_integer(context, var_type, *bool as u64, false),
            )
            .unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::RefVar {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = var_value
    {
        let var: PointerValue<'ctx> = compiler_objects.get_local(name_refvar).unwrap();

        let load = builder
            .build_load(
                utils::datatype_float_to_llvm_type(context, kind_refvar),
                var,
                "",
            )
            .unwrap();

        if utils::integer_autocast(
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

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = var_value
    {
        let result: BasicValueEnum<'_> = binaryop::bool_binaryop(
            builder,
            context,
            (left, op, right),
            var_type,
            compiler_objects,
        );

        builder.build_store(ptr, result).unwrap();

        compiler_objects.insert(var_name, ptr);

        return ptr.into();
    }

    if let Instruction::Group { instr, .. } = var_value {
        compile_boolean_var(
            builder,
            context,
            (var_name, var_type, instr),
            ptr,
            compiler_objects,
        );
    }

    unreachable!()
}
