use inkwell::targets::RelocMode;

pub trait LLVMRelocModeExtensions {
    fn to_metadata_repr(&self) -> u64;
}

impl LLVMRelocModeExtensions for RelocMode {
    #[inline]
    fn to_metadata_repr(&self) -> u64 {
        match self {
            RelocMode::Static | RelocMode::Default => 0,
            RelocMode::PIC => 1,
            RelocMode::DynamicNoPic => 1,
        }
    }
}
