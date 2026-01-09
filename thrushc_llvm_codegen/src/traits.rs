use inkwell::values::FunctionValue;

use thrushc_span::Span;
use thrushc_typesystem::Type;

pub trait AstLLVMGetType {
    fn llvm_get_type(&self) -> &Type;
}

pub trait LLVMFunctionExtensions<'ctx> {
    fn get_value(&self) -> FunctionValue<'ctx>;
    fn get_return_type(&self) -> &'ctx Type;
    fn get_call_convention(&self) -> u32;
    fn get_param_count(&self) -> usize;
    fn get_parameters_types(&self) -> &[Type];
    fn get_span(&self) -> Span;
}

pub trait LLVMDBGFunctionExtensions<'ctx> {
    fn get_value(&self) -> FunctionValue<'ctx>;
    fn get_name(&self) -> &str;
    fn get_return_type(&self) -> &'ctx Type;
    fn get_parameters_types(&self) -> Vec<Type>;
    fn get_span(&self) -> Span;

    fn is_definition(&self) -> bool;
    fn is_local(&self) -> bool;
}
