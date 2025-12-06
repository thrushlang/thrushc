use inkwell::targets::CodeModel;

pub trait LLVMCodeModelExtensions {
    fn to_metadata_repr(&self) -> u64;
}

impl LLVMCodeModelExtensions for CodeModel {
    fn to_metadata_repr(&self) -> u64 {
        match self {
            CodeModel::Default => 0,
            CodeModel::JITDefault => 1,
            CodeModel::Small => 2,
            CodeModel::Kernel => 3,
            CodeModel::Medium => 4,
            CodeModel::Large => 5,
        }
    }
}
