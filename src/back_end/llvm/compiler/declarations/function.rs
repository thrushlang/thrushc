use crate::back_end::llvm::compiler::attrbuilder::AttributeBuilder;
use crate::back_end::llvm::compiler::attrbuilder::LLVMAttributeApplicant;
use crate::back_end::llvm::compiler::attributes::LLVMAttribute;
use crate::back_end::llvm::compiler::attributes::LLVMAttributeComparator;
use crate::back_end::llvm::compiler::block;
use crate::back_end::llvm::compiler::codegen::LLVMCodegen;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::conventions::CallConvention;
use crate::back_end::llvm::compiler::obfuscation;
use crate::back_end::llvm::compiler::typegen;
use crate::back_end::llvm::types::repr::LLVMAttributes;
use crate::back_end::llvm::types::repr::LLVMFunction;
use crate::back_end::llvm::types::traits::LLVMAttributesExtensions;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstCodeBlockEntensions;
use crate::front_end::types::attributes::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::repr::Function;
use crate::front_end::types::parser::repr::FunctionParameter;
use crate::front_end::typesystem::types::Type;

use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

pub fn compile_decl<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, function: Function<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let name: &str = function.0;
    let ascii_name: &str = function.1;

    let function_type: &Type = function.2;

    let parameters: &[Ast<'ctx>] = function.3;
    let parameters_types: &[Type] = function.4;
    let attributes: LLVMAttributes = function.6.as_llvm_attributes();
    let span: Span = function.7;

    let ignore_args: bool = attributes.has_ignore_attribute();
    let is_public: bool = attributes.has_public_attribute();

    let call_convention: u32 = if let Some(LLVMAttribute::Convention(conv, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Convention)
    {
        conv as u32
    } else {
        CallConvention::Standard as u32
    };

    let canonical_name: &str = if let Some(LLVMAttribute::Extern(extern_name, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Extern)
    {
        extern_name
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

    if !is_public {
        llvm_function.set_linkage(Linkage::LinkerPrivate);
    }

    AttributeBuilder::new(
        llvm_context,
        &attributes,
        LLVMAttributeApplicant::Function(llvm_function),
    )
    .add_function_attributes();

    context.set_current_fn(llvm_function);

    context.new_function(
        name,
        (llvm_function, parameters_types, call_convention, span),
    );
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

        if function_type.is_void_type() && !function_body.has_terminator() {
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
