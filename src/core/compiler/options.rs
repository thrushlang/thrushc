#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use {
    crate::{
        core::compiler::backends::llvm::LLVMBackend,
        frontend::types::{ast::Ast, lexer::types::Tokens},
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

    clean_tokens: bool,
    clean_assembler: bool,
    clean_object: bool,
    clean_llvm_ir: bool,
    clean_llvm_bitcode: bool,
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
    Tokens(&'emited Tokens),
    Ast(&'emited [Ast<'emited>]),
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

    #[inline]
    pub fn is_none_opt(self) -> bool {
        matches!(self, ThrushOptimization::None)
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

            clean_tokens: false,
            clean_assembler: false,
            clean_object: false,
            clean_llvm_ir: false,
            clean_llvm_bitcode: false,
        }
    }
}

impl CompilerOptions {
    pub fn new_file(&mut self, name: String, path: PathBuf) {
        if self.files.iter().any(|file| file.path == path) {
            return;
        }

        self.files.push(CompilerFile::new(name, path));
    }
}

impl CompilerOptions {
    pub fn set_use_llvm_backend(&mut self, use_llvm_backend: bool) {
        self.use_llvm_backend = use_llvm_backend;
    }

    pub fn set_build_dir(&mut self, build_dir: PathBuf) {
        self.build_dir = build_dir;
    }

    pub fn set_clean_tokens(&mut self) {
        self.clean_tokens = true;
    }

    pub fn set_clean_assembler(&mut self) {
        self.clean_assembler = true;
    }

    pub fn set_clean_object(&mut self) {
        self.clean_object = true;
    }

    pub fn set_clean_llvm_ir(&mut self) {
        self.clean_llvm_ir = true;
    }

    pub fn set_clean_llvm_bitcode(&mut self) {
        self.clean_llvm_bitcode = true;
    }
}

impl CompilerOptions {
    pub fn get_use_llvm(&self) -> bool {
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

    pub fn get_clean_tokens(&self) -> bool {
        self.clean_tokens
    }

    pub fn get_clean_assembler(&self) -> bool {
        self.clean_assembler
    }

    pub fn get_clean_object(&self) -> bool {
        self.clean_object
    }

    pub fn get_clean_llvm_ir(&self) -> bool {
        self.clean_llvm_ir
    }

    pub fn get_clean_llvm_bitcode(&self) -> bool {
        self.clean_llvm_bitcode
    }

    pub fn is_build_dir_setted(&self) -> bool {
        self.build_dir.exists()
    }
}
