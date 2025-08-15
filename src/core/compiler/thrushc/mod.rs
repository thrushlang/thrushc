mod emit;
mod finisher;
mod interrupt;
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

use crate::{
    backend::{
        linking::compilers::{clang::Clang, gcc::GCC},
        llvm::{self, compiler::context::LLVMCodeGenContext},
    },
    core::{
        compiler::{
            backends::llvm::{LLVMBackend, target::LLVMTarget},
            linking::LinkingCompilersConfiguration,
            options::{CompilerFile, CompilerOptions, Emited, ThrushOptimization},
            reader,
        },
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
    },
    frontend::{
        lexer::{Lexer, token::Token},
        parser::{Parser, ParserContext},
        semantic::SemanticAnalyzer,
        types::ast::Ast,
    },
};

#[derive(Debug)]
pub struct TheThrushCompiler<'thrushc> {
    compiled: Vec<PathBuf>,
    uncompiled: &'thrushc [CompilerFile],

    options: &'thrushc CompilerOptions,
    linking_time: Duration,
    thrushc_time: Duration,
}

impl<'thrushc> TheThrushCompiler<'thrushc> {
    pub fn new(files: &'thrushc [CompilerFile], options: &'thrushc CompilerOptions) -> Self {
        Self {
            compiled: Vec::with_capacity(files.len()),
            uncompiled: files,
            options,
            linking_time: Duration::default(),
            thrushc_time: Duration::default(),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        let mut interrumped: bool = false;

        if self.get_options().get_use_llvm() {
            Target::initialize_all(&InitializationConfig::default());
        } else {
            logging::write(
                logging::OutputIn::Stderr,
                &format!(
                    "{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FAILED".bright_red().bold(),
                    "GCC is not supported yet. Please use LLVM Infrastructure with '-llvm' flag."
                ),
            );
        }

        self.uncompiled.iter().for_each(|file| {
            interrumped = self.compile_with_llvm(file).is_err();
        });

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();

        if interrumped
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

        let linking_compiler_configuration: &LinkingCompilersConfiguration =
            llvm_backend.get_linking_compilers_configuration();

        if linking_compiler_configuration.get_use_clang() {
            match Clang::new(
                self.get_compiled_files(),
                linking_compiler_configuration,
                llvm_backend,
            )
            .link()
            {
                Ok(clang_time) => {
                    self.linking_time += clang_time;

                    logging::write(
                        logging::OutputIn::Stdout,
                        &format!(
                            "{} {}\n",
                            "Linking".custom_color((141, 141, 142)).bold(),
                            "FINISHED".bright_green().bold()
                        ),
                    );
                }
                Err(_) => {
                    logging::write(
                        logging::OutputIn::Stderr,
                        &format!(
                            "\r{} {}\n",
                            "Linking".custom_color((141, 141, 142)).bold(),
                            "FAILED".bright_red().bold()
                        ),
                    );
                }
            }
        } else if linking_compiler_configuration.get_use_gcc() {
            match GCC::new(self.get_compiled_files(), linking_compiler_configuration).link() {
                Ok(gcc_time) => {
                    self.linking_time += gcc_time;

                    logging::write(
                        logging::OutputIn::Stdout,
                        &format!(
                            "{} {}\n",
                            "Linking".custom_color((141, 141, 142)).bold(),
                            "FINISHED".bright_green().bold()
                        ),
                    );
                }
                Err(_) => {
                    logging::write(
                        logging::OutputIn::Stderr,
                        &format!(
                            "\r{} {}\n",
                            "Linking".custom_color((141, 141, 142)).bold(),
                            "FAILED".bright_red().bold()
                        ),
                    );
                }
            }
        } else {
            logging::log(
                LoggingType::Error,
                "No compiler for linking was specified, use -clang or -gcc or see --help.",
            );

            logging::write(
                logging::OutputIn::Stderr,
                &format!(
                    "\r{} {}\n",
                    "Linking".custom_color((141, 141, 142)).bold(),
                    "FAILED".bright_red().bold()
                ),
            );
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }

    fn compile_with_llvm(&mut self, file: &'thrushc CompilerFile) -> Result<(), ()> {
        let archive_time: Instant = Instant::now();

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "RUNNING".bright_green().bold(),
                &file.path.to_string_lossy()
            ),
        );

        let source_code: String = reader::get_file_source_code(&file.path);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        let tokens: Vec<Token> = Lexer::lex(&source_code, file).unwrap_or_else(|error| {
            logging::print_frontend_panic(logging::LoggingType::FrontEndPanic, &error.display())
        });

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
        let llvm_module: Module = llvm_context.create_module(&file.name);

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

        let compiled_file: PathBuf =
            finisher::obj_compilation(&llvm_module, &target_machine, build_dir, &file.name);

        self.add_compiled_file(compiled_file);

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold(),
                &file.path.to_string_lossy()
            ),
        );

        Ok(())
    }
}

impl TheThrushCompiler<'_> {
    pub fn add_compiled_file(&mut self, path: PathBuf) {
        self.compiled.push(path);
    }
}

impl TheThrushCompiler<'_> {
    pub fn get_compiled_files(&self) -> &[PathBuf] {
        &self.compiled
    }

    pub fn get_options(&self) -> &CompilerOptions {
        self.options
    }
}
