use {
    inkwell::{
        targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
        OptimizationLevel,
    },
    std::path::PathBuf,
};

#[derive(Debug)]
pub struct CompilerOptions {
    pub output: String,
    pub target_triple: TargetTriple,
    pub optimization: Opt,
    pub emit_llvm_ir: bool,
    pub emit_raw_llvm_ir: bool,
    pub emit_llvm_bitcode: bool,
    pub emit_asm: bool,
    pub emit_thrush_ast: bool,
    pub emit_natives_apart: bool,
    pub library: bool,
    pub static_library: bool,
    pub executable: bool,
    pub linking: Linking,
    pub include_vector_api: bool,
    pub include_debug_api: bool,
    pub reloc_mode: RelocMode,
    pub code_model: CodeModel,
    pub files: Vec<ThrushFile>,
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

#[derive(Default, Debug)]
pub enum Linking {
    #[default]
    Static,
    Dynamic,
}

#[derive(Debug, Clone)]
pub struct ThrushFile {
    pub name: String,
    pub path: PathBuf,
}

impl Opt {
    #[inline(always)]
    pub const fn to_str(&self, single_slash: bool) -> &str {
        match self {
            Opt::None if !single_slash => "O0",
            Opt::Low if !single_slash => "O1",
            Opt::Mid if !single_slash => "O2",
            Opt::Mcqueen if !single_slash => "O3",
            Opt::Size if !single_slash => "Oz",
            Opt::None if single_slash => "-O0",
            Opt::Low if single_slash => "-O1",
            Opt::Mid if single_slash => "-O2",
            Opt::Mcqueen if single_slash => "-O3",
            Opt::Size if single_slash => "-Oz",
            _ => "-O0",
        }
    }

    #[inline(always)]
    pub const fn as_llvm_lto_opt(&self) -> &str {
        match self {
            Opt::None => "-O0",
            Opt::Low => "-O1",
            Opt::Mid => "-O2",
            Opt::Mcqueen | Opt::Size => "-O3",
        }
    }

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

impl Linking {
    #[inline]
    pub const fn to_str(&self) -> &str {
        match self {
            Linking::Static => "--static",
            Linking::Dynamic => "-dynamic",
        }
    }
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            output: String::new(),
            target_triple: TargetMachine::get_default_triple(),
            optimization: Opt::default(),
            emit_llvm_ir: false,
            emit_raw_llvm_ir: false,
            emit_llvm_bitcode: false,
            emit_natives_apart: false,
            emit_asm: false,
            emit_thrush_ast: false,
            library: false,
            static_library: false,
            executable: false,
            linking: Linking::default(),
            include_vector_api: false,
            include_debug_api: false,
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            files: Vec::new(),
            args: Vec::new(),
        }
    }
}
