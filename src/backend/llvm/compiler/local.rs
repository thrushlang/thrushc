use crate::middle::{
    statement::{Local, traits::MemoryFlagsBasics},
    types::Type,
};

use super::{
    Instruction, binaryop, call, generation, memory::AllocatedObject, objects::CompilerObjects,
    typegen, unaryop, utils, valuegen,
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
    let local_type: &Type = local.1;

    if local_type.is_ptr_type() {
        let allocated_pointer: PointerValue =
            valuegen::alloc(context, builder, local_type, local.3.is_stack_allocated());

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_ptr(
            module,
            builder,
            context,
            local,
            allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_str_type() {
        let allocated_pointer: PointerValue = valuegen::alloc(
            context,
            builder,
            local_type,
            local_type.is_stack_allocated(),
        );

        let allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

        compiler_objects.alloc_local_object(local.0, allocated_object);

        build_local_str(
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
            valuegen::alloc(context, builder, local.1, local.3.is_stack_allocated());

        let mut allocated_object: AllocatedObject =
            AllocatedObject::alloc(allocated_pointer, &local.3, local_type);

        build_local_structure(
            module,
            builder,
            context,
            local,
            &mut allocated_object,
            compiler_objects,
        );

        return;
    }

    if local_type.is_integer_type() {
        let allocated_pointer: PointerValue = valuegen::alloc(
            context,
            builder,
            local_type,
            local_type.is_stack_allocated(),
        );

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
        let allocated_pointer: PointerValue = valuegen::alloc(
            context,
            builder,
            local_type,
            local_type.is_stack_allocated(),
        );

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
        let allocated_pointer: PointerValue = valuegen::alloc(
            context,
            builder,
            local_type,
            local_type.is_stack_allocated(),
        );

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

    if local_type.is_ptr_type() {
        build_local_ptr(module, builder, context, local, object, compiler_objects);
        return;
    }

    todo!()
}

fn build_local_ptr<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    allocated_object: AllocatedObject<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_value: &Instruction = local.2;

    if local_value.is_null() {
        let null: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        );

        allocated_object.build_store(builder, null);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = local_value
    {
        let call: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap();

        allocated_object.build_store(builder, call);

        return;
    }

    if local_value.is_gep() {
        let gep: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        );

        allocated_object.build_store(builder, gep);

        return;
    }

    if local_value.is_carry() {
        let carry: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        );

        allocated_object.build_store(builder, carry);

        return;
    }

    if local_value.is_local_ref() {
        let reference: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        );

        allocated_object.build_store(builder, reference);

        return;
    }

    unreachable!()
}

