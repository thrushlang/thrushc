#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use {
    super::backends::LLVMBackend,
    crate::{
        frontend::lexer::token::Token,
        middle::types::frontend::parser::stmts::stmt::ThrushStatement,
    },
    inkwell::OptimizationLevel,
    std::path::PathBuf,
};

#[derive(Debug)]
pub struct CompilerOptions {
    use_llvm_backend: bool,
    llvm_backend: LLVMBackend,
    files: Vec<CompilerFile>,
    build_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CompilerFile {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub enum Emitable {
    RawLLVMIR,
    RawLLVMBitcode,
    LLVMBitcode,
    LLVMIR,
    Object,
    RawAssembly,
    Assembly,
    AST,
    Tokens,
}

#[derive(Debug)]
pub enum Emited<'emited> {
    Tokens(&'emited [Token<'emited>]),
    Statements(&'emited [ThrushStatement<'emited>]),
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ThrushOptimization {
    #[default]
    None,
    Size,
    Low,
    Mid,
    Mcqueen,
}

impl ThrushOptimization {
    #[inline]
    pub fn to_llvm_opt(self) -> OptimizationLevel {
        match self {
            ThrushOptimization::None => OptimizationLevel::None,
            ThrushOptimization::Low => OptimizationLevel::Default,
            ThrushOptimization::Mid => OptimizationLevel::Less,
            ThrushOptimization::Mcqueen | ThrushOptimization::Size => OptimizationLevel::Aggressive,
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
