use crate::back_end::llvm::compiler::abort;
use crate::back_end::llvm::compiler::attrbuilder::AttributeBuilder;
use crate::back_end::llvm::compiler::attrbuilder::LLVMAttributeApplicant;
use crate::back_end::llvm::compiler::attributes::LLVMAttribute;
use crate::back_end::llvm::compiler::attributes::LLVMAttributeComparator;
use crate::back_end::llvm::compiler::block;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::conventions::CallConvention;
use crate::back_end::llvm::compiler::obfuscation;
use crate::back_end::llvm::compiler::typegen;
use crate::back_end::llvm::types::repr::LLVMAttributes;
use crate::back_end::llvm::types::traits::AssemblerFunctionExtensions;
use crate::back_end::llvm::types::traits::LLVMAttributesExtensions;

use crate::core::diagnostic::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::repr::AssemblerFunction;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use std::path::PathBuf;

use inkwell::InlineAsmDialect;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::FunctionType;
use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::PointerValue;

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, asm_fn: AssemblerFunction<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();
    let llvm_builder: &Builder = context.get_llvm_builder();

    let name: &str = asm_fn.0;
    let ascii_name: &str = asm_fn.1;

    let assembler: String = asm_fn.2.to_string();
    let constraints: String = asm_fn.3.to_string();
    let return_type: &Type = asm_fn.4;
    let parameters: &[Ast] = asm_fn.5;
    let parameters_types: &[Type] = asm_fn.6;
    let attributes: LLVMAttributes = asm_fn.7.as_llvm_attributes();

    let span: Span = asm_fn.8;

    let sideeffects: bool = attributes.has_asmsideffects_attribute();
    let align_stack: bool = attributes.has_asmalignstack_attribute();
    let can_throw: bool = attributes.has_asmthrow_attribute();
    let is_public: bool = attributes.has_public_attribute();

    let call_convention: u32 = if let Some(LLVMAttribute::Convention(conv, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Convention)
    {
        conv as u32
    } else {
        CallConvention::Standard as u32
    };

    let syntax: InlineAsmDialect = if let Some(LLVMAttribute::AsmSyntax(new_syntax, ..)) =
        attributes.get_attr(LLVMAttributeComparator::AsmSyntax)
    {
        str::as_inline_assembler_dialect(new_syntax)
    } else {
        InlineAsmDialect::Intel
    };

    let llvm_function_name: String = if is_public {
        format!("__asm_fn_{}", ascii_name)
    } else {
        format!(
            "__asm_fn_{}_{}",
            obfuscation::generate_obfuscation_name(context, obfuscation::LONG_RANGE_OBFUSCATION),
            ascii_name
        )
    };

    let asm_function_type: FunctionType =
        typegen::generate_fn_type(context, return_type, parameters, false);

    let asm_function_ptr: PointerValue = llvm_context.create_inline_asm(
        asm_function_type,
        assembler,
        constraints,
        sideeffects,
        align_stack,
        Some(syntax),
        can_throw,
    );

    let asm_function: FunctionValue =
        llvm_module.add_function(&llvm_function_name, asm_function_type, None);

    if !is_public {
        asm_function.set_linkage(Linkage::LinkerPrivate);
    }

    AttributeBuilder::new(
        llvm_context,
        &attributes,
        LLVMAttributeApplicant::Function(asm_function),
    )
    .add_function_attributes();

    let last_block: BasicBlock = context.get_last_builder_block();

    let asm_function_block: BasicBlock = block::append_block(context, asm_function);

    llvm_builder.position_at_end(asm_function_block);

    let args: Vec<BasicMetadataValueEnum> = asm_function
        .get_param_iter()
        .map(|param| param.into())
        .collect();

    if let Ok(asm_fn_call) =
        llvm_builder.build_indirect_call(asm_function_type, asm_function_ptr, &args, "")
    {
        match (
            return_type.is_void_type(),
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
        name,
        (asm_function, parameters_types, call_convention, span),
    );
}
