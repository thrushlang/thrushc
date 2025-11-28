pub const LLVM_HOSTS_TARGETS_AVAILIABLE: &[&str] = &[
    "x86_64-pc-windows-gnu",
    "x86_64-pc-windows-libcmt",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
];

#[derive(Debug)]
pub enum LLVMBackendHostTarget {
    WindowsGNU,
    WindowsLIBCMT,
    WindowsMSVC,
    LinuxGNU,
    LinuxMusl,
}

impl LLVMBackendHostTarget {
    #[inline]
    pub fn get_triple(&self) -> &'static str {
        match self {
            LLVMBackendHostTarget::WindowsGNU => "x86_64-pc-windows-gnu",
            LLVMBackendHostTarget::WindowsLIBCMT => "x86_64-pc-windows-libcmt",
            LLVMBackendHostTarget::WindowsMSVC => "x86_64-pc-windows-msvc",
            LLVMBackendHostTarget::LinuxGNU => "x86_64-unknown-linux-gnu",
            LLVMBackendHostTarget::LinuxMusl => "x86_64-unknown-linux-musl",
        }
    }
}

impl LLVMBackendHostTarget {
    #[inline]
    pub fn is_windows_gnu(&self) -> bool {
        matches!(self, LLVMBackendHostTarget::WindowsGNU)
    }

    #[inline]
    pub fn is_windows_libcmt(&self) -> bool {
        matches!(self, LLVMBackendHostTarget::WindowsLIBCMT)
    }

    #[inline]
    pub fn is_windows_msvc(&self) -> bool {
        matches!(self, LLVMBackendHostTarget::WindowsMSVC)
    }

    #[inline]
    pub fn is_linux_gnu(&self) -> bool {
        matches!(self, LLVMBackendHostTarget::LinuxGNU)
    }

    #[inline]
    pub fn is_linux_musl(&self) -> bool {
        matches!(self, LLVMBackendHostTarget::LinuxMusl)
    }
}

#[inline]
pub fn parse_llvm_triple(triple: &str) -> Option<LLVMBackendHostTarget> {
    match triple {
        "x86_64-pc-windows-gnu" => Some(LLVMBackendHostTarget::WindowsGNU),
        "x86_64-pc-windows-libcmt" => Some(LLVMBackendHostTarget::WindowsLIBCMT),
        "x86_64-pc-windows-msvc" => Some(LLVMBackendHostTarget::WindowsMSVC),
        "x86_64-unknown-linux-gnu" => Some(LLVMBackendHostTarget::LinuxGNU),
        "x86_64-unknown-linux-musl" => Some(LLVMBackendHostTarget::LinuxMusl),
        _ => None,
    }
}
