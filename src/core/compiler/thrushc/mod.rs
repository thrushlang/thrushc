mod emit;
mod finisher;
mod interrupt;
mod linking;
mod print;
mod validate;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use colored::Colorize;

use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{InitializationConfig, Target, TargetMachine, TargetTriple},
};

use crate::backends::classical::linking::linkers::lld::LLVMLinker;
use crate::backends::classical::llvm::{self, compiler::context::LLVMCodeGenContext};

use crate::core::compiler::backends::linkers::LinkerModeType;
use crate::core::compiler::backends::llvm::{LLVMBackend, target::LLVMTarget};
use crate::core::compiler::linking::LinkingCompilersConfiguration;
use crate::core::compiler::options::{
    CompilationUnit, CompilerOptions, Emited, ThrushOptimization,
};
use crate::core::console::logging::{self, LoggingType};
use crate::core::diagnostic::diagnostician::Diagnostician;

use crate::frontends::classical::lexer::{Lexer, token::Token};
use crate::frontends::classical::parser::{Parser, ParserContext};
use crate::frontends::classical::semantic::SemanticAnalyzer;
use crate::frontends::classical::types::ast::Ast;

#[derive(Debug)]
pub struct ThrushCompiler<'thrushc> {
    compiled: Vec<PathBuf>,
    uncompiled: &'thrushc [CompilationUnit],

    options: &'thrushc CompilerOptions,
    linking_time: Duration,
    thrushc_time: Duration,
}

impl<'thrushc> ThrushCompiler<'thrushc> {
    pub fn new(files: &'thrushc [CompilationUnit], options: &'thrushc CompilerOptions) -> Self {
        Self {
            compiled: Vec::with_capacity(files.len()),
            uncompiled: files,

            options,

            linking_time: Duration::default(),
            thrushc_time: Duration::default(),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        if self.get_options().uses_llvm() {
            Target::initialize_all(&InitializationConfig::default());
        } else {
            logging::write(
                logging::OutputIn::Stderr,
                &format!(
                    "{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FAILED".bright_red().bold(),
                    "GCC is not supported yet. Please use LLVM Infrastructure with '-llvm-backend' flag."
                ),
            );
        }

        if self.get_options().get_linker_mode().get_status() {
            if let LinkerModeType::LLVMLinker =
                self.get_options().get_linker_mode().get_linker_type()
            {
                LLVMLinker::new(self.options).link();
            }
        }

        let mut interrumped: bool = false;

        self.uncompiled.iter().for_each(|file| {
            interrumped = self.compile_with_llvm(file).is_err();
        });

        if interrumped
            || self.get_options().get_was_printed()
            || self.get_options().get_was_emited()
            || self.get_compiled_files().is_empty()
        {
            return (self.thrushc_time.as_millis(), self.linking_time.as_millis());
        }

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "RUNNING".bright_green().bold()
            ),
        );

        let linking_compiler_config: &LinkingCompilersConfiguration =
            self.options.get_linking_compilers_configuration();

        if linking_compiler_config.get_use_clang() {
            linking::link_with_clang(self);
        }

        if linking_compiler_config.get_use_gcc() {
            linking::link_with_gcc(self);
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }

