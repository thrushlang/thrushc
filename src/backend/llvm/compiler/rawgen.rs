#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::type_complexity)]

use super::context::LLVMCodeGenContext;
use super::typegen;
use crate::backend::llvm::compiler::attributes::LLVMAttribute;
use crate::backend::llvm::compiler::memory::{self, SymbolAllocated};
use crate::backend::llvm::compiler::{builtins, cast, intgen, rawgen, utils, valuegen};
use crate::backend::types::representations::LLVMFunction;
use crate::backend::types::traits::AssemblerFunctionExtensions;
use crate::core::console::logging::{self, LoggingType};
use crate::frontend::types::lexer::ThrushType;
use crate::frontend::types::lexer::traits::{
    ThrushTypeMutableExtensions, ThrushTypePointerExtensions,
};

use crate::frontend::types::parser::stmts::stmt::ThrushStatement;
use crate::frontend::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontend::types::parser::stmts::types::ThrushAttributes;
use inkwell::types::{BasicTypeEnum, FunctionType, PointerType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, IntValue, PointerValue, StructValue,
};
use inkwell::{AddressSpace, InlineAsmDialect};
use inkwell::{builder::Builder, context::Context};
use std::fmt::Display;
use std::rc::Rc;

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    expr: &'ctx ThrushStatement,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    match expr {
        ThrushStatement::NullPtr { .. } => self::compile_null_ptr(context),

        ThrushStatement::Str { bytes, .. } => self::compile_string(context, bytes),

        ThrushStatement::Call {
            name, args, kind, ..
        } => compile_function_call(context, name, args, kind, cast_type),

        ThrushStatement::Group { expression, .. } => self::compile(context, expression, cast_type),

        ThrushStatement::As { from, cast, .. } => self::compile_cast(context, from, cast),

        ThrushStatement::Deref { value, kind, .. } => {
            self::compile_deref(context, value, kind, cast_type)
        }

        ThrushStatement::Property { name, indexes, .. } => {
            self::compile_property(context, name, indexes)
        }

        ThrushStatement::MemCpy {
            source,
            destination,
            size,
            ..
        } => builtins::memcpy::compile(context, source, destination, size),

        ThrushStatement::MemMove {
            source,
            destination,
            size,
            ..
        } => builtins::memmove::compile(context, source, destination, size),

        ThrushStatement::MemSet {
            destination,
            new_size,
            size,
            ..
        } => builtins::memset::compile(context, destination, new_size, size),

        ThrushStatement::Reference { name, .. } => self::compile_reference(context, name),

        ThrushStatement::AsmValue {
            assembler,
            constraints,
            args,
            kind,
            attributes,
            ..
        } => self::compile_inline_asm(context, assembler, constraints, args, kind, attributes),

        ThrushStatement::Index {
            index_to, indexes, ..
        } => self::compile_index(context, index_to, indexes),

        _ => self::handle_unknown_expression(context, expr),
    }
}

fn compile_function_call<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    args: &'ctx [ThrushStatement],
    kind: &'ctx ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let function: LLVMFunction = context.get_function(name);

    let (llvm_function, function_arg_types, function_convention) =
        (function.0, function.1, function.2);

    let llvm_builder: &Builder = context.get_llvm_builder();

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let arg_cast_type: Option<&ThrushType> = function_arg_types.get(i);

            let compiled_arg: BasicValueEnum =
                if arg_cast_type.is_some_and(|t| t.is_ptr_type() || t.is_mut_type()) {
                    self::compile(context, arg, arg_cast_type)
                } else {
                    valuegen::compile(context, arg, arg_cast_type)
                };

            compiled_arg.into()
        })
        .collect();

    let fn_value: BasicValueEnum = match llvm_builder.build_call(llvm_function, &compiled_args, "")
    {
        Ok(call) => {
            call.set_call_convention(function_convention);
            if !kind.is_void_type() {
                call.try_as_basic_value().left().unwrap_or_else(|| {
                    self::codegen_abort(format!("Function call '{}' returned no value", name));
                    self::compile_null_ptr(context)
                })
            } else {
                self::compile_null_ptr(context)
            }
        }
        Err(_) => {
            self::codegen_abort(format!("Failed to generate call to function '{}'", name));
            self::compile_null_ptr(context)
        }
    };

    if let Some(cast) = cast_type {
        cast::try_cast(context, cast, kind, fn_value).unwrap_or(fn_value)
    } else {
        fn_value
    }
}

