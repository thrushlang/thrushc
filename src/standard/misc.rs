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
    use_llvm_backend: bool,
    llvm_backend: LLVMBackend,
    files: Vec<CompilerFile>,
    build_dir: PathBuf,
}

#[derive(Debug)]
pub struct LLVMBackend {
    target_triple: TargetTriple,
    optimization: Opt,
    emit: Vec<Emitable>,
    reloc_mode: RelocMode,
    code_model: CodeModel,
    arguments: Vec<String>,
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

#[derive(Default, Debug, Clone, Copy)]
pub enum Opt {
    #[default]
    None,
    Size,
    Low,
    Mid,
    Mcqueen,
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
}

impl CompilerFile {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }
}

impl CompilerOptions {
    pub fn new() -> Self {
        Self {
            use_llvm_backend: false,
            llvm_backend: LLVMBackend::new(),
            files: Vec::with_capacity(100),
            build_dir: PathBuf::new(),
        }
    }

    pub fn use_llvm(&self) -> bool {
        self.use_llvm_backend
    }

    pub fn get_files(&self) -> &[CompilerFile] {
        self.files.as_slice()
    }

    pub fn get_llvm_backend_options(&self) -> &LLVMBackend {
        &self.llvm_backend
    }

    pub fn get_mut_llvm_backend_options(&mut self) -> &mut LLVMBackend {
        &mut self.llvm_backend
    }

    pub fn get_build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    pub fn new_file(&mut self, name: String, path: PathBuf) {
        self.files.push(CompilerFile::new(name, path));
    }

    pub fn set_use_llvm_backend(&mut self, use_llvm_backend: bool) {
        self.use_llvm_backend = use_llvm_backend;
    }

    pub fn set_build_dir(&mut self, build_dir: PathBuf) {
        self.build_dir = build_dir;
    }

    pub fn is_build_dir_setted(&self) -> bool {
        self.build_dir.exists()
    }
}

impl LLVMBackend {
    pub fn new() -> Self {
        Self {
            target_triple: TargetMachine::get_default_triple(),
            optimization: Opt::None,
            emit: Vec::with_capacity(10),
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            arguments: Vec::with_capacity(100),
        }
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

    pub fn set_target_triple(&mut self, target_triple: TargetTriple) {
        self.target_triple = target_triple;
    }

    pub fn add_emit_option(&mut self, emit: Emitable) {
        self.emit.push(emit);
    }

    pub fn get_target_triple(&self) -> &TargetTriple {
        &self.target_triple
    }

    pub fn get_opt(&self) -> Opt {
        self.optimization
    }

    pub fn was_emited(&self) -> bool {
        !self.emit.is_empty()
    }

    pub fn get_arguments(&self) -> &[String] {
        self.arguments.as_slice()
    }

    pub fn contains_emitable(&self, emit: Emitable) -> bool {
        self.emit.contains(&emit)
    }

    pub fn add_compiler_argument(&mut self, argument: String) {
        self.arguments.push(argument);
    }
}

impl Default for LLVMBackend {
    fn default() -> Self {
        Self {
            target_triple: TargetMachine::get_default_triple(),
            optimization: Opt::None,
            emit: Vec::with_capacity(10),
            reloc_mode: RelocMode::Default,
            code_model: CodeModel::Default,
            arguments: Vec::with_capacity(100),
        }
    }
}
