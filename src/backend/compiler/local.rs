use super::super::super::frontend::lexer::Type;

use super::{
    Instruction, binaryop, call, generation,
    memory::{AllocatedObject, MemoryFlag},
    objects::CompilerObjects,
    traits::MemoryFlagsBasics,
    types::Local,
    unaryop, utils,
};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{BasicValueEnum, PointerValue},
};

pub fn build<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_type: &Instruction = local.1;

    if local_type.is_raw_ptr_type() {
        build_local_ptr(module, builder, context, local, compiler_objects);
        return;
    }

    if local_type.is_str_type() {
        build_local_str(module, builder, context, local.0, local.2, compiler_objects);
        return;
    }

    if local_type.is_struct_type() {
        let allocated_pointer: PointerValue = utils::build_struct_ptr(
            context,
            builder,
            local.2,
            compiler_objects,
            local.3.is_stack_allocated(),
        );

        let mut allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

        build_local_structure(
            module,
            builder,
            context,
            local,
            compiler_objects,
            &mut allocated_object,
        );

        return;
    }

    if local_type.is_integer_type() {
        let local_basic_type: &Type = local_type.get_basic_type();

        let allocated_pointer: PointerValue = utils::build_ptr(context, builder, local_basic_type);

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_integer(
            module,
            builder,
            context,
            local,
            allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_float_type() {
        let local_basic_type: &Type = local_type.get_basic_type();

        let allocated_pointer: PointerValue = utils::build_ptr(context, builder, local_basic_type);

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_float(
            module,
            builder,
            context,
            local,
            allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_bool_type() {
        let local_basic_type: &Type = local_type.get_basic_type();

        let allocated_pointer: PointerValue = utils::build_ptr(context, builder, local_basic_type);
        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

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

    unreachable!()
}

pub fn build_local_mutation<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    compiler_objects: &mut CompilerObjects<'ctx>,
    local: Local<'ctx>,
) {
    let local_name: &str = local.0;
    let local_type: &Instruction = local.1;
    let local_value: &Instruction = local.2;

    let object: AllocatedObject = compiler_objects.get_allocated_object(local_name);

    if let Instruction::LocalMut { value, .. } = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        let expression: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            value,
            local_basic_type,
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

    if local_type.is_raw_ptr_type() {
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
    let local_type: &Instruction = local.1;
    let local_value: &Instruction = local.2;

    if local_value.is_null() {
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
            AllocatedObject::alloc(null, &[MemoryFlag::StackAllocated], local_type);

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
            AllocatedObject::alloc(call, &[MemoryFlag::HeapAllocated], local_type);

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
            AllocatedObject::alloc(gep, &[MemoryFlag::StackAllocated], local_type);

        compiler_objects.alloc_local_object(local_name, allocated_object);

        return;
    }

    if let Instruction::LocalRef { name, .. } = local_value {
        let refvar_object: AllocatedObject = compiler_objects.get_allocated_object(name);

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(refvar_object.ptr, &[MemoryFlag::HeapAllocated], local_type);

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
    allocated_object: &mut AllocatedObject<'ctx>,
) {
    let local_value: &Instruction = local.2;

    if let Instruction::InitStruct { arguments, .. } = local_value {
        arguments.iter().for_each(|argument| {
            let argument_instruction: &Instruction = &argument.1;
            let argument_basic_type: &Type = argument.2.get_basic_type();
            let argument_position: u32 = argument.3;

            let compiled_field: BasicValueEnum = generation::build_expression(
                module,
                builder,
                context,
                argument_instruction,
                argument_basic_type,
                compiler_objects,
            );

            let field_in_struct: PointerValue = builder
                .build_struct_gep(
                    local_value.build_struct_type(context, None, compiler_objects),
                    allocated_object.ptr,
                    argument_position,
                    "",
                )
                .unwrap();

            builder
                .build_store(field_in_struct, compiled_field)
                .unwrap();
        });

        compiler_objects.alloc_local_object(local.0, *allocated_object);

        return;
    }

    if let Instruction::LocalRef { name, .. } = local_value {
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(name);

        allocated_object.build_store(builder, localref_object.ptr);

        compiler_objects.alloc_local_object(local.0, *allocated_object);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
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

        allocated_object.build_store(builder, value);

        compiler_objects.alloc_local_object(local.0, *allocated_object);

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

        let allocated_object: AllocatedObject = AllocatedObject::alloc(
            str,
            &[MemoryFlag::StaticAllocated],
            &Instruction::Type(Type::Str, ""),
        );

        compiler_objects.alloc_local_object(name, allocated_object);

        return;
    }

    if let Instruction::LocalRef { kind, .. } = value {
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
            AllocatedObject::alloc(refvar, &[MemoryFlag::StaticAllocated], kind);

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

        let allocated_object: AllocatedObject = AllocatedObject::alloc(
            call,
            &[MemoryFlag::StaticAllocated],
            &Instruction::Type(Type::Str, ""),
        );

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
    let local_type: &Instruction = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::Null = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_integer(context, local_basic_type, 0, false),
        );

        return;
    }

    if let Instruction::Char(byte) = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_integer(context, local_basic_type, *byte as u64, false),
        );

        return;
    }

    if let Instruction::Integer(_, num, is_signed) = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_integer(context, local_basic_type, *num as u64, *is_signed),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: localref_name,
        kind: localref_type,
        ..
    } = local_value
    {
        let local_basic_type: &Type = local_type.get_basic_type();

        let localref_basic_type: &Type = localref_type.get_basic_type();
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(localref_name);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            utils::type_int_to_llvm_int_type(context, localref_basic_type),
        );

        if let Some(casted_value) = utils::integer_autocast(
            local_basic_type,
            localref_basic_type,
            None,
            value,
            builder,
            context,
        ) {
            value = casted_value;
        }

        object.build_store(builder, value);

        return;
    }

    if let Instruction::UnaryOp {
        op,
        expression,
        kind,
        ..
    } = local_value
    {
        let expression: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, (op, expression, kind), compiler_objects);

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let local_basic_type: &Type = local_type.get_basic_type();

        let expression: BasicValueEnum = binaryop::integer::compile_integer_binaryop(
            module,
            builder,
            context,
            (left, op, right),
            local_basic_type,
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
        let local_basic_type: &Type = local_type.get_basic_type();

        let mut expression: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if let Some(casted_expression) = utils::integer_autocast(
            local_basic_type,
            call_type.get_basic_type(),
            None,
            expression,
            builder,
            context,
        ) {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::Group { expression, .. } = local_value {
        build_local_integer(
            module,
            builder,
            context,
            (local_name, local_type, expression, local.3),
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
    let local_type: &Instruction = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::Null = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_float(builder, context, local_basic_type, 0.0, false),
        );

        return;
    }

    if let Instruction::Float(_, num, is_signed) = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_float(builder, context, local_basic_type, *num, *is_signed),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = local_value
    {
        let local_basic_type: &Type = local_type.get_basic_type();

        let kind_refvar: &Type = kind_refvar.get_basic_type();
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(name_refvar);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            utils::type_float_to_llvm_float_type(context, kind_refvar),
        );

        if let Some(casted_value) =
            utils::float_autocast(kind_refvar, local_basic_type, None, value, builder, context)
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
        let local_basic_type: &Type = local_type.get_basic_type();

        let mut expression: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if let Some(casted_expression) = utils::float_autocast(
            call_type.get_basic_type(),
            local_basic_type,
            None,
            expression,
            builder,
            context,
        ) {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::UnaryOp {
        op,
        expression,
        kind,
        ..
    } = local_value
    {
        let expression: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, (op, expression, kind), compiler_objects);

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let local_basic_type: &Type = local_type.get_basic_type();

        let expression: BasicValueEnum = binaryop::float::float_binaryop(
            module,
            builder,
            context,
            (left, op, right),
            local_basic_type,
            compiler_objects,
        );

        object.build_store(builder, expression);

        compiler_objects.alloc_local_object(local_name, object);

        return;
    }

    if let Instruction::Group { expression, .. } = local_value {
        build_local_float(
            module,
            builder,
            context,
            (local_name, local_type, expression, local.3),
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
    let local_type: &Instruction = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::Null = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_integer(context, local_basic_type, 0, false),
        );

        return;
    }

    if let Instruction::Boolean(bool) = local_value {
        let local_basic_type: &Type = local_type.get_basic_type();

        object.build_store(
            builder,
            utils::build_const_integer(context, local_basic_type, *bool as u64, false),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: name_refvar,
        kind: kind_refvar,
        ..
    } = local_value
    {
        let local_basic_type: &Type = local_type.get_basic_type();
        let refvar_basic_type: &Type = kind_refvar.get_basic_type();

        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(name_refvar);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            utils::type_float_to_llvm_float_type(context, refvar_basic_type),
        );

        if let Some(new_value) = utils::integer_autocast(
            local_basic_type,
            refvar_basic_type,
            None,
            value,
            builder,
            context,
        ) {
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
        let local_basic_type: &Type = local_type.get_basic_type();

        let mut expression: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        if let Some(casted_expression) = utils::integer_autocast(
            local_basic_type,
            call_type.get_basic_type(),
            None,
            expression,
            builder,
            context,
        ) {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::UnaryOp {
        op,
        expression,
        kind,
        ..
    } = local_value
    {
        let expression: BasicValueEnum =
            unaryop::compile_unary_op(builder, context, (op, expression, kind), compiler_objects);

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left, op, right, ..
    } = local_value
    {
        let local_basic_type: &Type = local_type.get_basic_type();

        let expression: BasicValueEnum = binaryop::boolean::bool_binaryop(
            module,
            builder,
            context,
            (left, op, right),
            local_basic_type,
            compiler_objects,
        );

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::Group { expression, .. } = local_value {
        build_local_boolean(
            module,
            builder,
            context,
            (local_name, local_type, expression, local.3),
            object,
            compiler_objects,
        );
    }

    unreachable!()
}
