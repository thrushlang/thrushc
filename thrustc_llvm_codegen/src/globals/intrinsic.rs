use thrustc_ast::Ast;
use thrustc_entities::Intrinsic;
use thrustc_llvm_attributes::LLVMAttribute;
use thrustc_llvm_attributes::LLVMAttributeComparator;
use thrustc_llvm_attributes::LLVMAttributes;
use thrustc_llvm_attributes::traits::LLVMAttributesExtensions;
use thrustc_llvm_callconventions::LLVMCallConvention;
use thrustc_span::Span;
use thrustc_typesystem::Type;

use crate::attrbuilder::AttributeBuilder;
use crate::attrbuilder::LLVMAttributeApplicant;
use crate::context::LLVMCodeGenContext;
use crate::typegeneration;
use crate::types::LLVMFunction;

use inkwell::module::Module;
use inkwell::types::FunctionType;
use inkwell::values::FunctionValue;

pub fn compile<'ctx>(context: &mut LLVMCodeGenContext<'_, 'ctx>, intrinsic: Intrinsic<'ctx>) {
    let llvm_module: &Module = context.get_llvm_module();

    let name: &str = intrinsic.0;
    let external_name: &str = intrinsic.1;
    let return_type: &Type = intrinsic.2;
    let parameters: &[Ast<'ctx>] = intrinsic.3;
    let parameters_types: &[Type] = intrinsic.4;
    let attributes: LLVMAttributes = thrustc_llvm_attributes::into_llvm_attributes(intrinsic.5);
    let span: Span = intrinsic.6;

    let ignore_args: bool = attributes.has_ignore_attribute();

    let convention: u32 = if let Some(LLVMAttribute::Convention(conv, ..)) =
        attributes.get_attr(LLVMAttributeComparator::Convention)
    {
        conv as u32
    } else {
        LLVMCallConvention::Standard as u32
    };

    let function_type: FunctionType =
        typegeneration::compile_as_function_type(context, return_type, parameters, ignore_args);

    let llvm_function: FunctionValue = llvm_module.add_function(external_name, function_type, None);

    AttributeBuilder::new(attributes, LLVMAttributeApplicant::Function(llvm_function))
        .add_function_attributes(context);

    let prototype: LLVMFunction = (
        llvm_function,
        return_type,
        parameters_types,
        convention,
        span,
    );

    context.new_function(name, prototype);
}
