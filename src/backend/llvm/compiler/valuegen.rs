use crate::backend::llvm::compiler::attributes::LLVMAttribute;
use crate::backend::llvm::compiler::memory::{self, LLVMAllocationSite, SymbolAllocated};
use crate::backend::llvm::compiler::{binaryop, builtins, unaryop, utils, valuegen};
use crate::backend::types::representations::LLVMFunction;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::lexer::traits::{
    LLVMTypeExtensions, ThrushTypeMutableExtensions, ThrushTypePointerExtensions,
};
use crate::frontend::types::parser::stmts::stmt::ThrushStatement;
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;

use crate::backend::types::traits::AssemblerFunctionExtensions;

use super::context::LLVMCodeGenContext;
use super::typegen;

use inkwell::module::{Linkage, Module};
use inkwell::targets::TargetData;
use inkwell::types::{
    ArrayType, BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, PointerType, StructType,
};

use inkwell::values::{
    ArrayValue, BasicMetadataValueEnum, BasicValueEnum, FloatValue, FunctionValue, GlobalValue,
    IntValue, StructValue,
};

use inkwell::{AddressSpace, InlineAsmDialect};
use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn alloc<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let target_data: &TargetData = context.get_target_data();

    let llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, kind);

    if attributes.has_heap_attr() {
        if let Ok(allocated_heap_ptr) = llvm_builder.build_malloc(llvm_type, "") {
            return allocated_heap_ptr;
        }
    } else if attributes.has_stack_attr() {
        if let Ok(allocated_stack_ptr) = llvm_builder.build_alloca(llvm_type, "") {
            return allocated_stack_ptr;
        }
    } else if kind.is_probably_heap_allocated(llvm_context, target_data) {
        if let Ok(allocated_heap_ptr) = llvm_builder.build_malloc(llvm_type, "") {
            return allocated_heap_ptr;
        }
    } else if let Ok(allocated_stack_ptr) = llvm_builder.build_alloca(llvm_type, "") {
        return allocated_stack_ptr;
    }

    logging::log(
        LoggingType::Bug,
        &format!("Unable to allocate pointer with type: '{}'.", kind),
    );

    unreachable!()
}

pub fn alloc_constant<'ctx>(
    module: &Module<'ctx>,
    name: &str,
    llvm_type: BasicTypeEnum<'ctx>,
    llvm_value: BasicValueEnum<'ctx>,
    attributes: &'ctx ThrushAttributes<'ctx>,
) -> PointerValue<'ctx> {
    let global: GlobalValue = module.add_global(llvm_type, Some(AddressSpace::default()), name);

    if !attributes.has_public_attribute() {
        global.set_linkage(Linkage::LinkerPrivate)
    }

    global.set_initializer(&llvm_value);
    global.set_constant(true);

    global.as_pointer_value()
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

#[derive(Debug, Clone, Copy, Default)]
pub struct CompileChanges {
    load_raw: bool,
}

