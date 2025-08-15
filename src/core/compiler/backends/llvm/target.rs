use inkwell::targets::TargetTriple;

#[derive(Debug)]
pub struct LLVMTarget {
    pub name: String,
    pub target_triple: TargetTriple,
}

impl LLVMTarget {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn get_triple(&self) -> &TargetTriple {
        &self.target_triple
    }
}

impl LLVMTarget {
    #[inline]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[inline]
    pub fn set_target_triple(&mut self, triple: TargetTriple) {
        self.target_triple = triple;
    }
}
