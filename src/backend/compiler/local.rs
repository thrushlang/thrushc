use {
    super::{
        super::super::frontend::lexer::Type,
        Instruction, binaryop, call, generation,
        memory::{AllocatedObject, MemoryFlag},
        objects::CompilerObjects,
        types::Local,
        unaryop, utils,
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
    local: Local<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_type: &Type = local.1;

    if local_type.is_raw_ptr() {
        build_local_ptr(
            module,
            builder,
            context,
            (local.0, local_type, local.2),
            compiler_objects,
        );

        return;
    }

    if local_type.is_str() {
        build_local_str(module, builder, context, local.0, local.2, compiler_objects);

        return;
    }

    if local_type.is_integer_type() {
        let allocated_pointer: PointerValue = utils::build_ptr(context, builder, *local_type);
        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &[MemoryFlag::StackAllocated]);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_integer(
            module,
            builder,
            context,
            (local.0, local_type, local.2),
            allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_float_type() {
        let allocated_pointer: PointerValue = utils::build_ptr(context, builder, *local_type);
        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &[MemoryFlag::StackAllocated]);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_float(
            module,
            builder,
            context,
            (local.0, local_type, local.2),
            allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_bool_type() {
        let allocated_pointer: PointerValue = utils::build_ptr(context, builder, *local_type);
        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &[MemoryFlag::StackAllocated]);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_boolean(
            module,
            builder,
            context,
            local,
            allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_struct_type() {
        let allocated_pointer: PointerValue =
            utils::build_struct_ptr(context, builder, local.2, compiler_objects);
        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &[MemoryFlag::HeapAllocated]);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_structure(
            module,
            builder,
            context,
            (local.0, local_type, local.2),
            compiler_objects,
            allocated_object,
        );

        return;
    }

    unreachable!()
}

pub fn build_local_mut<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    compiler_objects: &mut CompilerObjects<'ctx>,
    local: Local<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    let object: AllocatedObject = compiler_objects.get_allocated_object(local_name);

    if let Instruction::LocalMut { value, .. } = local_value {
        let expression: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            value,
            local_type,
            compiler_objects,
        );

        object.build_store(builder, expression);

        compiler_objects.alloc_local_object(local_name, object);

        return;
    }

    if local_type.is_integer_type() {
        build_local_integer(module, builder, context, local, object, compiler_objects);
        return;
    }

    if local_type.is_float_type() {
        build_local_float(module, builder, context, local, object, compiler_objects);
        return;
    }

    if local_type.is_bool_type() {
        build_local_boolean(module, builder, context, local, object, compiler_objects);
        return;
    }

    if local_type.is_raw_ptr() {
        build_local_ptr(module, builder, context, local, compiler_objects);
        return;
    }

    todo!()
}

fn build_local_ptr<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_name: &str = local.0;
    let local_value: &Instruction = local.2;

    if local_value.is_nullt() {
        let null: PointerValue = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        )
        .into_pointer_value();

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(null, &[MemoryFlag::StackAllocated]);

        compiler_objects.alloc_local_object(local_name, allocated_object);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = local_value
    {
        let call: PointerValue = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(call, &[MemoryFlag::HeapAllocated]);

        compiler_objects.alloc_local_object(local_name, allocated_object);

        return;
    }

    if local_value.is_gep() {
        let gep: PointerValue = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::U64,
            compiler_objects,
        )
        .into_pointer_value();

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(gep, &[MemoryFlag::StackAllocated]);

        compiler_objects.alloc_local_object(local_name, allocated_object);

        return;
    }

    if let Instruction::LocalRef { name, .. } = local_value {
        let refvar_object: AllocatedObject = compiler_objects.get_allocated_object(name);

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(refvar_object.ptr, &[MemoryFlag::HeapAllocated]);

        compiler_objects.alloc_local_object(local_name, allocated_object);

        return;
    }

    unreachable!()
}

fn build_local_structure<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
    object: AllocatedObject<'ctx>,
) {
    let local_value: &Instruction = local.2;

    if let Instruction::InitStruct { fields, .. } = local_value {
        fields.iter().for_each(|field| {
            let compiled_field: BasicValueEnum = generation::build_expression(
                module,
                builder,
                context,
                &field.1,
                &field.2,
                compiler_objects,
            );

            let field_in_struct: PointerValue = builder
                .build_struct_gep(
                    local_value.build_struct_type(context, None, compiler_objects),
                    object.ptr,
                    field.3,
                    "",
                )
                .unwrap();

            builder
                .build_store(field_in_struct, compiled_field)
                .unwrap();
        });

        return;
    }

    if let Instruction::LocalRef { name, .. } = local_value {
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(name);

        object.build_store(builder, localref_object.ptr);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        struct_type: struct_name,
    } = local_value
    {
        let value: PointerValue = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        object.build_store(builder, value);

        let struct_type: StructType =
            local_value.build_struct_type(context, None, compiler_objects);

        if let Some(structure) = compiler_objects.get_struct(struct_name) {
            structure
                .iter()
                .filter(|field| field.1.is_heaped_ptr())
                .for_each(|field| {
                    let field_in_struct: PointerValue<'ctx> = builder
                        .build_struct_gep(struct_type, value, field.2, "")
                        .unwrap();

                    let loaded_field: PointerValue<'ctx> = builder
                        .build_load(field_in_struct.get_type(), field_in_struct, "")
                        .unwrap()
                        .into_pointer_value();

                    builder.build_free(loaded_field).unwrap();
                });
        };

        builder.build_free(value).unwrap();

        return;
    }

    unreachable!()
}

