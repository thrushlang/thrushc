#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use {
    crate::{
        core::{
            compiler::{
                backends::{linkers::LinkerMode, llvm::LLVMBackend},
                linking::LinkingCompilersConfiguration,
            },
            console::logging::{self, LoggingType},
        },
        frontends::classical::types::{ast::Ast, lexer::types::Tokens},
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

    emit: Vec<EmitableUnit>,
    printable: Vec<PrintableUnit>,

    clean_tokens: bool,
    clean_assembler: bool,
    clean_object: bool,
    clean_llvm_ir: bool,
    clean_llvm_bitcode: bool,
    ofuscate_archive_names: bool,

    linking_compilers_config: LinkingCompilersConfiguration,
    linker_mode: LinkerMode,
}

#[derive(Debug, Clone)]
pub struct CompilerFile {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq)]
pub enum EmitableUnit {
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

#[derive(Debug, PartialEq)]
pub enum PrintableUnit {
    RawLLVMIR,
    LLVMIR,
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
            files: Vec::with_capacity(1000),

            emit: Vec::with_capacity(10),
            printable: Vec::with_capacity(10),

            build_dir: PathBuf::new(),

            clean_tokens: false,
            clean_assembler: false,
            clean_object: false,
            clean_llvm_ir: false,
            clean_llvm_bitcode: false,
            ofuscate_archive_names: true,

            linking_compilers_config: LinkingCompilersConfiguration::new(),
            linker_mode: LinkerMode::new(Vec::with_capacity(50)),
        }
    }
}

impl CompilerOptions {
    pub fn new_file(&mut self, name: String, path: PathBuf) {
        if self.files.iter().any(|file| file.path == path) {
            logging::log(
                LoggingType::Warning,
                &format!("File skipped due to repetition '{}'.", path.display()),
            );

            return;
        }

        self.files.push(CompilerFile::new(name, path));
    }
}

impl CompilerOptions {
    #[inline]
    pub fn set_use_llvm_backend(&mut self, use_llvm_backend: bool) {
        self.use_llvm_backend = use_llvm_backend;
    }

    #[inline]
    pub fn set_build_dir(&mut self, build_dir: PathBuf) {
        self.build_dir = build_dir;
    }

    #[inline]
    pub fn set_clean_tokens(&mut self) {
        self.clean_tokens = true;
    }

    #[inline]
    pub fn set_clean_assembler(&mut self) {
        self.clean_assembler = true;
    }

    #[inline]
    pub fn set_clean_object(&mut self) {
        self.clean_object = true;
    }

    #[inline]
    pub fn set_clean_llvm_ir(&mut self) {
        self.clean_llvm_ir = true;
    }

    #[inline]
    pub fn set_clean_llvm_bitcode(&mut self) {
        self.clean_llvm_bitcode = true;
    }

    #[inline]
    pub fn no_ofuscate_archive_names(&mut self) {
        self.ofuscate_archive_names = false;
    }

    #[inline]
    pub fn add_emit_option(&mut self, emit: EmitableUnit) {
        self.emit.push(emit);
    }

    #[inline]
    pub fn add_print_option(&mut self, printable: PrintableUnit) {
        self.printable.push(printable);
    }
}

impl CompilerOptions {
    #[inline]
    pub fn uses_llvm(&self) -> bool {
        self.use_llvm_backend
    }

    #[inline]
    pub fn get_files(&self) -> &[CompilerFile] {
        self.files.as_slice()
    }

    #[inline]
    pub fn get_llvm_backend_options(&self) -> &LLVMBackend {
        &self.llvm_backend
    }

    #[inline]
    pub fn get_build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    #[inline]
    pub fn get_clean_tokens(&self) -> bool {
        self.clean_tokens
    }

    #[inline]
    pub fn get_clean_assembler(&self) -> bool {
        self.clean_assembler
    }

    #[inline]
    pub fn get_clean_object(&self) -> bool {
        self.clean_object
    }

    #[inline]
    pub fn get_clean_llvm_ir(&self) -> bool {
        self.clean_llvm_ir
    }

    #[inline]
    pub fn get_clean_llvm_bitcode(&self) -> bool {
        self.clean_llvm_bitcode
    }

    #[inline]
    pub fn is_build_dir_setted(&self) -> bool {
        self.build_dir.exists()
    }

    #[inline]
    pub fn ofuscate_archive_names(&self) -> bool {
        self.ofuscate_archive_names
    }

    #[inline]
    pub fn get_was_emited(&self) -> bool {
        !self.emit.is_empty()
    }

    #[inline]
    pub fn get_was_printed(&self) -> bool {
        !self.printable.is_empty()
    }

    #[inline]
    pub fn contains_emitable(&self, emit: EmitableUnit) -> bool {
        self.emit.contains(&emit)
    }

    #[inline]
    pub fn contains_printable(&self, printable: PrintableUnit) -> bool {
        self.printable.contains(&printable)
    }

    #[inline]
    pub fn get_linker_mode(&self) -> &LinkerMode {
        &self.linker_mode
    }

    #[inline]
    pub fn get_linking_compilers_configuration(&self) -> &LinkingCompilersConfiguration {
        &self.linking_compilers_config
    }
}

impl CompilerOptions {
    #[inline]
    pub fn get_mut_llvm_backend_options(&mut self) -> &mut LLVMBackend {
        &mut self.llvm_backend
    }

    #[inline]
    pub fn get_mut_linker_mode(&mut self) -> &mut LinkerMode {
        &mut self.linker_mode
    }

    #[inline]
    pub fn get_mut_linking_compilers_configuration(
        &mut self,
    ) -> &mut LinkingCompilersConfiguration {
        &mut self.linking_compilers_config
    }
}
