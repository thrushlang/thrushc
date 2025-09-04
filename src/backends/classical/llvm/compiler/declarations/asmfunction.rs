use crate::backends::classical::llvm::compiler::attributes::LLVMAttribute;
use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::conventions::CallConvention;
use crate::backends::classical::llvm::compiler::utils;
use crate::backends::classical::llvm::compiler::utils::LONG_RANGE_OBFUSCATION;

use crate::backends::classical::llvm::compiler::typegen;
use crate::backends::classical::types::traits::AssemblerFunctionExtensions;

use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;
use crate::frontends::classical::typesystem::types::Type;

use crate::frontends::classical::types::parser::repr::GlobalAssemblerFunction;

use std::fmt::Display;

use inkwell::{
    InlineAsmDialect,
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::FunctionType,
    values::{BasicMetadataValueEnum, FunctionValue, PointerValue},
};

pub fn compile<'ctx>(
    context: &mut LLVMCodeGenContext<'_, 'ctx>,
    asm_fn: GlobalAssemblerFunction<'ctx>,
) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let asm_function_name: &str = asm_fn.0;
    let asm_function_ascii_name: &str = asm_fn.1;
    let asm_function_assembler: String = asm_fn.2.to_string();
    let asm_function_constraints: String = asm_fn.3.to_string();
    let asm_function_return_type: &Type = asm_fn.4;
    let asm_function_parameters: &[Ast] = asm_fn.5;
    let asm_function_parameters_types: &[Type] = asm_fn.6;
    let asm_function_attributes: &ThrushAttributes = asm_fn.7;

    let sideeffects: bool = asm_function_attributes.has_asmsideffects_attribute();
    let align_stack: bool = asm_function_attributes.has_asmalignstack_attribute();
    let can_throw: bool = asm_function_attributes.has_asmthrow_attribute();
    let is_public: bool = asm_function_attributes.has_public_attribute();

    let mut call_convention: u32 = CallConvention::Standard as u32;
    let mut syntax: InlineAsmDialect = InlineAsmDialect::Intel;

    asm_function_attributes.iter().for_each(|attribute| {
        if let LLVMAttribute::Convention(call_conv, _) = attribute {
            call_convention = (*call_conv) as u32;
        }

        if let LLVMAttribute::AsmSyntax(new_syntax, ..) = *attribute {
            syntax = str::to_inline_assembler_dialect(new_syntax);
        }
    });

    let llvm_function_name: String = if is_public {
        format!("__asm_fn_{}", asm_function_ascii_name)
    } else {
        format!(
            "__asm_fn_{}_{}",
            utils::generate_random_string(LONG_RANGE_OBFUSCATION),
            asm_function_ascii_name
        )
    };

    let asm_function_type: FunctionType = typegen::function_type(
        context,
        asm_function_return_type,
        asm_function_parameters,
        false,
    );

    let asm_function_ptr: PointerValue = llvm_context.create_inline_asm(
        asm_function_type,
        asm_function_assembler,
        asm_function_constraints,
        sideeffects,
        align_stack,
        Some(syntax),
        can_throw,
    );

    let llvm_asm_function: FunctionValue =
        llvm_module.add_function(&llvm_function_name, asm_function_type, None);

    if !is_public {
        llvm_asm_function.set_linkage(Linkage::LinkerPrivate);
    }

    let original_block: BasicBlock = context.get_last_builder_block();

    let entry: BasicBlock = llvm_context.append_basic_block(llvm_asm_function, "");

    llvm_builder.position_at_end(entry);

    let args: Vec<BasicMetadataValueEnum> = llvm_asm_function
        .get_param_iter()
        .map(|param| param.into())
        .collect();

    if let Ok(asm_fn_call) =
        llvm_builder.build_indirect_call(asm_function_type, asm_function_ptr, &args, "")
    {
        match (
            asm_function_return_type.is_void_type(),
            asm_fn_call.try_as_basic_value().left(),
        ) {
            (false, Some(return_value)) => {
                llvm_builder.build_return(Some(&return_value))
            .map_err(|_| {
                self::codegen_abort(
                    "Failed to create return terminator with value in assembly function generation.");
            })
            .ok();
            }
            _ => {
                llvm_builder.build_return(None)
            .map_err(|_| {
                self::codegen_abort("Failed to create void return terminator in assembly function generation.",);
            })
            .ok();
            }
        }
    } else {
        self::codegen_abort("Unable to create indirect call for call assembly function.");
    }

    llvm_builder.position_at_end(original_block);

    context.new_function(
        asm_function_name,
        (
            llvm_asm_function,
            asm_function_parameters_types,
            call_convention,
        ),
    );
}

#[inline]
fn codegen_abort<T: Display>(message: T) -> ! {
    logging::print_backend_bug(LoggingType::BackendBug, &format!("{}", message));
}
