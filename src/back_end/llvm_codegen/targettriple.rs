use inkwell::targets::TargetTriple;

#[derive(Debug)]
pub struct LLVMTargetTriple {
    arch: String,
    vendor: String,
    os: String,
    abi: String,
}

impl LLVMTargetTriple {
    #[inline]
    pub fn new(triple: &TargetTriple) -> Self {
        let triple_str: String = triple.as_str().to_string_lossy().to_string();
        let triple_dissasembled: Vec<&str> = triple_str.split('-').collect();

        let arch: String = triple_dissasembled
            .first()
            .unwrap_or(&"unknown")
            .to_string();

        let vendor: String = triple_dissasembled.get(1).unwrap_or(&"unknown").to_string();
        let os: String = triple_dissasembled.get(2).unwrap_or(&"unknown").to_string();
        let abi: String = triple_dissasembled.get(3).unwrap_or(&"unknown").to_string();

        Self {
            arch,
            vendor,
            os,
            abi,
        }
    }
}

impl LLVMTargetTriple {
    #[inline]
    pub fn get_abi(&self) -> &str {
        &self.abi
    }

    #[inline]
    pub fn get_arch(&self) -> &str {
        &self.arch
    }

    #[inline]
    pub fn get_os(&self) -> &str {
        &self.os
    }

    #[inline]
    pub fn get_vendor(&self) -> &str {
        &self.vendor
    }
}

impl LLVMTargetTriple {
    #[inline]
    pub fn has_posix_thread_model(&self) -> bool {
        matches!(
            self.get_os(),
            "linux" | "android" | "freebsd" | "netbsd" | "openbsd"
        ) || matches!(self.get_abi(), "gnu")
    }
}
