use inkwell::{InlineAsmDialect, values::FunctionValue};

use crate::{
    back_end::llvm_codegen::attributes::{LLVMAttribute, LLVMAttributeComparator},
    front_end::typesystem::types::Type,
};

pub trait AssemblerFunctionExtensions {
    fn as_inline_assembler_dialect(syntax: &str) -> InlineAsmDialect;
}

pub trait LLVMAttributesExtensions {
    fn has_extern_attribute(&self) -> bool;
    fn has_ignore_attribute(&self) -> bool;
    fn has_public_attribute(&self) -> bool;
    fn has_linkage_attribute(&self) -> bool;
    fn has_hot_attr(&self) -> bool;
    fn has_inline_attr(&self) -> bool;
    fn has_noinline_attr(&self) -> bool;
    fn has_minsize_attr(&self) -> bool;
    fn has_inlinealways_attr(&self) -> bool;

    fn has_heap_attr(&self) -> bool;

    fn has_asmalignstack_attribute(&self) -> bool;
    fn has_asmthrow_attribute(&self) -> bool;
    fn has_asmsideffects_attribute(&self) -> bool;
    fn has_asmsyntax_attribute(&self) -> bool;

    fn get_attr(&self, cmp: LLVMAttributeComparator) -> Option<LLVMAttribute<'_>>;
}

pub trait LLVMAttributeComparatorExtensions {
    fn as_attr_cmp(&self) -> LLVMAttributeComparator;
}

pub trait LLVMLinkageExtensions {
    fn fmt(&self) -> &'static str;
}

pub trait LLVMFunctionExtensions<'ctx> {
    fn get_value(&self) -> FunctionValue<'ctx>;
    fn get_return_type(&self) -> &'ctx Type;
    fn get_call_convention(&self) -> u32;
    fn get_param_count(&self) -> usize;
    fn get_parameters_types(&self) -> &[Type];
}
