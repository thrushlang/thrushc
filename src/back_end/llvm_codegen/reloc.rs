use inkwell::targets::RelocMode;

pub trait LLVMRelocModeExtensions {
    fn to_metadata_repr(&self) -> u64;
    fn is_no_pic(&self) -> bool;
    fn is_pic(&self) -> bool;
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

    #[inline]
    fn is_no_pic(&self) -> bool {
        matches!(self, RelocMode::Static | RelocMode::Default)
    }

    #[inline]
    fn is_pic(&self) -> bool {
        matches!(self, RelocMode::PIC)
    }
}
