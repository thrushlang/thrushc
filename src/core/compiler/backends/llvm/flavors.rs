use lld::flavor::LLDFlavor;

#[derive(Debug)]
pub enum LLVMLinkerFlavor {
    Elf,
    Wasm = 1,
    MachO = 2,
    Coff = 3,
}

impl LLVMLinkerFlavor {
    #[inline]
    pub fn to_lld_pure_flavor(&self) -> LLDFlavor {
        match self {
            LLVMLinkerFlavor::Elf => LLDFlavor::Elf,
            LLVMLinkerFlavor::Coff => LLDFlavor::Coff,
            LLVMLinkerFlavor::MachO => LLDFlavor::MachO,
            LLVMLinkerFlavor::Wasm => LLDFlavor::Wasm,
        }
    }
}

impl LLVMLinkerFlavor {
    #[inline]
    pub fn raw_to_lld_flavor(raw: &str) -> LLVMLinkerFlavor {
        match raw {
            "elf" => LLVMLinkerFlavor::Elf,
            "coff" => LLVMLinkerFlavor::Coff,
            "mach0" => LLVMLinkerFlavor::MachO,
            "wasm" => LLVMLinkerFlavor::Wasm,

            _ => {
                if cfg!(target_os = "linux") {
                    return LLVMLinkerFlavor::Elf;
                }

                if cfg!(target_os = "windows") {
                    return LLVMLinkerFlavor::Coff;
                }

                if cfg!(target_os = "macos") {
                    return LLVMLinkerFlavor::MachO;
                }

                LLVMLinkerFlavor::Elf
            }
        }
    }
}