    fn compile_with_llvm(&mut self, file: &'thrushc CompilationUnit) -> Result<(), ()> {
        let archive_time: Instant = Instant::now();

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "RUNNING".bright_green().bold(),
                &file.get_path().to_string_lossy()
            ),
        );

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        let tokens: Vec<Token> = Lexer::lex(file).unwrap_or_else(|error| {
            logging::print_frontend_panic(LoggingType::FrontEndPanic, &error.display())
        });

        if print::after_frontend(self, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation(self, archive_time, file);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation(self, archive_time, file);
        }

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file);

        let parser_result: (ParserContext, bool) = parser;
        let parser_throwed_errors: bool = parser_result.1;

        let parser_context: ParserContext = parser_result.0;

        let ast: &[Ast] = parser_context.get_ast();

        let semantic_analysis_throwed_errors: bool =
            SemanticAnalyzer::new(ast, file).check(parser_throwed_errors);

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            return finisher::archive_compilation(self, archive_time, file);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Ast(ast)) {
            return finisher::archive_compilation(self, archive_time, file);
        }

        let llvm_context: Context = Context::create();
        let llvm_builder: Builder = llvm_context.create_builder();
        let llvm_module: Module = llvm_context.create_module(file.get_name());

        let target: &LLVMTarget = llvm_backend.get_target();

        let llvm_triple: &TargetTriple = target.get_triple();

        let llvm_cpu_name: &str = llvm_backend.get_target_cpu().get_cpu_name();
        let llvm_cpu_features: &str = llvm_backend.get_target_cpu().get_cpu_features();

        let thrush_opt: ThrushOptimization = llvm_backend.get_optimization();
        let llvm_opt: OptimizationLevel = thrush_opt.to_llvm_opt();

        llvm_module.set_triple(llvm_triple);

        let target: Target = Target::from_triple(llvm_triple).unwrap_or_else(|_| {
            logging::print_frontend_panic(
                logging::LoggingType::BackendPanic,
                "Cannot generate a target from LLVM target triple.",
            );
        });

        let target_machine: TargetMachine = target
            .create_target_machine(
                llvm_triple,
                llvm_cpu_name,
                llvm_cpu_features,
                llvm_opt,
                llvm_backend.get_reloc_mode(),
                llvm_backend.get_code_model(),
            )
            .unwrap_or_else(|| {
                logging::print_frontend_panic(
                    logging::LoggingType::FrontEndPanic,
                    "Cannot generate a target machine from target.",
                );
            });

        llvm_module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        let mut llvm_codegen_context: LLVMCodeGenContext = LLVMCodeGenContext::new(
            &llvm_module,
            &llvm_context,
            &llvm_builder,
            target_machine.get_target_data(),
            Diagnostician::new(file),
        );

        llvm::compiler::LLVMCompiler::compile(&mut llvm_codegen_context, ast);

        validate::llvm_codegen(&llvm_module, file)?;

        if print::llvm_before_optimization(self, &llvm_module, file) {
            return finisher::archive_compilation(self, archive_time, file);
        }

        if emit::llvm_before_optimization(
            self,
            archive_time,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
        )? {
            return finisher::archive_compilation(self, archive_time, file);
        }

        if thrush_opt.is_none_opt() {
            llvm::compiler::optimizations::optimizator::LLVMCompilerOptimizer::new(
                &llvm_module,
                &llvm_context,
            )
            .optimize();
        }

        llvm::compiler::optimizations::passes::LLVMOptimizer::new(
            &llvm_module,
            &target_machine,
            llvm_opt,
            llvm_backend.get_opt_passes(),
            llvm_backend.get_modificator_passes(),
        )
        .optimize();

        if print::llvm_after_optimization(self, &llvm_module, file) {
            return finisher::archive_compilation(self, archive_time, file);
        }

        if emit::llvm_after_optimization(
            self,
            archive_time,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
        )? {
            return finisher::archive_compilation(self, archive_time, file);
        }

        let compiled_file: PathBuf = finisher::llvm_obj_compilation(
            &llvm_module,
            &target_machine,
            build_dir,
            file.get_name(),
        );

        self.add_compiled_unit(compiled_file);

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold(),
                file.get_path().to_string_lossy()
            ),
        );

        Ok(())
    }
}

impl ThrushCompiler<'_> {
    #[inline]
    pub fn get_compiled_files(&self) -> &[PathBuf] {
        &self.compiled
    }

    #[inline]
    pub fn get_options(&self) -> &CompilerOptions {
        self.options
    }
}

impl ThrushCompiler<'_> {
    pub fn add_compiled_unit(&mut self, path: PathBuf) {
        self.compiled.push(path);
    }
}
