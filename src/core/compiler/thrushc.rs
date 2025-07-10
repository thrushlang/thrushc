use std::{
    fs::write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use colored::Colorize;

use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{FileType, InitializationConfig, Target, TargetMachine, TargetTriple},
};

use crate::{
    backend::{
        linking::compilers::{clang::Clang, gcc::GCC},
        llvm::{self, compiler::context::LLVMCodeGenContext},
    },
    core::{
        compiler::{
            backends::llvm::LLVMBackend,
            emitters,
            linking::LinkingCompilersConfiguration,
            options::{CompilerFile, CompilerOptions, Emitable, Emited, ThrushOptimization},
            reader,
        },
        console::logging::{self, LoggingType},
        diagnostic::diagnostician::Diagnostician,
        utils::rand,
    },
    frontend::{
        lexer::{self, Lexer, token::Token},
        parser::{Parser, ParserContext},
        semantic::SemanticAnalyzer,
        types::ast::Ast,
    },
};

pub struct TheThrushCompiler<'thrushc> {
    compiled_files: Vec<PathBuf>,
    files: &'thrushc [CompilerFile],
    options: &'thrushc CompilerOptions,
    linking_time: Duration,
    thrushc_time: Duration,
}

impl<'thrushc> TheThrushCompiler<'thrushc> {
    pub fn new(files: &'thrushc [CompilerFile], options: &'thrushc CompilerOptions) -> Self {
        Self {
            compiled_files: Vec::with_capacity(files.len()),
            files,
            options,
            linking_time: Duration::default(),
            thrushc_time: Duration::default(),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        let mut interrumped: bool = false;

        if self.options.get_use_llvm() {
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

        self.files.iter().for_each(|file| {
            interrumped = self.compile_file_with_llvm(file).is_err();
        });

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();

        if interrumped || llvm_backend.get_was_emited() || self.compiled_files.is_empty() {
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
                llvm_backend.get_target_triple(),
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

    fn compile_file_with_llvm(&mut self, file: &'thrushc CompilerFile) -> Result<(), ()> {
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

        let tokens: Vec<Token> = match Lexer::lex(&source_code, file) {
            Ok(tokens) => tokens,
            Err(error) => {
                logging::log(logging::LoggingType::FrontEndPanic, &error.display());
                unreachable!()
            }
        };

        if self.emit_after_frontend(llvm_backend, build_dir, file, Emited::Tokens(&tokens)) {
            return self.finish_archive_compilation(archive_time, file);
        }

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file);

        let parser_result: (ParserContext, bool) = parser;
        let parser_throwed_errors: bool = parser_result.1;

        let parser_context: ParserContext = parser_result.0;

        let ast: &[Ast] = parser_context.get_ast();

        let semantic_analysis_throwed_errors: bool =
            SemanticAnalyzer::new(ast, file).check(parser_throwed_errors);

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            return self.interrupt_archive_compilation(archive_time, file);
        }

        if self.emit_after_frontend(llvm_backend, build_dir, file, Emited::Ast(ast)) {
            return self.finish_archive_compilation(archive_time, file);
        }

        let llvm_context: Context = Context::create();
        let llvm_builder: Builder = llvm_context.create_builder();
        let llvm_module: Module = llvm_context.create_module(&file.name);

        let target_triple: &TargetTriple = llvm_backend.get_target_triple();
        let target_cpu: &str = llvm_backend.get_target_cpu();
        let thrush_opt: ThrushOptimization = llvm_backend.get_optimization();
        let llvm_opt: OptimizationLevel = thrush_opt.to_llvm_opt();

        llvm_module.set_triple(target_triple);

        let target: Target = Target::from_triple(target_triple).unwrap_or_else(|_| {
            logging::log(
                logging::LoggingType::BackendPanic,
                "Cannot generate a target from LLVM target triple.",
            );

            unreachable!()
        });

        let target_machine: TargetMachine = target
            .create_target_machine(
                target_triple,
                target_cpu,
                "",
                llvm_opt,
                llvm_backend.get_reloc_mode(),
                llvm_backend.get_code_model(),
            )
            .unwrap_or_else(|| {
                logging::log(
                    logging::LoggingType::FrontEndPanic,
                    "Cannot generate a target machine from target.",
                );

                unreachable!()
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

        self.validate_codegen(&llvm_module, file)?;

        if self.emit_before_optimization(
            archive_time,
            llvm_backend,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
        )? {
            return self.finish_archive_compilation(archive_time, file);
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

        if self.emit_after_optimization(
            archive_time,
            llvm_backend,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
        )? {
            return self.finish_archive_compilation(archive_time, file);
        }

        let compiled_file: PathBuf =
            self.finish_obj_compilation(&llvm_module, &target_machine, build_dir, &file.name);

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

    fn interrupt_archive_compilation(
        &mut self,
        archive_time: Instant,
        file: &CompilerFile,
    ) -> Result<(), ()> {
        self.thrushc_time += archive_time.elapsed();

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "FAILED".bright_red().bold(),
                &file.path.to_string_lossy()
            ),
        );

        Err(())
    }

    fn finish_archive_compilation(
        &mut self,
        archive_time: Instant,
        file: &CompilerFile,
    ) -> Result<(), ()> {
        self.thrushc_time += archive_time.elapsed();

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

    fn validate_codegen(&self, llvm_module: &Module, file: &CompilerFile) -> Result<(), ()> {
        if let Err(codegen_error) = llvm_module.verify() {
            logging::log(
                LoggingType::BackendPanic,
                codegen_error.to_string().trim_end(),
            );

            logging::write(
                logging::OutputIn::Stderr,
                &format!(
                    "\r{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FAILED".bright_red().bold(),
                    &file.path.to_string_lossy()
                ),
            );

            return Err(());
        }

        Ok(())
    }

    fn finish_obj_compilation(
        &mut self,
        llvm_module: &Module,
        target_machine: &TargetMachine,
        build_dir: &Path,
        file_name: &str,
    ) -> PathBuf {
        let obj_file_path: PathBuf = build_dir.join(format!(
            "{}_{}.o",
            rand::generate_random_string(),
            file_name
        ));

        target_machine
            .write_to_file(llvm_module, FileType::Object, &obj_file_path)
            .unwrap_or_else(|_| {
                logging::log(
                    logging::LoggingType::FrontEndPanic,
                    &format!("'{}' cannot be emitted.", obj_file_path.display()),
                );

                unreachable!()
            });

        obj_file_path
    }

    fn emit_after_frontend(
        &self,
        llvm_backend: &LLVMBackend,
        build_dir: &Path,
        file: &CompilerFile,
        emited: Emited<'thrushc>,
    ) -> bool {
        emitters::cleaner::auto_clean(self.options);

        if llvm_backend.contains_emitable(Emitable::Tokens) {
            if let Emited::Tokens(tokens) = emited {
                if lexer::printer::print_to_file(tokens, build_dir, &file.name).is_err() {
                    return false;
                }

                return true;
            }
        }

        if llvm_backend.contains_emitable(Emitable::AST) {
            if let Emited::Ast(stmts) = emited {
                let _ = write(
                    build_dir.join(format!("{}.ast", file.name)),
                    format!("{:#?}", stmts),
                );

                return true;
            }
        }

        false
    }

    fn emit_before_optimization(
        &mut self,
        archive_time: Instant,
        llvm_backend: &LLVMBackend,
        llvm_module: &Module,
        target_machine: &TargetMachine,
        build_dir: &Path,
        file: &CompilerFile,
    ) -> Result<bool, ()> {
        emitters::cleaner::auto_clean(self.options);

        if llvm_backend.contains_emitable(Emitable::RawLLVMIR) {
            if let Err(error) =
                emitters::llvmir::emit_llvm_ir(llvm_module, build_dir, &file.name, true)
            {
                logging::log(LoggingType::Error, &error.to_string());
                self.interrupt_archive_compilation(archive_time, file)?;
            }

            return Ok(true);
        }

        if llvm_backend.contains_emitable(Emitable::RawLLVMBitcode) {
            if !emitters::llvmbitcode::emit_llvm_bitcode(llvm_module, build_dir, &file.name, true) {
                logging::log(LoggingType::Error, "Failed to emit LLVM bitcode.");
                self.interrupt_archive_compilation(archive_time, file)?;
            }

            return Ok(true);
        }

        if llvm_backend.contains_emitable(Emitable::RawAssembly) {
            if let Err(error) = emitters::assembler::emit_llvm_assembler(
                llvm_module,
                target_machine,
                build_dir,
                &file.name,
                true,
            ) {
                logging::log(LoggingType::Error, &error.to_string());
                self.interrupt_archive_compilation(archive_time, file)?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    fn emit_after_optimization(
        &mut self,
        archive_time: Instant,
        llvm_backend: &LLVMBackend,
        llvm_module: &Module,
        target_machine: &TargetMachine,
        build_dir: &Path,
        file: &CompilerFile,
    ) -> Result<bool, ()> {
        emitters::cleaner::auto_clean(self.options);

        if llvm_backend.contains_emitable(Emitable::LLVMBitcode) {
            if !emitters::llvmbitcode::emit_llvm_bitcode(llvm_module, build_dir, &file.name, false)
            {
                logging::log(LoggingType::Error, "Failed to emit LLVM bitcode.");
                self.interrupt_archive_compilation(archive_time, file)?;
            }

            return Ok(true);
        }

        if llvm_backend.contains_emitable(Emitable::LLVMIR) {
            if let Err(error) =
                emitters::llvmir::emit_llvm_ir(llvm_module, build_dir, &file.name, false)
            {
                logging::log(LoggingType::Error, &error.to_string());
                self.interrupt_archive_compilation(archive_time, file)?;
            }

            return Ok(true);
        }

        if llvm_backend.contains_emitable(Emitable::Assembly) {
            if let Err(error) = emitters::assembler::emit_llvm_assembler(
                llvm_module,
                target_machine,
                build_dir,
                &file.name,
                false,
            ) {
                logging::log(LoggingType::Error, &error.to_string());
                self.interrupt_archive_compilation(archive_time, file)?;
            };

            return Ok(true);
        }

        if llvm_backend.contains_emitable(Emitable::Object) {
            if let Err(error) = emitters::obj::emit_llvm_object(
                llvm_module,
                target_machine,
                build_dir,
                &file.name,
                false,
            ) {
                logging::log(LoggingType::Error, &error.to_string());
                self.interrupt_archive_compilation(archive_time, file)?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    fn get_compiled_files(&self) -> &[PathBuf] {
        &self.compiled_files
    }

    fn add_compiled_file(&mut self, path: PathBuf) {
        self.compiled_files.push(path);
    }
}
