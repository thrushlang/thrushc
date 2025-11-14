use crate::back_end::llvm::compiler::attrbuilder::{AttributeBuilder, LLVMAttributeApplicant};
use crate::back_end::llvm::compiler::attributes::LLVMAttribute;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::conventions::CallConvention;
use crate::back_end::llvm::compiler::typegen;

use crate::front_end::lexer::span::Span;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::repr::Intrinsic;
use crate::front_end::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;
use crate::front_end::types::semantic::linter::types::LLVMAttributeComparator;
use crate::front_end::typesystem::types::Type;

use inkwell::{context::Context, module::Module, types::FunctionType, values::FunctionValue};

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, intrinsic: Intrinsic<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let name: &str = intrinsic.0;
    let external_name: &str = intrinsic.1;
    let return_type: &Type = intrinsic.2;
    let parameters: &[Ast<'ctx>] = intrinsic.3;
    let parameters_types: &[Type] = intrinsic.4;
    let attributes: &ThrushAttributes = intrinsic.5;

    let span: Span = intrinsic.6;

    let ignore_args: bool = attributes.has_ignore_attribute();

    let mut convention: u32 = if let Some(LLVMAttribute::Convention(conv, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Convention)
    {
        conv as u32
    } else {
        CallConvention::Standard as u32
    };

    let function_type: FunctionType =
        typegen::generate_fn_type(context, return_type, parameters, ignore_args);

    let llvm_function: FunctionValue = llvm_module.add_function(external_name, function_type, None);

    let mut attribute_builder: AttributeBuilder = AttributeBuilder::new(
        llvm_context,
        attributes,
        LLVMAttributeApplicant::Function(llvm_function),
    );

    attribute_builder.add_function_attributes(&mut convention);

    context.new_function(name, (llvm_function, parameters_types, convention, span));
}