fn compile_cast<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    from: &'ctx ThrushStatement,
    cast: &ThrushType,
) -> BasicValueEnum<'ctx> {
    let from_type: &ThrushType = from.get_type_unwrapped();

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let val: BasicValueEnum = rawgen::compile(context, from, None);

    if !val.is_pointer_value() {
        self::codegen_abort(format!(
            "Cannot cast non-pointer value in expression '{}'.",
            from
        ));

        return self::compile_null_ptr(context);
    }

    let raw_ptr: PointerValue = val.into_pointer_value();
    let to_type: PointerType = typegen::generate_type(llvm_context, cast).into_pointer_type();

    match (
        from_type.is_str_type(),
        cast.is_ptr_type() || cast.is_mut_type(),
    ) {
        (true, true) => {
            let str_loaded: BasicValueEnum = memory::load_anon(context, raw_ptr, from_type);
            let str_structure: StructValue = str_loaded.into_struct_value();

            match llvm_builder.build_extract_value(str_structure, 0, "") {
                Ok(cstr) => {
                    match llvm_builder.build_pointer_cast(cstr.into_pointer_value(), to_type, "") {
                        Ok(casted_ptr) => casted_ptr.into(),
                        Err(_) => {
                            self::codegen_abort(format!(
                                "Failed to cast string pointer in '{}'",
                                from
                            ));

                            self::compile_null_ptr(context)
                        }
                    }
                }
                Err(_) => {
                    self::codegen_abort(format!("Failed to extract string value in '{}'", from));
                    self::compile_null_ptr(context)
                }
            }
        }
        (false, true) => match llvm_builder.build_pointer_cast(raw_ptr, to_type, "") {
            Ok(casted_ptr) => casted_ptr.into(),
            Err(_) => {
                self::codegen_abort(format!("Failed to cast pointer in '{}'", from));
                self::compile_null_ptr(context)
            }
        },
        _ => {
            self::codegen_abort(format!(
                "Unsupported cast from '{}' to '{}'",
                from_type, cast
            ));

            self::compile_null_ptr(context)
        }
    }
}

fn compile_deref<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    value: &'ctx ThrushStatement,
    kind: &ThrushType,
    cast_type: Option<&ThrushType>,
) -> BasicValueEnum<'ctx> {
    let val: BasicValueEnum = compile(context, value, Some(kind));

    let deref_value: BasicValueEnum = if val.is_pointer_value() {
        memory::load_anon(context, val.into_pointer_value(), kind)
    } else {
        self::codegen_abort(format!(
            "Cannot dereference non-pointer value in '{}'",
            value
        ));

        val
    };

    if let Some(cast) = cast_type {
        cast::try_cast(context, cast, kind, deref_value).unwrap_or(deref_value)
    } else {
        deref_value
    }
}

fn compile_property<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
    indexes: &[(ThrushType, u32)],
) -> BasicValueEnum<'ctx> {
    let symbol: SymbolAllocated = context.get_allocated_symbol(name);

    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    if !symbol.is_pointer() {
        self::codegen_abort(format!(
            "Symbol '{}' is not a pointer for property access",
            name
        ));

        return self::compile_null_ptr(context);
    }

    let mut ptr: PointerValue = symbol.gep_struct(llvm_context, llvm_builder, indexes[0].1);

    for index in indexes.iter().skip(1) {
        let index_type: BasicTypeEnum = typegen::generate_type(llvm_context, &index.0);

        match llvm_builder.build_struct_gep(index_type, ptr, index.1, "") {
            Ok(new_ptr) => ptr = new_ptr,
            Err(_) => {
                self::codegen_abort(format!(
                    "Failed to access property at index {} for '{}'",
                    index.1, name
                ));

                return self::compile_null_ptr(context);
            }
        }
    }

    ptr.into()
}

fn compile_reference<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    name: &str,
) -> BasicValueEnum<'ctx> {
    let symbol = context.get_allocated_symbol(name);
    symbol.raw_load().into()
}

