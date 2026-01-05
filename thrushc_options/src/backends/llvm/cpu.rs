#[derive(Debug)]
pub struct LLVMTargetCPU {
    pub target_cpu: String,
    pub target_cpu_features: String,
}

impl LLVMTargetCPU {
    #[inline]
    pub fn get_cpu_name(&self) -> &str {
        &self.target_cpu
    }

    #[inline]
    pub fn get_cpu_features(&self) -> &str {
        &self.target_cpu_features
    }
}

impl LLVMTargetCPU {
    #[inline]
    pub fn set_cpu_name(&mut self, name: String) {
        self.target_cpu = name;
    }

    #[inline]
    pub fn set_processador_features(&mut self, features: String) {
        self.target_cpu_features = features;
    }
}
