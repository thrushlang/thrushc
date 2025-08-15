use inkwell::targets::TargetTriple;

#[derive(Debug)]
pub struct LLVMTarget {
    pub arch: String,
    pub target_triple: TargetTriple,
}

impl LLVMTarget {
    #[inline]
    pub fn get_arch(&self) -> &str {
        &self.arch
    }

    #[inline]
    pub fn get_triple(&self) -> &TargetTriple {
        &self.target_triple
    }
}

impl LLVMTarget {
    #[inline]
    pub fn set_arch(&mut self, arch: String) {
        self.arch = arch;
    }

    #[inline]
    pub fn set_target_triple(&mut self, triple: TargetTriple) {
        self.target_triple = triple;
    }
}
