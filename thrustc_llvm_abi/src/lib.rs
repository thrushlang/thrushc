use thrustc_llvm_abi_x86::X86SystemVABI;

#[derive(Debug)]
pub enum FunctionABI {
    X86SystemV(X86SystemVABI),
}

impl FunctionABI {
    #[inline]
    pub fn is_x86_system_v(&self) -> bool {
        matches!(self, FunctionABI::X86SystemV(..))
    }
}
