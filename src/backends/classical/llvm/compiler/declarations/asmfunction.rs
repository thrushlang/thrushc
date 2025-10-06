use crate::backends::classical::llvm::compiler::context::LLVMCodeGenContext;
use crate::backends::classical::llvm::compiler::conventions::CallConvention;
use crate::backends::classical::llvm::compiler::{attributes::LLVMAttribute, obfuscation};

use crate::backends::classical::llvm::compiler::{abort, block, typegen};
use crate::backends::classical::types::traits::AssemblerFunctionExtensions;

use crate::frontends::classical::lexer::span::Span;
use crate::frontends::classical::types::ast::Ast;
use crate::frontends::classical::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::frontends::classical::types::parser::stmts::types::ThrushAttributes;
use crate::frontends::classical::typesystem::types::Type;

use crate::frontends::classical::types::parser::repr::GlobalAssemblerFunction;

use std::path::PathBuf;

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

    let span: Span = asm_fn.8;

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
            obfuscation::generate_obfuscation_name(context, obfuscation::LONG_RANGE_OBFUSCATION),
            asm_function_ascii_name
        )
    };

    let asm_function_type: FunctionType = typegen::generate_fn_type(
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

    let last_block: BasicBlock = context.get_last_builder_block();

    let asm_function_block: BasicBlock = block::append_block(context, llvm_asm_function);

    llvm_builder.position_at_end(asm_function_block);

    if !is_public {
        llvm_asm_function.set_linkage(Linkage::LinkerPrivate);
    }

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
                llvm_builder
                    .build_return(Some(&return_value))
                    .map_err(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile assembly function!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .ok();
            }
            _ => {
                llvm_builder
                    .build_return(None)
                    .map_err(|_| {
                        abort::abort_codegen(
                            context,
                            "Failed to compile assembly function!",
                            span,
                            PathBuf::from(file!()),
                            line!(),
                        );
                    })
                    .ok();
            }
        }
    } else {
        abort::abort_codegen(
            context,
            "Failed to compile indirect call for assembly function!",
            span,
            PathBuf::from(file!()),
            line!(),
        );
    }

    llvm_builder.position_at_end(last_block);

    context.new_function(
        asm_function_name,
        (
            llvm_asm_function,
            asm_function_parameters_types,
            call_convention,
            span,
        ),
    );
}