fn compile_inline_asm<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    assembler: &str,
    constraints: &str,
    args: &'ctx [ThrushStatement],
    kind: &ThrushType,
    attributes: &ThrushAttributes,
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let asm_function_type: FunctionType = typegen::function_type(context, kind, args, false);

    let compiled_args: Vec<BasicMetadataValueEnum> = args
        .iter()
        .map(|arg| valuegen::compile(context, arg, None).into()) // Recursive
        .collect();

    let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();

    for attr in attributes {
        if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attr {
            syntax = str::assembler_syntax_attr_to_inline_assembler_dialect(new_syntax);
        }
    }

    let fn_inline_assembler: PointerValue = llvm_context.create_inline_asm(
        asm_function_type,
        assembler.to_string(),
        constraints.to_string(),
        sideeffects,
        align_stack,
        Some(syntax),
        can_throw,
    );

    match llvm_builder.build_indirect_call(
        asm_function_type,
        fn_inline_assembler,
        &compiled_args,
        "",
    ) {
        Ok(call) if !kind.is_void_type() => call.try_as_basic_value().left().unwrap_or_else(|| {
            self::codegen_abort("Inline assembler returned no value");
            self::compile_null_ptr(context)
        }),

        Ok(_) => compile_null_ptr(context),

        Err(_) => {
            self::codegen_abort("Failed to build inline assembler");
            self::compile_null_ptr(context)
        }
    }
}

fn compile_index<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    index_to: &'ctx (
        Option<(&'ctx str, Rc<ThrushStatement<'ctx>>)>,
        Option<Rc<ThrushStatement<'ctx>>>,
    ),
    indexes: &'ctx [ThrushStatement],
) -> BasicValueEnum<'ctx> {
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    match index_to {
        (Some((name, _)), _) => {
            let symbol: SymbolAllocated = context.get_allocated_symbol(name);
            let symbol_type: &ThrushType = symbol.get_type();

            let ordered_indexes: Vec<IntValue> =
                self::compute_indexes(context, indexes, symbol_type);

            symbol
                .gep(llvm_context, llvm_builder, &ordered_indexes)
                .into()
        }
        (_, Some(expr)) => {
            let expr_ptr: PointerValue = rawgen::compile(context, expr, None).into_pointer_value();
            let expr_type: &ThrushType = expr.get_type_unwrapped();

            let ordered_indexes: Vec<IntValue> = self::compute_indexes(context, indexes, expr_type);

            memory::gep_anon(context, expr_ptr, expr_type, &ordered_indexes).into()
        }
        _ => {
            self::codegen_abort("Invalid index target in expression");
            self::compile_null_ptr(context)
        }
    }
}

fn compute_indexes<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    indexes: &'ctx [ThrushStatement],
    kind: &'ctx ThrushType,
) -> Vec<IntValue<'ctx>> {
    let llvm_context: &Context = context.get_llvm_context();

    indexes
        .iter()
        .flat_map(|index| {
            if kind.is_mut_fixed_array_type() || kind.is_ptr_fixed_array_type() {
                let base = intgen::integer(llvm_context, &ThrushType::U32, 0, false);
                let depth =
                    valuegen::compile(context, index, Some(&ThrushType::U32)).into_int_value();
                vec![base, depth]
            } else {
                let depth =
                    valuegen::compile(context, index, Some(&ThrushType::U64)).into_int_value();
                vec![depth]
            }
        })
        .collect()
}

fn compile_null_ptr<'ctx>(context: &LLVMCodeGenContext<'_, 'ctx>) -> BasicValueEnum<'ctx> {
    context
        .get_llvm_context()
        .ptr_type(AddressSpace::default())
        .const_null()
        .into()
}

fn compile_string<'ctx>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    bytes: &'ctx [u8],
) -> BasicValueEnum<'ctx> {
    utils::build_str_constant(context.get_llvm_module(), context.get_llvm_context(), bytes).into()
}

fn handle_unknown_expression<'ctx, T: Display>(
    context: &LLVMCodeGenContext<'_, 'ctx>,
    expr: T,
) -> BasicValueEnum<'ctx> {
    self::codegen_abort(format!("Unsupported expression: '{}'", expr));

    compile_null_ptr(context)
}

fn codegen_abort<T: Display>(message: T) {
    logging::log(
        LoggingType::Bug,
        &format!("CODE GENERATION: '{}'.", message),
    );
}
