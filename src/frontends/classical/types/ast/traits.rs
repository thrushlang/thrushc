pub trait LLVMAstExtensions {
    fn is_llvm_constant_value(&self) -> bool;
}

pub trait AstExtensions {
    fn is_lli(&self) -> bool;
}
