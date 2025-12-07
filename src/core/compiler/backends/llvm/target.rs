use inkwell::targets::TargetTriple;

#[derive(Debug)]
pub struct LLVMTarget {
    pub arch: String,
    pub target_triple: TargetTriple,
    pub target_triple_darwin_variant: Option<TargetTriple>,
    pub macos_version: Option<String>,
    pub ios_version: Option<String>,
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

    #[inline]
    pub fn get_macos_version(&self) -> Option<(u64, u64, u64)> {
        let macos_version: &str = self.macos_version.as_ref()?;
        let mut split: std::str::Split<'_, char> = macos_version.split('.');

        let major: u64 = split.next()?.parse::<u64>().ok()?;
        let minor: u64 = split.next()?.parse::<u64>().ok()?;
        let patch: u64 = split.next()?.parse::<u64>().ok()?;

        Some((major, minor, patch))
    }

    #[inline]
    pub fn get_ios_version(&self) -> Option<(u64, u64, u64)> {
        let ios_version: &str = self.ios_version.as_ref()?;
        let mut split: std::str::Split<'_, char> = ios_version.split('.');

        let major: u64 = split.next()?.parse::<u64>().ok()?;
        let minor: u64 = split.next()?.parse::<u64>().ok()?;
        let patch: u64 = split.next()?.parse::<u64>().ok()?;

        Some((major, minor, patch))
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

    #[inline]
    pub fn set_macos_version(&mut self, version: String) {
        self.macos_version = Some(version);
    }

    #[inline]
    pub fn set_ios_version(&mut self, version: String) {
        self.ios_version = Some(version);
    }
}