fn build_local_structure<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    local: Local<'ctx>,
    allocated_object: &mut AllocatedObject<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_type: &Type = local.1;
    let local_value: &Instruction = local.2;

    if let Instruction::InitStruct { arguments, .. } = local_value {
        arguments.iter().for_each(|argument| {
            let argument_instruction: &Instruction = &argument.1;
            let argument_type: &Type = &argument.2;
            let argument_position: u32 = argument.3;

            let compiled_field: BasicValueEnum = generation::build_expression(
                module,
                builder,
                context,
                argument_instruction,
                argument_type,
                compiler_objects,
            );

            let field_in_struct: PointerValue = builder
                .build_struct_gep(
                    typegen::generate_type(context, local_type),
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
        let value: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, call_arguments),
            compiler_objects,
        )
        .unwrap();

        allocated_object.build_store(builder, value);

        compiler_objects.alloc_local_object(local.0, *allocated_object);

        return;
    }

    if local_value.is_gep() {
        let value: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            local_type,
            compiler_objects,
        );

        allocated_object.build_store(builder, value);

        compiler_objects.alloc_local_object(local.0, *allocated_object);

        return;
    }

    if local_value.is_carry() {
        let value: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            local_type,
            compiler_objects,
        );

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
    local: Local<'ctx>,
    allocated_object: AllocatedObject<'ctx>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) {
    let local_value: &Instruction = local.2;

    if local_value.is_str() {
        let str_compiled: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        );

        allocated_object.build_store(builder, str_compiled);

        return;
    }

    if let Instruction::LocalRef { .. } = local_value {
        let compiled_refvar: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Void,
            compiler_objects,
        );

        allocated_object.build_store(builder, compiled_refvar);

        return;
    }

    if let Instruction::Call {
        name: call_name,
        args,
        kind: call_type,
        ..
    } = local_value
    {
        let compiled_call: BasicValueEnum = call::build_call(
            module,
            builder,
            context,
            (call_name, call_type, args),
            compiler_objects,
        )
        .unwrap();

        allocated_object.build_store(builder, compiled_call);

        return;
    }

    if local_value.is_carry() {
        let value: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            &Type::Str,
            compiler_objects,
        );

        allocated_object.build_store(builder, value);
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
        object.build_store(builder, valuegen::integer(context, local_type, 0, false));
        return;
    }

    if let Instruction::Char(_, byte) = local_value {
        object.build_store(
            builder,
            valuegen::integer(context, local_type, *byte as u64, false),
        );
        return;
    }

    if let Instruction::Integer(_, number, is_signed) = local_value {
        object.build_store(
            builder,
            valuegen::integer(context, local_type, *number as u64, *is_signed),
        );
        return;
    }

    if let Instruction::LocalRef {
        name: ref_name,
        kind: ref_type,
        ..
    }
    | Instruction::ConstRef {
        name: ref_name,
        kind: ref_type,
        ..
    } = local_value
    {
        let target_type: &Type = local_type;
        let ref_object: AllocatedObject = compiler_objects.get_allocated_object(ref_name);

        let mut value: BasicValueEnum = ref_object.load_from_memory(
            builder,
            typegen::type_int_to_llvm_int_type(context, ref_type),
        );

        if let Some(casted_value) =
            utils::integer_autocast(target_type, ref_type, None, value, builder, context)
        {
            value = casted_value;
        }

        object.build_store(builder, value);

        return;
    }

    if let Instruction::UnaryOp {
        operator,
        kind,
        expression,
        ..
    } = local_value
    {
        let expression: BasicValueEnum = unaryop::unary_op(
            builder,
            context,
            (operator, kind, expression),
            compiler_objects,
        );

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left,
        operator,
        right,
        ..
    } = local_value
    {
        let expression: BasicValueEnum = binaryop::integer::integer_binaryop(
            module,
            builder,
            context,
            (left, operator, right),
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

    if local_value.is_carry() {
        let value: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            local_type,
            compiler_objects,
        );

        object.build_store(builder, value);
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
            valuegen::float(builder, context, local_type, 0.0, false),
        );

        return;
    }

    if let Instruction::Float(_, num, is_signed) = local_value {
        object.build_store(
            builder,
            valuegen::float(builder, context, local_type, *num, *is_signed),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: ref_name,
        kind: ref_type,
        ..
    }
    | Instruction::ConstRef {
        name: ref_name,
        kind: ref_type,
        ..
    } = local_value
    {
        let target_type: &Type = local_type;

        let ref_object: AllocatedObject = compiler_objects.get_allocated_object(ref_name);

        let mut value: BasicValueEnum = ref_object.load_from_memory(
            builder,
            typegen::type_float_to_llvm_float_type(context, ref_type),
        );

        if let Some(casted_value) =
            utils::float_autocast(target_type, ref_type, None, value, builder, context)
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
            utils::float_autocast(local_type, call_type, None, expression, builder, context)
        {
            expression = casted_expression;
        };

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::UnaryOp {
        operator,
        kind,
        expression,
        ..
    } = local_value
    {
        let expression: BasicValueEnum = unaryop::unary_op(
            builder,
            context,
            (operator, kind, expression),
            compiler_objects,
        );

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left,
        operator,
        right,
        ..
    } = local_value
    {
        let expression: BasicValueEnum = binaryop::float::float_binaryop(
            module,
            builder,
            context,
            (left, operator, right),
            local_type,
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

    if local_value.is_carry() {
        let value: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            local_type,
            compiler_objects,
        );

        object.build_store(builder, value);
        return;
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
        object.build_store(builder, valuegen::integer(context, local_type, 0, false));

        return;
    }

    if let Instruction::Boolean(_, bool) = local_value {
        object.build_store(
            builder,
            valuegen::integer(context, local_type, *bool as u64, false),
        );

        return;
    }

    if let Instruction::LocalRef {
        name: ref_name,
        kind: ref_type,
        ..
    } = local_value
    {
        let localref_object: AllocatedObject = compiler_objects.get_allocated_object(ref_name);

        let mut value: BasicValueEnum = localref_object.load_from_memory(
            builder,
            typegen::type_int_to_llvm_int_type(context, ref_type),
        );

        if let Some(new_value) =
            utils::integer_autocast(local_type, ref_type, None, value, builder, context)
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
        operator,
        kind,
        expression,
        ..
    } = local_value
    {
        let expression: BasicValueEnum = unaryop::unary_op(
            builder,
            context,
            (operator, kind, expression),
            compiler_objects,
        );

        object.build_store(builder, expression);

        return;
    }

    if let Instruction::BinaryOp {
        left,
        operator,
        right,
        ..
    } = local_value
    {
        let expression: BasicValueEnum = binaryop::boolean::bool_binaryop(
            module,
            builder,
            context,
            (left, operator, right),
            local_type,
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

    if local_value.is_carry() {
        let value: BasicValueEnum = generation::build_expression(
            module,
            builder,
            context,
            local_value,
            local_type,
            compiler_objects,
        );

        object.build_store(builder, value);
        return;
    }

    unreachable!()
}
