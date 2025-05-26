use std::rc::Rc;

use crate::backend::llvm::compiler::context::LLVMCodeGenContextPosition;
use crate::backend::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backend::llvm::compiler::{binaryop, builtins, cast, unaryop, utils};
use crate::standard::logging::{self, LoggingType};
use crate::types::backend::llvm::types::LLVMFunction;
use crate::types::frontend::lexer::types::ThrushType;
use crate::types::frontend::parser::stmts::stmt::ThrushStatement;
use crate::types::frontend::parser::stmts::traits::CompilerAttributesExtensions;
use crate::types::frontend::parser::stmts::types::CompilerAttributes;

use super::context::LLVMCodeGenContext;
use super::typegen;

use inkwell::module::{Linkage, Module};
use inkwell::targets::TargetData;
use inkwell::types::{BasicTypeEnum, PointerType};

use inkwell::AddressSpace;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, CallSiteValue, FloatValue, FunctionValue, GlobalValue,
    IntValue,
};
use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn alloc<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    kind: &ThrushType,
    heap: bool,
) -> PointerValue<'ctx> {
    let llvm_type: BasicTypeEnum = typegen::generate_subtype(context, kind);

    if heap {
        return builder.build_malloc(llvm_type, "").unwrap();
    }

    builder.build_alloca(llvm_type, "").unwrap()
}

pub fn integer<'ctx>(
    context: &'ctx Context,
    kind: &'ctx ThrushType,
    number: u64,
    signed: bool,
) -> IntValue<'ctx> {
    match kind {
        ThrushType::Char => context.i8_type().const_int(number, signed).const_neg(),
        ThrushType::S8 if signed => context.i8_type().const_int(number, signed).const_neg(),
        ThrushType::S8 => context.i8_type().const_int(number, signed),
        ThrushType::S16 if signed => context.i16_type().const_int(number, signed).const_neg(),
        ThrushType::S16 => context.i16_type().const_int(number, signed),
        ThrushType::S32 if signed => context.i32_type().const_int(number, signed).const_neg(),
        ThrushType::S32 => context.i32_type().const_int(number, signed),
        ThrushType::S64 if signed => context.i64_type().const_int(number, signed).const_neg(),
        ThrushType::S64 => context.i64_type().const_int(number, signed),
        ThrushType::U8 => context.i8_type().const_int(number, false),
        ThrushType::U16 => context.i16_type().const_int(number, false),
        ThrushType::U32 => context.i32_type().const_int(number, false),
        ThrushType::U64 => context.i64_type().const_int(number, false),
        ThrushType::Bool => context.bool_type().const_int(number, false),
        _ => unreachable!(),
    }
}

#[inline]
pub fn float<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    kind: &'ctx ThrushType,
    number: f64,
    signed: bool,
) -> FloatValue<'ctx> {
    match kind {
        ThrushType::F32 if signed => builder
            .build_float_neg(context.f32_type().const_float(number), "")
            .unwrap(),
        ThrushType::F32 => context.f32_type().const_float(number),
        ThrushType::F64 if signed => builder
            .build_float_neg(context.f64_type().const_float(number), "")
            .unwrap(),
        ThrushType::F64 => context.f64_type().const_float(number),
        _ => unreachable!(),
    }
}