// IDEA DE RAW GEN, CONTRA PARTE CRUDA DE PUNTEROS.

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx ThrushStatement,
    changes: CompileChanges,
) -> BasicValueEnum<'ctx> {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    /* ######################################################################


        EXPRESSIONS - START


    ########################################################################*/

    if let ThrushStatement::NullPtr { .. } = expr {
        return llvm_context
            .ptr_type(AddressSpace::default())
            .const_null()
            .into();
    }

    if let ThrushStatement::Str { bytes, kind, .. } = expr {
        let ptr: PointerValue = utils::build_str_constant(llvm_module, llvm_context, bytes);

        if changes.load_raw() {
            return ptr.into();
        }

        return memory::load_anon(context, ptr, kind);
    }

    if let ThrushStatement::Float {
        kind,
        value,
        signed,
        ..
    } = expr
    {
        return self::float(llvm_builder, llvm_context, kind, *value, *signed).into();
    }

    if let ThrushStatement::Integer {
        kind,
        value,
        signed,
        ..
    } = expr
    {
        return self::integer(llvm_context, kind, *value, *signed).into();
    }

    if let ThrushStatement::Char { byte, .. } = expr {
        return llvm_context.i8_type().const_int(*byte, false).into();
    }

    if let ThrushStatement::Boolean { value, .. } = expr {
        return llvm_context.bool_type().const_int(*value, false).into();
    }

    if let ThrushStatement::Call {
        name, args, kind, ..
    } = expr
    {
        if *name == "is_signed!" {
            return builtins::build_is_signed(context, (name, kind, args));
        }

        let function: LLVMFunction = context.get_function(name);
        let function_arguments_types: &[ThrushType] = function.1;
        let function_convention: u32 = function.2;

        let llvm_function: FunctionValue = function.0;

        let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());

        args.iter().enumerate().for_each(|arg| {
            let arg_position: usize = arg.0;
            let arg_expr: &ThrushStatement = arg.1;

            let cast: &ThrushType = function_arguments_types
                .get(arg_position)
                .unwrap_or(&ThrushType::Void);

            compiled_args.push(
                self::compile(
                    context,
                    arg_expr,
                    CompileChanges::new(cast.is_mut_type() || cast.is_ptr_type()),
                )
                .into(),
            );
        });

        if let Ok(call) = llvm_builder.build_call(llvm_function, &compiled_args, "") {
            call.set_call_convention(function_convention);

            if !kind.is_void_type() {
                let return_value: BasicValueEnum = call.try_as_basic_value().unwrap_left();

                if changes.load_raw() {
                    return return_value;
                }

                return return_value;
            }

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        logging::log(
            LoggingType::Bug,
            "Unable to create a function call at code generation time.",
        );
    }

    if let ThrushStatement::Group { expression, .. } = expr {
        return self::compile(context, expression, changes);
    }

    if let ThrushStatement::BinaryOp {
        left,
        operator,
        right,
        kind: binaryop_type,
        ..
    } = expr
    {
        if binaryop_type.is_float_type() {
            return binaryop::float::float_binaryop(context, (left, operator, right));
        }

        if binaryop_type.is_integer_type() {
            return binaryop::integer::integer_binaryop(context, (left, operator, right));
        }

        if binaryop_type.is_bool_type() {
            return binaryop::boolean::bool_binaryop(context, (left, operator, right));
        }

        if binaryop_type.is_ptr_type() {
            return binaryop::ptr::ptr_binaryop(context, (left, operator, right));
        }

        logging::log(
            LoggingType::Bug,
            "Could not process a binary operation of invalid type.",
        );
    }

    if let ThrushStatement::UnaryOp {
        operator,
        kind,
        expression,
        ..
    } = expr
    {
        return unaryop::unary_op(context, (operator, kind, expression));
    }

    /* ######################################################################


        EXPRESSIONS - END


    ########################################################################*/

    /* ######################################################################


        BUILTINS - START


    ########################################################################*/

    if let ThrushStatement::SizeOf { sizeof, .. } = expr {
        let sizeof_type: BasicTypeEnum = typegen::generate_type(llvm_context, sizeof);

        return sizeof_type
            .size_of()
            .unwrap_or_else(|| {
                logging::log(
                    LoggingType::Panic,
                    &format!("Unable to get size of the type: '{}'.", sizeof),
                );

                unreachable!()
            })
            .into();
    }

    /* ######################################################################


        BUILTINS - END


    ########################################################################*/

    /* ######################################################################


        LOW LEVEL INSTRUCTIONS - START


    ########################################################################*/

    if let ThrushStatement::Alloc {
        type_to_alloc,
        site_allocation,
        ..
    } = expr
    {
        let site_allocation: LLVMAllocationSite = site_allocation.to_llvm_allocation_site();

        return memory::alloc_anon(
            site_allocation,
            context,
            type_to_alloc,
            type_to_alloc.is_all_ptr(),
        )
        .into();
    }

    if let ThrushStatement::Write {
        write_to,
        write_type,
        write_value,
        ..
    } = expr
    {
        let value: BasicValueEnum = self::compile(context, write_value, CompileChanges::new(false));

        if let Some(reference) = &write_to.0 {
            let reference_name: &str = reference.0;

            let symbol: SymbolAllocated = context.get_allocated_symbol(reference_name);

            symbol.store(context, value);

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        if let Some(expr) = write_to.1.as_ref() {
            let expr: PointerValue =
                self::compile(context, expr, CompileChanges::new(true)).into_pointer_value();

            memory::store_anon(context, expr, write_type, value);

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        logging::log(LoggingType::Bug, "Could not get value of 'write' LLI");
    }

    if let ThrushStatement::Load { value, kind, .. } = expr {
        if let Some(any_reference) = &value.0 {
            let name: &str = any_reference.0;

            let ptr: PointerValue = context
                .get_allocated_symbol(name)
                .load(context)
                .into_pointer_value();

            return memory::load_anon(context, ptr, kind);
        }

        if let Some(expr) = &value.1 {
            let ptr: PointerValue =
                self::compile(context, expr, CompileChanges::default()).into_pointer_value();

            return memory::load_anon(context, ptr, kind);
        }

        logging::log(LoggingType::Bug, "Could not get value of 'load' LLI.");
    }

    if let ThrushStatement::Address {
        address_to,
        indexes,
        ..
    } = expr
    {
        if let Some(any_reference) = &address_to.0 {
            let reference_name: &str = any_reference.0;

            let symbol: SymbolAllocated = context.get_allocated_symbol(reference_name);

            let indexes: Vec<IntValue> = indexes
                .iter()
                .map(|indexe| {
                    self::compile(context, indexe, CompileChanges::new(false)).into_int_value()
                })
                .collect();

            return symbol.gep(llvm_context, llvm_builder, &indexes).into();
        }

        if let Some(expr) = &address_to.1 {
            let kind: &ThrushType = expr.get_type_unwrapped();

            let ptr: PointerValue =
                self::compile(context, expr, CompileChanges::new(true)).into_pointer_value();

            let indexes: Vec<IntValue> = indexes
                .iter()
                .map(|indexe| {
                    self::compile(context, indexe, CompileChanges::new(false)).into_int_value()
                })
                .collect();

            return memory::gep_anon(context, ptr, kind, &indexes).into();
        }

        logging::log(
            LoggingType::Bug,
            "Unable to get pointer element at value generation time.",
        );
    }

    /* ######################################################################


        LOW LEVEL INSTRUCTIONS - END


    ########################################################################*/

    /* ######################################################################


        CASTS - START


    ########################################################################*/

    if let ThrushStatement::As { from, cast, .. } = expr {
        if cast.is_ptr_type() || cast.is_str_type() || cast.is_mut_any_nonumeric_type() {
            let val: BasicValueEnum = self::compile(context, from, CompileChanges::new(true));

            if val.is_pointer_value() {
                let to: PointerType =
                    typegen::generate_type(llvm_context, cast).into_pointer_type();

                if let Ok(casted_ptr) =
                    llvm_builder.build_pointer_cast(val.into_pointer_value(), to, "")
                {
                    return casted_ptr.into();
                }
            }
        } else {
            let val: BasicValueEnum = self::compile(context, from, CompileChanges::default());

            let from_type: &ThrushType = from.get_type_unwrapped();

            let target_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, cast);

            if from_type.is_same_size(context, cast) {
                if let Ok(casted_value) = llvm_builder.build_bit_cast(val, target_type, "") {
                    return casted_value;
                }
            }

            if val.is_int_value() && target_type.is_int_type() {
                if let Ok(casted_value) = llvm_builder.build_int_cast(
                    val.into_int_value(),
                    target_type.into_int_type(),
                    "",
                ) {
                    return casted_value.into();
                }
            }

            if val.is_float_value() && target_type.is_float_type() {
                if let Ok(casted_value) = llvm_builder.build_float_cast(
                    val.into_float_value(),
                    target_type.into_float_type(),
                    "",
                ) {
                    return casted_value.into();
                }
            }
        }

        logging::log(
            LoggingType::Bug,
            &format!(
                "Primitive casting could not be perform at 'cast' from: '{}'.",
                from
            ),
        );
    }

    if let ThrushStatement::CastRaw { from, .. } = expr {
        return self::compile(context, from, CompileChanges::new(true));
    }

    /* ######################################################################


        CASTS - END


    ########################################################################*/

    /* ######################################################################


        DEFERENCE OPERATION - START


    ########################################################################*/

    if let ThrushStatement::Deref { .. } = expr {
        return self::compile_deref(context, expr);
    }

    /* ######################################################################


        DEFERENCE OPERATION - END


    ########################################################################*/

    /* ######################################################################


        REFERENCES OPERATIONS - START


    ########################################################################*/

    if let ThrushStatement::Property {
        name,
        indexes,
        kind,
        ..
    } = expr
    {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        let first_idx: u32 = indexes[0].1;

        if symbol.is_pointer() {
            let mut ptr: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, first_idx);

            indexes.iter().skip(1).for_each(|indexe| {
                let llvm_indexe_type: BasicTypeEnum =
                    typegen::generate_type(llvm_context, &indexe.0);

                let depth: u32 = indexe.1;

                if let Ok(new_ptr) = llvm_builder.build_struct_gep(llvm_indexe_type, ptr, depth, "")
                {
                    ptr = new_ptr;
                }
            });

            if changes.load_raw() {
                return ptr.into();
            }

            return memory::load_anon(context, ptr, kind);
        } else {
            let mut value: BasicValueEnum = symbol.extract_value(llvm_builder, first_idx);

            indexes.iter().skip(1).for_each(|indexe| {
                let depth: u32 = indexe.1;

                if value.is_struct_value() {
                    let value_struct_value: StructValue = value.into_struct_value();

                    if let Ok(new_extracted_value) =
                        llvm_builder.build_extract_value(value_struct_value, depth, "")
                    {
                        value = new_extracted_value;
                    }
                }
            });

            return value;
        }
    }

    if let ThrushStatement::Reference { name, .. } = expr {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);

        if changes.load_raw() {
            return symbol.raw_load().into();
        }

        return symbol.load(context);
    }

    /* ######################################################################


        REFERENCES OPERATIONS - END


    ########################################################################*/

    if let ThrushStatement::AsmValue {
        assembler,
        constraints,
        args,
        kind,
        attributes,
        ..
    } = expr
    {
        let asm_function_type: FunctionType = typegen::function_type(context, kind, args, false);

        let args: Vec<BasicMetadataValueEnum> = args
            .iter()
            .map(|arg| self::compile(context, arg, changes).into())
            .collect();

        let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

        let sideeffects: bool = attributes.has_asmsideffects_attribute();
        let align_stack: bool = attributes.has_asmalignstack_attribute();
        let can_throw: bool = attributes.has_asmthrow_attribute();

        attributes.iter().for_each(|attribute| {
            if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attribute {
                syntax = str::assembler_syntax_attr_to_inline_assembler_dialect(new_syntax);
            }
        });

        let fn_inline_assembler: PointerValue = llvm_context.create_inline_asm(
            asm_function_type,
            assembler.to_string(),
            constraints.to_string(),
            sideeffects,
            align_stack,
            Some(syntax),
            can_throw,
        );

        if let Ok(indirect_call) =
            llvm_builder.build_indirect_call(asm_function_type, fn_inline_assembler, &args, "")
        {
            if !kind.is_void_type() {
                let return_value: BasicValueEnum = indirect_call.try_as_basic_value().unwrap_left();

                if changes.load_raw() {
                    return return_value;
                }

                return return_value;
            }

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        logging::log(LoggingType::Bug, "Unable to build inline assembler value");
        unreachable!()
    }

    if let ThrushStatement::Mut { source, value, .. } = expr {
        let value_type: &ThrushType = value.get_type_unwrapped();

        if let Some(any_reference) = &source.0 {
            let reference_name: &str = any_reference.0;

            let symbol: SymbolAllocated = context.get_allocated_symbol(reference_name);

            let value: BasicValueEnum = self::compile(context, value, CompileChanges::new(false));

            symbol.store(context, value);

            return value;
        }

        if let Some(expr) = &source.1 {
            let source: BasicValueEnum = self::compile(context, expr, CompileChanges::new(true));
            let value: BasicValueEnum = self::compile(context, value, CompileChanges::new(false));

            memory::store_anon(context, source.into_pointer_value(), value_type, value);

            return source;
        }

        logging::log(LoggingType::Bug, "Could not get value of an mutation.");
    }

    if let ThrushStatement::Array { items, kind, .. } = expr {
        if expr.is_constant_array() {
            return self::compile_constant_fixed_array(context, kind, items, changes);
        } else {
            return self::compile_fixed_array(context, kind, items, changes);
        }
    }

    if let ThrushStatement::Index {
        index_to,
        indexes,
        kind,
        ..
    } = expr
    {
        if let Some(any_reference) = &index_to.0 {
            let name: &str = any_reference.0;

            let symbol: SymbolAllocated = context.get_allocated_symbol(name);

            let mut ordered_indexes: Vec<IntValue> = Vec::with_capacity(indexes.len() * 2);

            indexes.iter().for_each(|indexe| {
                if kind.is_mut_fixed_array_type() || kind.is_ptr_fixed_array_type() {
                    let base: IntValue =
                        valuegen::integer(llvm_context, &ThrushType::U32, 0, false);

                    ordered_indexes.push(base);
                }

                let depth: IntValue =
                    self::compile(context, indexe, CompileChanges::default()).into_int_value();

                ordered_indexes.push(depth);
            });

            let ptr: PointerValue = symbol.gep(llvm_context, llvm_builder, &ordered_indexes);

            if changes.load_raw() {
                return ptr.into();
            }

            return memory::load_anon(context, ptr, kind);
        }

        if let Some(expr) = &index_to.1 {
            let expr: PointerValue =
                self::compile(context, expr, CompileChanges::new(true)).into_pointer_value();

            let mut ordered_indexes: Vec<IntValue> = Vec::with_capacity(indexes.len() * 2);

            indexes.iter().for_each(|indexe| {
                if kind.is_mut_fixed_array_type() || kind.is_ptr_fixed_array_type() {
                    let base: IntValue =
                        valuegen::integer(llvm_context, &ThrushType::U32, 0, false);

                    ordered_indexes.push(base);
                }

                let depth: IntValue =
                    self::compile(context, indexe, CompileChanges::default()).into_int_value();

                ordered_indexes.push(depth);
            });

            let ptr: PointerValue = memory::gep_anon(context, expr, kind, &ordered_indexes);

            if changes.load_raw() {
                return ptr.into();
            }

            return memory::load_anon(context, ptr, kind);
        }

        logging::log(
            LoggingType::Bug,
            &format!(
                "A memory address calculation could not be performed with the expression: '{}'.",
                expr
            ),
        );
    }

    logging::log(
        LoggingType::Bug,
        &format!("Unable to compile unknown expression: '{}'.", expr),
    );

    unreachable!()
}

