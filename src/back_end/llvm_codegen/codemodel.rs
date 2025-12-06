use inkwell::targets::CodeModel;

pub trait LLVMCodeModelExtensions {
    fn to_metadata_repr(&self) -> u64;
}

impl LLVMCodeModelExtensions for CodeModel {
    fn to_metadata_repr(&self) -> u64 {
        match self {
            CodeModel::Default => 0,
            CodeModel::JITDefault => 0,
            CodeModel::Small => 0,
            CodeModel::Kernel => 2,
            CodeModel::Medium => 3,
            CodeModel::Large => 4,
        }
    }
}
