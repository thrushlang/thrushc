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

    #[cfg_attr(target_vendor = "apple", inline)]
    pub fn get_macos_version(&self) -> Option<(u64, u64, u64)> {
        let macos_version: &str = self.macos_version.as_ref()?;
        let mut split: std::str::Split<'_, char> = macos_version.split('.');

        let major: u64 = split.next()?.parse::<u64>().ok()?;
        let minor: u64 = split.next()?.parse::<u64>().ok()?;
        let patch: u64 = split.next()?.parse::<u64>().ok()?;

        Some((major, minor, patch))
    }

    #[cfg_attr(target_vendor = "apple", inline)]
    pub fn get_ios_version(&self) -> Option<(u64, u64, u64)> {
        let ios_version: &str = self.ios_version.as_ref()?;
        let mut split: std::str::Split<'_, char> = ios_version.split('.');

        let major: u64 = split.next()?.parse::<u64>().ok()?;
        let minor: u64 = split.next()?.parse::<u64>().ok()?;
        let patch: u64 = split.next()?.parse::<u64>().ok()?;

        Some((major, minor, patch))
    }

    #[inline]
    pub fn dissamble_target_triple(&self) -> (String, String, String, String) {
        let triple: std::borrow::Cow<'_, str> = self.target_triple.as_str().to_string_lossy();
        let mut split: std::str::Split<'_, char> = triple.split('-');

        let arch: String = split.next().unwrap_or_default().to_string();
        let vendor: String = split.next().unwrap_or_default().to_string();
        let os: String = split.next().unwrap_or_default().to_string();
        let abi: String = split.next().unwrap_or_default().to_string();

        (arch, vendor, os, abi)
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
