use crate::back_end::llvm_codegen::attrbuilder::AttributeBuilder;
use crate::back_end::llvm_codegen::attrbuilder::LLVMAttributeApplicant;
use crate::back_end::llvm_codegen::attributes::LLVMAttribute;
use crate::back_end::llvm_codegen::attributes::LLVMAttributeComparator;
use crate::back_end::llvm_codegen::callconventions::CallConvention;
use crate::back_end::llvm_codegen::context::LLVMCodeGenContext;
use crate::back_end::llvm_codegen::typegen;
use crate::back_end::llvm_codegen::types::repr::LLVMAttributes;
use crate::back_end::llvm_codegen::types::traits::LLVMAttributesExtensions;

use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

use crate::core::diagnostic::span::Span;

use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::repr::Intrinsic;
use crate::front_end::typesystem::types::Type;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, intrinsic: Intrinsic<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();
    let llvm_context: &Context = context.get_llvm_context();

    let name: &str = intrinsic.0;
    let external_name: &str = intrinsic.1;
    let return_type: &Type = intrinsic.2;
    let parameters: &[Ast<'ctx>] = intrinsic.3;
    let parameters_types: &[Type] = intrinsic.4;
    let attributes: LLVMAttributes = intrinsic.5.as_llvm_attributes();

    let span: Span = intrinsic.6;

    let ignore_args: bool = attributes.has_ignore_attribute();

    let convention: u32 = if let Some(LLVMAttribute::Convention(conv, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Convention)
    {
        conv as u32
    } else {
        CallConvention::Standard as u32
    };

    let function_type: FunctionType =
        typegen::generate_fn_type(context, return_type, parameters, ignore_args);

    let llvm_function: FunctionValue = llvm_module.add_function(external_name, function_type, None);

    AttributeBuilder::new(
        llvm_context,
        &attributes,
        LLVMAttributeApplicant::Function(llvm_function),
    )
    .add_function_attributes();

    context.new_function(name, (llvm_function, parameters_types, convention, span));
}