pub fn build<'ctx>(
    expression: &'ctx ThrushStatement,
    cast_target: &ThrushType,
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
) -> BasicValueEnum<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if let ThrushStatement::NullPtr { .. } = expression {
        return llvm_context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let ThrushStatement::Str { bytes, .. } = expression {
        return utils::build_str_constant(llvm_module, llvm_context, bytes).into();
    }

    if let ThrushStatement::Float {
        kind,
        value,
        signed,
        ..
    } = expression
    {
        let mut float: FloatValue = float(llvm_builder, llvm_context, kind, *value, *signed);

        if let Some(casted_float) = cast::float(context, cast_target, kind, float.into()) {
            float = casted_float.into_float_value();
        }

        return float.into();
    }

    if let ThrushStatement::Integer {
        kind,
        value,
        signed,
        ..
    } = expression
    {
        let mut integer: IntValue = integer(llvm_context, kind, *value, *signed);

        if let Some(casted_integer) = cast::integer(context, cast_target, kind, integer.into()) {
            integer = casted_integer.into_int_value();
        }

        return integer.into();
    }

    if let ThrushStatement::Char { byte, .. } = expression {
        return llvm_context.i8_type().const_int(*byte, false).into();
    }

    if let ThrushStatement::Boolean { value, .. } = expression {
        return llvm_context.bool_type().const_int(*value, false).into();
    }

    if let ThrushStatement::Alloc {
        type_to_alloc,
        site_allocation,
        ..
    } = expression
    {
        let site_allocation: memory::LLVMAllocationSite = site_allocation.to_llvm_allocation_site();

        return memory::alloc_anon(site_allocation, context, type_to_alloc).into();
    }

    if let ThrushStatement::Write {
        write_to,
        write_type,
        write_value,
        ..
    } = expression
    {
        let write_value: BasicValueEnum = self::build(write_value, write_type, context);

        if let Some(expression) = write_to.1.as_ref() {
            let compiled_expression: PointerValue =
                self::build(expression, &ThrushType::Void, context).into_pointer_value();

            if let Ok(store) = llvm_builder.build_store(compiled_expression, write_value) {
                let target_data: &TargetData = &context.target_data;

                let preferred_memory_alignment: u32 =
                    target_data.get_preferred_alignment(&compiled_expression.get_type());

                let _ = store.set_alignment(preferred_memory_alignment);
            }

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        if let Some(ref_name) = write_to.0 {
            let symbol: SymbolAllocated = context.get_allocated_symbol(ref_name);

            symbol.store(context, write_value);

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }
    }

    if let ThrushStatement::CastPtr {
        from, cast_type, ..
    } = expression
    {
        let ptr: PointerValue = self::build(from, cast_type, context).into_pointer_value();

        let cast_type: PointerType =
            typegen::generate_type(llvm_context, cast_type).into_pointer_type();

        if let Ok(casted_ptr) = llvm_builder.build_pointer_cast(ptr, cast_type, "") {
            return casted_ptr.into();
        }

        logging::log(
            LoggingType::Panic,
            &format!("Pointer casting could not be completed from: '{}'.", from),
        );
    }

    if let ThrushStatement::Load { load, kind, .. } = expression {
        if let Some(expression) = &load.1 {
            let ptr: PointerValue = self::build(expression, kind, context).into_pointer_value();

            return memory::load_anon(context, kind, ptr);
        }

        if let Some(ref_name) = load.0 {
            let ptr: PointerValue = context
                .get_allocated_symbol(ref_name)
                .load(context)
                .into_pointer_value();

            return memory::load_anon(context, kind, ptr);
        }
    }

    if let ThrushStatement::Address { name, indexes, .. } = expression {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        let mut compiled_indexes: Vec<IntValue> = Vec::with_capacity(10);

        indexes.iter().for_each(|indexe| {
            let mut compiled_indexe: BasicValueEnum =
                self::build(indexe, &ThrushType::U32, context);

            if let Some(casted_index) = cast::integer(
                context,
                &ThrushType::U32,
                indexe.get_type_unwrapped(),
                compiled_indexe,
            ) {
                compiled_indexe = casted_index;
            }

            compiled_indexes.push(compiled_indexe.into_int_value());
        });

        return symbol
            .gep(llvm_context, llvm_builder, &compiled_indexes)
            .into();
    }

    if let ThrushStatement::Property {
        name,
        indexes,
        kind,
        ..
    } = expression
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        let last_indexe_position: u32 = indexes[0].1;

        let mut last_memory_calculation: PointerValue =
            symbol.gep_struct(llvm_context, llvm_builder, last_indexe_position);

        indexes.iter().skip(1).for_each(|indexe| {
            let llvm_indexe_type: BasicTypeEnum = typegen::generate_type(llvm_context, &indexe.0);
            let indexe_position: u32 = indexe.1;

            if let Ok(new_memory_calculation) = llvm_builder.build_struct_gep(
                llvm_indexe_type,
                last_memory_calculation,
                indexe_position,
                "",
            ) {
                last_memory_calculation = new_memory_calculation;
            }
        });

        if context.get_position().in_mutation() {
            return last_memory_calculation.into();
        }

        if context.get_position().in_call() && cast_target.is_mut_type() {
            return last_memory_calculation.into();
        }

        return memory::load_anon(context, kind, last_memory_calculation);
    }

    if let ThrushStatement::Reference {
        name,
        identificator,
        ..
    } = expression
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if cast_target.is_mut_type()
            && context.get_position().in_call()
            && !identificator.is_constant()
        {
            return symbol.take();
        }

        return symbol.load(context);
    }

    if let ThrushStatement::BinaryOp {
        left,
        operator,
        right,
        kind: binaryop_type,
        ..
    } = expression
    {
        if binaryop_type.is_float_type() {
            return binaryop::float::float_binaryop((left, operator, right), cast_target, context);
        }

        if binaryop_type.is_integer_type() {
            return binaryop::integer::integer_binaryop(
                (left, operator, right),
                cast_target,
                context,
            );
        }

        if binaryop_type.is_bool_type() {
            return binaryop::boolean::bool_binaryop((left, operator, right), cast_target, context);
        }

        println!("{:?}", expression);
        unreachable!()
    }

    if let ThrushStatement::UnaryOp {
        operator,
        kind,
        expression,
        ..
    } = expression
    {
        return unaryop::unary_op(context, (operator, kind, expression));
    }

    if let ThrushStatement::Mut {
        source,
        kind,
        value,
        ..
    } = expression
    {
        context.set_position(LLVMCodeGenContextPosition::Mutation);

        let source_name: Option<&str> = source.0;
        let source_expression: Option<&Rc<ThrushStatement<'_>>> = source.1.as_ref();

        let value_type: &ThrushType = value.get_type_unwrapped();

        if let Some(expression) = source_expression {
            let source: BasicValueEnum = build(expression, kind, context);
            let value: BasicValueEnum = build(value, kind, context);

            memory::store_anon(
                context,
                source.into_pointer_value(),
                memory::load_maybe(context, value_type, value),
            );

            context.set_position_irrelevant();

            return source;
        }

        if let Some(name) = source_name {
            let symbol: SymbolAllocated = context.get_allocated_symbol(name);

            let expr: BasicValueEnum = build(value, kind, context);

            symbol.store(context, memory::load_maybe(context, value_type, expr));

            context.set_position_irrelevant();

            return expr;
        }
    }

    if let ThrushStatement::Call {
        name,
        args: call_args,
        kind: call_type,
        ..
    } = expression
    {
        if *name == "sizeof!" {
            return builtins::build_sizeof(context, (name, call_type, call_args));
        }

        if *name == "is_signed!" {
            return builtins::build_is_signed(context, (name, call_type, call_args));
        }

        context.set_position(LLVMCodeGenContextPosition::Call);

        let previous_position: LLVMCodeGenContextPosition = context.get_previous_position();

        let function: LLVMFunction = context.get_function(name);
        let function_arguments_types: &[ThrushType] = function.1;
        let function_convention: u32 = function.2;

        let llvm_function: FunctionValue = function.0;

        let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(call_args.len());

        call_args.iter().enumerate().for_each(|arg| {
            let arg_position: usize = arg.0;
            let arg_expr: &ThrushStatement = arg.1;

            let cast_target: &ThrushType = function_arguments_types
                .get(arg_position)
                .unwrap_or(&ThrushType::Void);

            compiled_args.push(build(arg_expr, cast_target, context).into());
        });

        let call: CallSiteValue = llvm_builder
            .build_call(llvm_function, &compiled_args, "")
            .unwrap();

        call.set_call_convention(function_convention);

        if !call_type.is_void_type() {
            let llvm_context: &Context = context.get_llvm_context();
            let return_value: BasicValueEnum = call.try_as_basic_value().unwrap_left();

            if call_type.is_heap_allocated(llvm_context, &context.target_data) {
                context.add_scope_call((call_type, return_value));
            }

            if cast_target.is_mut_type() && context.get_position().in_call() {
                context.set_position_irrelevant();
                return return_value;
            }

            if call_type.is_heap_allocated(llvm_context, &context.target_data)
                && previous_position.in_local()
                || previous_position.in_call()
            {
                context.set_position_irrelevant();

                return memory::load_anon(context, call_type, return_value.into_pointer_value());
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

    if let ThrushStatement::Return {
        expression, kind, ..
    } = expression
    {
        let null: PointerValue = llvm_context.ptr_type(AddressSpace::default()).const_null();

        if expression.is_none() {
            llvm_builder.build_return(None).unwrap();

            return null.into();
        }

        if let Some(expression) = expression {
            llvm_builder
                .build_return(Some(&build(expression, kind, context)))
                .unwrap();
        }

        return null.into();
    }

    if let ThrushStatement::Group { expression, .. } = expression {
        return build(expression, cast_target, context);
    }

    println!("{:?}", expression);
    unreachable!()
}

pub fn alloc_constant<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    attributes: &'ctx CompilerAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let global: GlobalValue = module.add_global(llvm_type, Some(AddressSpace::default()), name);

    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate)
    }

    global.set_initializer(&llvm_value);
    global.set_constant(true);

    global.as_pointer_value()
}