fn build_local_str<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    name: &'ctx str,
    value: &'ctx Instruction<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    if let Instruction::Str(_) = value {
        let str: PointerValue = generation::build_expression(
            module,
            builder,
            context,
            value,
            &Type::Void,
            compiler_objects,
        )
        .into_pointer_value();

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(str, &[MemoryFlag::StaticAllocated]);

        compiler_objects.alloc_local_object(name, allocated_object);

        return;
    }

    if let Instruction::LocalRef { .. } = value {
        let refvar: PointerValue = generation::build_expression(
            module,
            builder,
            context,
            value,
            &Type::Void,
            compiler_objects,
        )
        .into_pointer_value();

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(refvar, &[MemoryFlag::StaticAllocated]);

        compiler_objects.alloc_local_object(name, allocated_object);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = value
    {
        let call: PointerValue = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap()
        .into_pointer_value();

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(call, &[MemoryFlag::StaticAllocated]);

        compiler_objects.alloc_local_object(name, allocated_object);

        return;
    }

    unreachable!()
}

fn build_local_integer<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    object: AllocatedObject<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::Null = local_value {
        object.build_store(
            builder,
            utils::build_const_integer(context, local_type, 0, false),
        );

        return;
    }

    if let Instruction::Char(byte) = local_value {
        object.build_store(
            builder,
            utils::build_const_integer(context, local_type, *byte as u64, false),
        );

        return;
    }

    if let Instruction::Integer(_, num, is_signed) = local_value {
        object.build_store(
            builder,
            utils::build_const_integer(context, local_type, *num as u64, *is_signed),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: reflocal_name,
        kind: reflocal_type,
        ..
    } = local_value
    {
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(reflocal_name);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            utils::type_int_to_llvm_int_type(context, reflocal_type),
        );

        if let Some(casted_value) =
            utils::integer_autocast(local_type, reflocal_type, None, value, builder, context)
        {
            value = casted_value;
        }

        object.build_store(builder, value);

        return;
    }

    if let Instruction::UnaryOp {
        op, value, kind, ..
    } = local_value
    {
        let expression: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let expression: BasicValueEnum = binaryop::integer::compile_integer_binaryop(
            module,
            builder,
            context,
            (left, op, right),
            local_type,
            compiler_objects,
        );

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let mut expression: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if let Some(casted_expression) =
            utils::integer_autocast(local_type, call_type, None, expression, builder, context)
        {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::Group { instr, .. } = local_value {
        build_local_integer(
            module,
            builder,
            context,
            (local_name, local_type, instr),
            object,
            compiler_objects,
        );

        return;
    }

    unimplemented!()
}

fn build_local_float<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    object: AllocatedObject<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::Null = local_value {
        object.build_store(
            builder,
            utils::build_const_float(builder, context, local_type, 0.0, false),
        );

        return;
    }

    if let Instruction::Float(_, num, is_signed) = local_value {
        object.build_store(
            builder,
            utils::build_const_float(builder, context, local_type, *num, *is_signed),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = local_value
    {
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(name_refvar);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            utils::type_float_to_llvm_float_type(context, kind_refvar),
        );

        if let Some(casted_value) =
            utils::float_autocast(kind_refvar, local_type, None, value, builder, context)
        {
            value = casted_value;
        }

        object.build_store(builder, value);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let mut expression: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if let Some(casted_expression) =
            utils::float_autocast(call_type, local_type, None, expression, builder, context)
        {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::UnaryOp {
        op, value, kind, ..
    } = local_value
    {
        let expression: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let expression: BasicValueEnum = binaryop::float::float_binaryop(
            module,
            builder,
            context,
            (left, op, right),
            local_type,
            compiler_objects,
        );

        object.build_store(builder, expression);

        compiler_objects.alloc_local_object(local_name, object);

        return;
    }

    if let Instruction::Group { instr, .. } = local_value {
        build_local_float(
            module,
            builder,
            context,
            (local_name, local_type, instr),
            object,
            compiler_objects,
        );
    }

    unimplemented!()
}

fn build_local_boolean<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    object: AllocatedObject<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::Null = local_value {
        object.build_store(
            builder,
            utils::build_const_integer(context, local_type, 0, false),
        );

        return;
    }

    if let Instruction::Boolean(bool) = local_value {
        object.build_store(
            builder,
            utils::build_const_integer(context, local_type, *bool as u64, false),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = local_value
    {
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(name_refvar);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            utils::type_float_to_llvm_float_type(context, kind_refvar),
        );

        if let Some(new_value) =
            utils::integer_autocast(local_type, kind_refvar, None, value, builder, context)
        {
            value = new_value;
        }

        object.build_store(builder, value);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let mut expression: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if let Some(casted_expression) =
            utils::integer_autocast(local_type, call_type, None, expression, builder, context)
        {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::UnaryOp {
        op, value, kind, ..
    } = local_value
    {
        let expression: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, (op, value, kind), compiler_objects);

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let expression: BasicValueEnum = binaryop::boolean::bool_binaryop(
            module,
            builder,
            context,
            (left, op, right),
            local_type,
            compiler_objects,
        );

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::Group { instr, .. } = local_value {
        build_local_boolean(
            module,
            builder,
            context,
            (local_name, local_type, instr),
            object,
            compiler_objects,
        );
    }

    unreachable!()
}