fn compile_deref<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx ThrushStatement,
) -> BasicValueEnum<'ctx> {
    match expr {
        ThrushStatement::Deref { value, kind, .. } => {
            let value: BasicValueEnum = self::compile_deref(context, value);

            if value.is_pointer_value() {
                let ptr: PointerValue = value.into_pointer_value();

                return memory::load_anon(context, ptr, kind);
            }

            value
        }

        expr => self::compile(context, expr, CompileChanges::new(true)),
    }
}

fn compile_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &'ctx ThrushType,
    items: &'ctx [ThrushStatement],
    changes: CompileChanges,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let array_ptr: PointerValue =
        memory::alloc_anon(LLVMAllocationSite::Stack, context, kind, true);

    let array_ptr_type: BasicTypeEnum = typegen::generate_type(llvm_context, kind);

    for (idx, item) in items.iter().enumerate() {
        let llvm_idx: IntValue = llvm_context.i32_type().const_int(idx as u64, false);

        let element_ptr: PointerValue = unsafe {
            llvm_builder.build_gep(
                array_ptr_type,
                array_ptr,
                &[llvm_context.i32_type().const_zero(), llvm_idx],
                "",
            )
        }
        .unwrap_or_else(|_| {
            logging::log(
                LoggingType::Bug,
                "Unable to calculate memory address of an element of a array.",
            );

            unreachable!()
        });

        let compiled_item: BasicValueEnum = self::compile(context, item, changes);

        memory::store_anon(
            context,
            element_ptr,
            item.get_type_unwrapped(),
            compiled_item,
        );
    }

    if changes.load_raw() {
        return array_ptr.into();
    }

    memory::load_anon(context, array_ptr, kind)
}
fn compile_constant_fixed_array<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    kind: &ThrushType,
    items: &'ctx [ThrushStatement],
    changes: CompileChanges,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();

    let array_type: &ThrushType = kind.get_array_type();

    let array_type: BasicTypeEnum = typegen::generate_type(llvm_context, array_type);

    let values: Vec<BasicValueEnum> = items
        .iter()
        .map(|item| self::compile(context, item, changes))
        .collect();

    if array_type.is_int_type() {
        let array_type: IntType = array_type.into_int_type();
        let values: Vec<IntValue> = values.iter().map(|value| value.into_int_value()).collect();

        return array_type.const_array(&values).into();
    }

    if array_type.is_float_type() {
        let array_type: FloatType = array_type.into_float_type();

        let values: Vec<FloatValue> = values
            .iter()
            .map(|value| value.into_float_value())
            .collect();

        return array_type.const_array(&values).into();
    }

    if array_type.is_array_type() {
        let array_type: ArrayType = array_type.into_array_type();

        let values: Vec<ArrayValue> = values
            .iter()
            .map(|value| value.into_array_value())
            .collect();

        return array_type.const_array(&values).into();
    }

    if array_type.is_struct_type() {
        let array_type: StructType = array_type.into_struct_type();
        let values: Vec<StructValue> = values
            .iter()
            .map(|value| value.into_struct_value())
            .collect();

        return array_type.const_array(&values).into();
    }

    if array_type.is_pointer_type() {
        let array_type: PointerType = array_type.into_pointer_type();

        let values: Vec<PointerValue> = values
            .iter()
            .map(|value| value.into_pointer_value())
            .collect();

        return array_type.const_array(&values).into();
    }

    logging::log(
        LoggingType::Bug,
        "An attempt was made to create an LLVM array from an incompatible type.",
    );

    unreachable!()
}

impl CompileChanges {
    pub fn new(load_raw: bool) -> Self {
        Self { load_raw }
    }

    pub fn load_raw(&self) -> bool {
        self.load_raw
    }
}
