#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use {
    inkwell::{
        OptimizationLevel,
        targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    },
    std::path::PathBuf,
};

#[derive(Debug)]
pub struct CompilerOptions {
    flag_position: FlagsPosition,
    llvm_backend: bool,
    llvm_backend_options: LLVMBackendOptions,
    files: Vec<CompilerFile>,
}

#[derive(Debug)]
pub struct LLVMBackendOptions {
    target_triple: TargetTriple,
    optimization: Opt,
    emit: Vec<Emitable>,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    static_compiler_args: Vec<String>,
    linker_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompilerFile {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub enum Emitable {
    LLVMIR,
    RawLLVMIR,
    LLVMBitcode,
    Assembly,
    AST,
    Tokens,
}

#[derive(Debug, Clone, Copy)]
pub enum FlagsPosition {
    ThrushCompiler,
    LLVMLinker,
    LLVMStaticCompiler,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Opt {
    #[default]
    None,
    Size,
    Low,
    Mid,
    Mcqueen,
}

impl FlagsPosition {
    pub fn llvm_linker(&self) -> bool {
        matches!(self, FlagsPosition::LLVMLinker)
    }

    pub fn llvm_static_compiler(&self) -> bool {
        matches!(self, FlagsPosition::LLVMStaticCompiler)
    }

    pub fn thrush_compiler(&self) -> bool {
        matches!(self, FlagsPosition::ThrushCompiler)
    }
}

impl Opt {
    #[inline]
    pub fn to_llvm_opt(self) -> OptimizationLevel {
        match self {
            Opt::None => OptimizationLevel::None,
            Opt::Low => OptimizationLevel::Default,
            Opt::Mid => OptimizationLevel::Less,
            Opt::Mcqueen | Opt::Size => OptimizationLevel::Aggressive,
        }
    }

    #[inline]
    pub fn to_llvm_17_passes(self) -> &'static str {
        match self {
            Opt::None => "default<O0>",
            Opt::Low => "default<O1>",
            Opt::Mid => "default<O2>",
            Opt::Mcqueen => "default<O3>",
            Opt::Size => "default<Oz>",
        }
    }
}

impl CompilerOptions {
    pub fn new() -> Self {
        Self {
            flag_position: FlagsPosition::ThrushCompiler,
            llvm_backend: false,
            llvm_backend_options: LLVMBackendOptions::default(),
            files: Vec::with_capacity(100),
        }
    }

    pub fn use_llvm(&self) -> bool {
        self.llvm_backend
    }

    pub fn get_files(&self) -> &[CompilerFile] {
        self.files.as_slice()
    }

    pub fn get_llvm_backend_options(&self) -> &LLVMBackendOptions {
        &self.llvm_backend_options
    }

    pub fn get_mut_llvm_backend_options(&mut self) -> &mut LLVMBackendOptions {
        &mut self.llvm_backend_options
    }

    pub fn get_flag_position(&self) -> FlagsPosition {
        self.flag_position
    }

    pub fn add_file(&mut self, name: String, path: PathBuf) {
        self.files.push(CompilerFile { name, path });
    }

    pub fn set_flag_position(&mut self, flag_position: FlagsPosition) {
        self.flag_position = flag_position;
    }

    pub fn set_use_llvm_backend(&mut self, llvm_backend: bool) {
        self.llvm_backend = llvm_backend;
    }
}

impl LLVMBackendOptions {
    pub fn get_linker_arguments(&self) -> &[String] {
        self.linker_args.as_slice()
    }

    pub fn get_static_compiler_arguments(&self) -> &[String] {
        self.static_compiler_args.as_slice()
    }

    pub fn get_reloc_mode(&self) -> RelocMode {
        self.reloc_mode
    }

    pub fn get_code_model(&self) -> CodeModel {
        self.code_model
    }

    pub fn set_optimization(&mut self, opt: Opt) {
        self.optimization = opt;
    }

    pub fn set_reloc_mode(&mut self, reloc_mode: RelocMode) {
        self.reloc_mode = reloc_mode;
    }

    pub fn set_code_model(&mut self, code_model: CodeModel) {
        self.code_model = code_model;
    }

    pub fn add_static_compiler_argument(&mut self, arg: String) {
        self.static_compiler_args.push(arg);
    }

    pub fn add_linker_argument(&mut self, arg: String) {
        self.linker_args.push(arg);
    }

    pub fn set_target_triple(&mut self, target_triple: TargetTriple) {
        self.target_triple = target_triple;
    }

    pub fn add_emit_option(&mut self, emit: Emitable) {
        self.emit.push(emit);
    }

    pub fn get_target_triple(&self) -> &TargetTriple {
        &self.target_triple
    }

    pub fn get_optimization(&self) -> Opt {
        self.optimization
    }

    pub fn contains_emitable(&self, emit: Emitable) -> bool {
        self.emit.contains(&emit)
    }
}

impl Default for LLVMBackendOptions {
    fn default() -> Self {
        Self {
            target_triple: TargetMachine::get_default_triple(),
            optimization: Opt::None,
            emit: Vec::with_capacity(10),
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            static_compiler_args: Vec::with_capacity(100),
            linker_args: Vec::with_capacity(100),
        }
    }
}
