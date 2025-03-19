use {
    super::{
        super::{super::frontend::lexer::Type, instruction::Instruction},
        call,
        objects::CompilerObjects,
        utils,
    },
    inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::Module,
        values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
    },
};

pub fn build_expression<'ctx>(
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    instr: &'ctx Instruction,
    casting_target: Option<Type>,
    compiler_objects: &mut CompilerObjects<'ctx>,
) -> BasicValueEnum<'ctx> {
    if let Instruction::NullPtr = instr {
        return context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let Instruction::Str(str) = instr {
        return utils::build_string_constant(module, builder, context, str).into();
    }

    if let Instruction::Float(kind, num, is_signed) = instr {
        let mut float: FloatValue =
            utils::build_const_float(builder, context, kind, *num, *is_signed);

        if casting_target.is_some() {
            if let Some(casted_float) = utils::float_autocast(
                kind,
                &casting_target.unwrap(),
                None,
                float.into(),
                builder,
                context,
            ) {
                float = casted_float.into_float_value();
            }
        }

        return float.into();
    }

    if let Instruction::Integer(kind, num, is_signed) = instr {
        let mut integer: IntValue =
            utils::build_const_integer(context, kind, *num as u64, *is_signed);

        if casting_target.is_some() {
            if let Some(casted_integer) = utils::integer_autocast(
                kind,
                &casting_target.unwrap(),
                None,
                integer.into(),
                builder,
                context,
            ) {
                integer = casted_integer.into_int_value();
            }
        }

        return integer.into();
    }

    if let Instruction::Char(char) = instr {
        return context.i8_type().const_int(*char as u64, false).into();
    }

    if let Instruction::Boolean(bool) = instr {
        return context.bool_type().const_int(*bool as u64, false).into();
    }

    if let Instruction::LocalRef { name, kind, .. } = instr {
        let local: PointerValue = compiler_objects.get_local(name).unwrap();

        if kind.is_float_type() {
            return builder
                .build_load(
                    utils::type_float_to_llvm_float_type(context, kind),
                    local,
                    "",
                )
                .unwrap();
        }

        if kind.is_integer_type() || kind.is_bool_type() {
            return builder
                .build_load(utils::type_int_to_llvm_int_type(context, kind), local, "")
                .unwrap();
        }

        if *kind == Type::Str {
            return local.into();
        }

        if *kind == Type::Struct {
            return builder.build_load(local.get_type(), local, "").unwrap();
        }

        unreachable!()
    }

    if let Instruction::Call {
        name: call_name,
        args: call_arguments,
        kind: call_type,
        ..
    } = instr
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

    println!("{:?}", instr);

    unreachable!()
}
