#![allow(non_camel_case_types, clippy::upper_case_acronyms)]

use std::path::Path;
use std::path::PathBuf;

use inkwell::OptimizationLevel;

use crate::core::compiler::backends::linkers::LinkerMode;
use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::linking::LinkingCompilersConfiguration;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::lexer::types::Tokens;

#[derive(Debug)]
pub struct CompilerOptions {
    use_llvm_backend: bool,
    llvm_backend: LLVMBackend,
    files: Vec<CompilationUnit>,
    build_dir: PathBuf,

    emit: Vec<EmitableUnit>,
    printable: Vec<PrintableUnit>,

    disable_default_opt: bool,
    clean_tokens: bool,
    clean_assembler: bool,
    clean_object: bool,
    clean_llvm_ir: bool,
    clean_llvm_bitcode: bool,
    clean_build: bool,
    obfuscate_archive_names: bool,
    obfuscate_ir: bool,

    linking_compilers_config: LinkingCompilersConfiguration,
    linker_mode: LinkerMode,
}

#[derive(Debug, Clone)]
pub struct CompilationUnit {
    name: String,
    base_name: String,
    path: PathBuf,
    content: String,
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
    RawAssembly,
    Assembly,
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
    pub fn is_opt(self) -> bool {
        matches!(
            self,
            ThrushOptimization::Low
                | ThrushOptimization::Mid
                | ThrushOptimization::Mcqueen
                | ThrushOptimization::Size
        )
    }
}

impl CompilationUnit {
    #[inline]
    pub fn new(name: String, path: PathBuf, content: String, base_name: String) -> Self {
        Self {
            name,
            path,
            content,
            base_name,
        }
    }
}

impl CompilerOptions {
    #[inline]
    pub fn new() -> Self {
        Self {
            use_llvm_backend: false,
            llvm_backend: LLVMBackend::new(),
            files: Vec::with_capacity(1000),

            emit: Vec::with_capacity(10),
            printable: Vec::with_capacity(10),

            build_dir: PathBuf::new(),

            disable_default_opt: false,
            clean_tokens: false,
            clean_assembler: false,
            clean_object: false,
            clean_llvm_ir: false,
            clean_llvm_bitcode: false,
            clean_build: false,
            obfuscate_archive_names: true,
            obfuscate_ir: true,

            linking_compilers_config: LinkingCompilersConfiguration::new(),
            linker_mode: LinkerMode::new(Vec::with_capacity(50)),
        }
    }
}

impl CompilerOptions {
    #[inline]
    pub fn add_compilation_unit(
        &mut self,
        name: String,
        path: PathBuf,
        content: String,
        base_name: String,
    ) {
        if self.files.iter().any(|file| file.path == path) {
            logging::print_warn(
                LoggingType::Warning,
                &format!("File skipped due to repetition '{}'.", path.display()),
            );

            return;
        }

        self.files
            .push(CompilationUnit::new(name, path, content, base_name));
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
    pub fn set_clean_build(&mut self) {
        self.clean_build = true;
    }

    #[inline]
    pub fn set_disable_default_opt(&mut self) {
        self.disable_default_opt = true;
    }

    #[inline]
    pub fn set_no_obfuscate_archive_names(&mut self) {
        self.obfuscate_archive_names = false;
    }

    #[inline]
    pub fn set_no_obfuscate_ir(&mut self) {
        self.obfuscate_ir = false;
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
    pub fn get_files(&self) -> &[CompilationUnit] {
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
    pub fn get_clean_build(&self) -> bool {
        self.clean_build
    }

    #[inline]
    pub fn is_build_dir_setted(&self) -> bool {
        self.build_dir.exists()
    }

    #[inline]
    pub fn need_obfuscate_archive_names(&self) -> bool {
        self.obfuscate_archive_names
    }

    #[inline]
    pub fn need_obfuscate_ir(&self) -> bool {
        self.obfuscate_ir
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
    pub fn disable_default_opt(&self) -> bool {
        self.disable_default_opt
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

impl CompilationUnit {
    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn get_unit_content(&self) -> &str {
        &self.content
    }

    #[inline]
    pub fn get_unit_clone(&self) -> String {
        self.content.clone()
    }

    #[inline]
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn get_base_name(&self) -> String {
        self.base_name.clone()
    }
}
