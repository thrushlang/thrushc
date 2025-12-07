use inkwell::targets::TargetTriple;

#[derive(Debug)]
pub struct LLVMTarget {
    pub arch: String,
    pub target_triple: TargetTriple,
    pub target_triple_darwin_variant: Option<TargetTriple>,
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

    #[inline]
    pub fn get_triple_darwin_variant(&self) -> Option<&TargetTriple> {
        self.target_triple_darwin_variant.as_ref()
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

    #[inline]
    pub fn set_target_triple_darwin_variant(&mut self, triple: TargetTriple) {
        self.target_triple_darwin_variant = Some(triple);
    }
}
