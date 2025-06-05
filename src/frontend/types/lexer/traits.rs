use crate::{
    backend::llvm::compiler::context::LLVMCodeGenContext, frontend::types::lexer::ThrushType,
};

pub trait ThrushStructTypeExtensions {
    fn get_name(&self) -> String;
}

pub trait LLVMTypeExtensions {
    fn is_same_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &ThrushType) -> bool;
}
