use {
    inkwell::{
        OptimizationLevel,
        targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    },
    std::path::PathBuf,
};

#[derive(Debug)]
pub struct CompilerOptions {
    pub target_triple: TargetTriple,
    pub optimization: Opt,
    pub emit_llvm_ir: bool,
    pub emit_raw_llvm_ir: bool,
    pub emit_llvm_bitcode: bool,
    pub emit_asm: bool,
    pub emit_ast: bool,
    pub reloc_mode: RelocMode,
    pub code_model: CodeModel,
    pub files: Vec<CompilerFile>,
    pub args: Vec<String>,
}

#[derive(Default, Debug)]
pub enum Opt {
    #[default]
    None,
    Size,
    Low,
    Mid,
    Mcqueen,
}

#[derive(Debug, Clone)]
pub struct CompilerFile {
    pub name: String,
    pub path: PathBuf,
}

impl Opt {
    #[inline(always)]
    pub const fn to_llvm_opt(&self) -> OptimizationLevel {
        match self {
            Opt::None => OptimizationLevel::None,
            Opt::Low => OptimizationLevel::Default,
            Opt::Mid => OptimizationLevel::Less,
            Opt::Mcqueen | Opt::Size => OptimizationLevel::Aggressive,
        }
    }

    #[inline(always)]
    pub const fn to_llvm_17_passes(&self) -> &str {
        match self {
            Opt::None => "default<O0>",
            Opt::Low => "default<O1>",
            Opt::Mid => "default<O2>",
            Opt::Mcqueen => "default<O3>",
            Opt::Size => "default<Oz>",
        }
    }
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target_triple: TargetMachine::get_default_triple(),
            optimization: Opt::default(),
            emit_llvm_ir: false,
            emit_raw_llvm_ir: false,
            emit_llvm_bitcode: false,
            emit_asm: false,
            emit_ast: false,
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            files: Vec::with_capacity(20),
            args: Vec::with_capacity(20),
        }
    }
}
