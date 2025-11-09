use crate::back_end::llvm::compiler::attrbuilder::{AttributeBuilder, LLVMAttributeApplicant};
use crate::back_end::llvm::compiler::attributes::LLVMAttribute;
use crate::back_end::llvm::compiler::codegen::LLVMCodegen;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::conventions::CallConvention;
use crate::back_end::llvm::compiler::{block, obfuscation, typegen};
use crate::back_end::llvm::types::repr::LLVMFunction;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::repr::Function;
use crate::front_end::types::parser::repr::FunctionParameter;
use crate::front_end::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;
use crate::front_end::typesystem::types::Type;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;

use inkwell::{
    context::Context,
    module::{Linkage, Module},
    types::FunctionType,
    values::FunctionValue,
};

pub fn compile_decl<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, function: Function<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let name: &str = function.0;
    let ascii_name: &str = function.1;

    let function_type: &Type = function.2;

    let parameters: &[Ast<'ctx>] = function.3;
    let parameters_types: &[Type] = function.4;
    let attributes: &ThrushAttributes = function.6;
    let span: Span = function.7;

    let ignore_args: bool = attributes.has_ignore_attribute();
    let is_public: bool = attributes.has_public_attribute();

    let mut extern_name: Option<&str> = None;
    let mut convention: u32 = CallConvention::Standard as u32;

    attributes.iter().for_each(|attribute| match attribute {
        LLVMAttribute::Extern(name, ..) => {
            extern_name = Some(name);
        }

        LLVMAttribute::Convention(conv, _) => {
            convention = (*conv) as u32;
        }
        _ => (),
    });

    let canonical_name: &str = if let Some(ffi_name) = extern_name {
        ffi_name
    } else if is_public {
        ascii_name
    } else {
        &format!(
            "__fn_{}_{}",
            obfuscation::generate_obfuscation_name(context, obfuscation::LONG_RANGE_OBFUSCATION),
            ascii_name
        )
    };

    let function_type: FunctionType =
        typegen::generate_fn_type(context, function_type, parameters, ignore_args);

    let llvm_function: FunctionValue =
        llvm_module.add_function(canonical_name, function_type, None);

    if !is_public && extern_name.is_none() {
        llvm_function.set_linkage(Linkage::LinkerPrivate);
    }

    let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
        llvm_context,
        attributes,
        LLVMAttributeApplicant::Function(llvm_function),
    );

    attribute_builder.add_function_attributes(&mut convention);

    context.set_current_fn(llvm_function);

    context.new_function(name, (llvm_function, parameters_types, convention, span));
}

pub fn compile_body<'ctx>(codegen: &mut LLVMCodegen<'_, 'ctx>, function: Function<'ctx>) {
    let llvm_builder: &Builder = codegen.get_context().get_llvm_builder();

    let function_name: &str = function.0;
    let function_type: &Type = function.2;
    let function_parameters: &[Ast<'ctx>] = function.3;
    let function_body: Option<&Ast> = function.5;

    let represented_llvm_function: LLVMFunction = codegen
        .get_context()
        .get_table()
        .get_function(function_name);

    let llvm_function: FunctionValue = represented_llvm_function.0;
    let llvm_function_block: BasicBlock = block::append_block(codegen.get_context(), llvm_function);

    llvm_builder.position_at_end(llvm_function_block);

    codegen.get_mut_context().set_current_fn(llvm_function);

    function_parameters.iter().for_each(|parameter| {
        self::compile_parameter(codegen, llvm_function, parameter.as_function_parameter());
    });

    if let Some(function_body) = function_body {
        codegen.codegen_block(function_body);

        if function_type.is_void_type() && !function_body.has_return_for_function() {
            let _ = llvm_builder.build_return(None).is_err();
        }
    }

    codegen.get_mut_context().unset_current_function();
}

pub fn compile_parameter<'ctx>(
    codegen: &mut LLVMCodegen<'_, 'ctx>,
    llvm_fn: FunctionValue<'ctx>,
    parameter: FunctionParameter<'ctx>,
) {
    let name: &str = parameter.0;
    let ascii_name: &str = parameter.1;

    let kind: &Type = parameter.2;
    let position: u32 = parameter.3;

    let span: Span = parameter.4;

    if let Some(value) = llvm_fn.get_nth_param(position) {
        codegen
            .get_mut_context()
            .new_parameter(name, ascii_name, kind, value, span);
    }
}
