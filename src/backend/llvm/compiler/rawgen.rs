use crate::backend::llvm::compiler::attributes::LLVMAttribute;
use crate::backend::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backend::llvm::compiler::{intgen, rawgen, utils, valuegen};
use crate::backend::types::representations::LLVMFunction;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::lexer::traits::{
    ThrushTypeMutableExtensions, ThrushTypePointerExtensions,
};
use crate::frontend::types::parser::stmts::stmt::ThrushStatement;
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;

use crate::backend::types::traits::AssemblerFunctionExtensions;

use super::context::LLVMCodeGenContext;
use super::typegen;

use inkwell::module::Module;

use inkwell::types::{BasicTypeEnum, FunctionType, PointerType};

use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, StructValue,
};

use inkwell::{AddressSpace, InlineAsmDialect};
use inkwell::{builder::Builder, context::Context, values::PointerValue};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx ThrushStatement,
    cast_type: Option<&ThrushType>,
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

    if let ThrushStatement::Str { bytes, .. } = expr {
        return utils::build_str_constant(llvm_module, llvm_context, bytes).into();
    }

    if let ThrushStatement::Call {
        name, args, kind, ..
    } = expr
    {
        let function: LLVMFunction = context.get_function(name);
        let function_arguments_types: &[ThrushType] = function.1;
        let function_convention: u32 = function.2;

        let llvm_function: FunctionValue = function.0;

        let mut compiled_args: Vec<BasicMetadataValueEnum> = Vec::with_capacity(args.len());

        args.iter().enumerate().for_each(|arg| {
            let arg_position: usize = arg.0;
            let expr: &ThrushStatement = arg.1;

            let cast_type: Option<&ThrushType> = function_arguments_types.get(arg_position);

            let compiled_arg: BasicValueEnum = if cast_type
                .is_some_and(|cast_type| cast_type.is_ptr_type() || cast_type.is_mut_type())
            {
                self::compile(context, expr, cast_type)
            } else {
                valuegen::compile(context, expr, cast_type)
            };

            compiled_args.push(compiled_arg.into());
        });

        if let Ok(call) = llvm_builder.build_call(llvm_function, &compiled_args, "") {
            call.set_call_convention(function_convention);

            if !kind.is_void_type() {
                return call.try_as_basic_value().unwrap_left();
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
        return self::compile(context, expression, cast_type);
    }

    /* ######################################################################


        EXPRESSIONS - END


    ########################################################################*/

    /* ######################################################################


        CASTS - START


    ########################################################################*/

    if let ThrushStatement::As { from, cast, .. } = expr {
        let from_type: &ThrushType = from.get_type_unwrapped();

        if from_type.is_str_type() && cast.is_ptr_type() {
            let val: BasicValueEnum = rawgen::compile(context, from, None);

            if val.is_pointer_value() {
                let raw_str_ptr: PointerValue = val.into_pointer_value();

                let str_loaded: BasicValueEnum = memory::load_anon(context, raw_str_ptr, from_type);
                let str_structure: StructValue = str_loaded.into_struct_value();

                if let Ok(cstr) = llvm_builder.build_extract_value(str_structure, 0, "") {
                    let to: PointerType =
                        typegen::generate_type(llvm_context, cast).into_pointer_type();

                    if let Ok(casted_ptr) =
                        llvm_builder.build_pointer_cast(cstr.into_pointer_value(), to, "")
                    {
                        return casted_ptr.into();
                    }
                }
            }
        } else if cast.is_ptr_type() || cast.is_mut_type() {
            let val: BasicValueEnum = rawgen::compile(context, from, None);

            if val.is_pointer_value() {
                let to: PointerType =
                    typegen::generate_type(llvm_context, cast).into_pointer_type();

                if let Ok(casted_ptr) =
                    llvm_builder.build_pointer_cast(val.into_pointer_value(), to, "")
                {
                    return casted_ptr.into();
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

    /* ######################################################################


        CASTS - END


    ########################################################################*/

    /* ######################################################################


        DEFERENCE OPERATION - START


    ########################################################################*/

    if let ThrushStatement::Deref { .. } = expr {
        return self::deref(context, expr, cast_type);
    }

    /* ######################################################################


        DEFERENCE OPERATION - END


    ########################################################################*/

    /* ######################################################################


        REFERENCES OPERATIONS - START


    ########################################################################*/

    if let ThrushStatement::Property { name, indexes, .. } = expr {
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

            return ptr.into();
        }
    }

    if let ThrushStatement::Reference { name, .. } = expr {
        let symbol: SymbolAllocated = context.get_allocated_symbol(name);
        return symbol.raw_load().into();
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
            .map(|arg| valuegen::compile(context, arg, None).into())
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

                return return_value;
            }

            return llvm_context
                .ptr_type(AddressSpace::default())
                .const_null()
                .into();
        }

        logging::log(LoggingType::Bug, "Unable to build inline assembler value.");

        unreachable!()
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
                    let base: IntValue = intgen::integer(llvm_context, &ThrushType::U32, 0, false);

                    let depth: IntValue =
                        valuegen::compile(context, indexe, Some(&ThrushType::U32)).into_int_value();

                    ordered_indexes.push(base);
                    ordered_indexes.push(depth);
                } else {
                    let depth: IntValue =
                        valuegen::compile(context, indexe, Some(&ThrushType::U64)).into_int_value();

                    ordered_indexes.push(depth);
                }
            });

            let ptr: PointerValue = symbol.gep(llvm_context, llvm_builder, &ordered_indexes);

            return ptr.into();
        }

        if let Some(expr) = &index_to.1 {
            let expr: PointerValue = self::compile(context, expr, None).into_pointer_value();

            let mut ordered_indexes: Vec<IntValue> = Vec::with_capacity(indexes.len() * 2);

            indexes.iter().for_each(|indexe| {
                if kind.is_mut_fixed_array_type() || kind.is_ptr_fixed_array_type() {
                    let base: IntValue = intgen::integer(llvm_context, &ThrushType::U32, 0, false);

                    let depth: IntValue =
                        valuegen::compile(context, indexe, Some(&ThrushType::U32)).into_int_value();

                    ordered_indexes.push(base);
                    ordered_indexes.push(depth);
                } else {
                    let depth: IntValue =
                        valuegen::compile(context, indexe, Some(&ThrushType::U64)).into_int_value();

                    ordered_indexes.push(depth);
                }
            });

            let ptr: PointerValue = memory::gep_anon(context, expr, kind, &ordered_indexes);

            return ptr.into();
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

fn deref<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx ThrushStatement,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    match expr {
        ThrushStatement::Deref { value, kind, .. } => {
            let value: BasicValueEnum = self::deref(context, value, Some(kind));

            if value.is_pointer_value() {
                let ptr: PointerValue = value.into_pointer_value();

                return memory::load_anon(context, ptr, kind);
            }

            value
        }

        expr => self::compile(context, expr, cast_type),
    }
}
