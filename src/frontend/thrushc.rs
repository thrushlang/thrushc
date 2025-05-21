use std::{
    fs::{File, write},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use colored::Colorize;

use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{FileType, Target, TargetMachine, TargetTriple},
};

use crate::{
    backend::llvm::{self, linkers::lld::LLVMLinker},
    frontend::{
        lexer::{Lexer, token::Token},
        parser::{Parser, ParserContext},
    },
    standard::{
        backends::{LLVMBackend, LLVMExecutableFlavor},
        diagnostic::Diagnostician,
        logging::{self, LoggingType},
        misc::{CompilerFile, CompilerOptions, Emitable, Emited, ThrushOptimization},
    },
    types::frontend::parser::stmts::stmt::ThrushStatement,
};

use super::semantic::SemanticAnalyzer;

pub struct TheThrushCompiler<'thrushc> {
    compiled_files: Vec<PathBuf>,
    files: &'thrushc [CompilerFile],
    options: &'thrushc CompilerOptions,
    llvm_time: Duration,
    thrushc_time: Duration,
}

impl<'thrushc> TheThrushCompiler<'thrushc> {
    pub fn new(files: &'thrushc [CompilerFile], options: &'thrushc CompilerOptions) -> Self {
        Self {
            compiled_files: Vec::with_capacity(files.len()),
            files,
            options,
            llvm_time: Duration::default(),
            thrushc_time: Duration::default(),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        self.files.iter().for_each(|file| {
            self.compile_file(file);
        });

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();

        if llvm_backend.was_emited() || self.compiled_files.is_empty() {
            return (self.thrushc_time.as_millis(), self.llvm_time.as_millis());
        }

        let executable_flavor: LLVMExecutableFlavor = llvm_backend.get_executable_flavor();

        let linker_time: Duration = LLVMLinker::new(
            self.get_compiled_files(),
            llvm_backend.get_linker_flags(),
            executable_flavor.into_llvm_linker_flavor(),
        )
        .link();

        self.llvm_time += linker_time;

        (self.thrushc_time.as_millis(), self.llvm_time.as_millis())
    }

    fn compile_file(&mut self, file: &'thrushc CompilerFile) {
        let thrushc_time: Instant = Instant::now();

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "RUNNING".bright_green().bold(),
                &file.path.to_string_lossy()
            ),
        );

        let source_code: &[u8] = &self.get_source_code(&file.path);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        let tokens: Vec<Token> = match Lexer::lex(source_code, file) {
            Ok(tokens) => tokens,
            Err(error) => {
                logging::log(logging::LoggingType::Panic, &error.display());
                unreachable!()
            }
        };

        if self.emit_after_frontend(llvm_backend, build_dir, file, Emited::Tokens(&tokens)) {
            self.thrushc_time += thrushc_time.elapsed();
            return;
        }

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file);
        let parser_result: (ParserContext, bool) = parser;
        let parser_context: ParserContext = parser_result.0;
        let parser_throwed_errors: bool = parser_result.1;

        let stmts: &[ThrushStatement] = parser_context.get_stmts();

        let semantic_analysis_throwed_errors: bool = SemanticAnalyzer::new(stmts, file).check();

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            logging::write(
                logging::OutputIn::Stderr,
                &format!(
                    "{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FAILED".bright_red().bold(),
                    &file.path.to_string_lossy()
                ),
            );

            return;
        }

        if self.emit_after_frontend(llvm_backend, build_dir, file, Emited::Statements(stmts)) {
            self.thrushc_time += thrushc_time.elapsed();

            logging::write(
                logging::OutputIn::Stdout,
                &format!(
                    "{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FINISHED".bright_green().bold(),
                    &file.path.to_string_lossy()
                ),
            );

            return;
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
                logging::LoggingType::Panic,
                "Cannot generate a target from triple target.",
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
                    logging::LoggingType::Panic,
                    "Cannot generate a target machine from target.",
                );

                unreachable!()
            });

        llvm_module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        llvm::compiler::LLVMCompiler::compile(
            &llvm_module,
            &llvm_builder,
            &llvm_context,
            stmts,
            target_machine.get_target_data(),
            Diagnostician::new(file),
        );

        if self.emit_before_optimization(
            llvm_backend,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
        ) {
            self.thrushc_time += thrushc_time.elapsed();

            logging::write(
                logging::OutputIn::Stdout,
                &format!(
                    "{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FINISHED".bright_green().bold(),
                    &file.path.to_string_lossy()
                ),
            );

            return;
        }

        llvm::compiler::passes::LLVMOptimizer::new(
            &llvm_module,
            &target_machine,
            llvm_opt,
            llvm_backend.get_opt_passes(),
            llvm_backend.get_modificator_passes(),
        )
        .optimize();

        if self.emit_after_optimization(
            llvm_backend,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
        ) {
            self.thrushc_time += thrushc_time.elapsed();

            logging::write(
                logging::OutputIn::Stdout,
                &format!(
                    "{} {} {}\n",
                    "Compilation".custom_color((141, 141, 142)).bold(),
                    "FINISHED".bright_green().bold(),
                    &file.path.to_string_lossy()
                ),
            );

            return;
        }

        let object_file_path: PathBuf = build_dir.join(format!("{}.o", &file.name));

        target_machine
            .write_to_file(&llvm_module, FileType::Object, &object_file_path)
            .unwrap_or_else(|_| {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", object_file_path.display()),
                );

                unreachable!()
            });

        self.add_compiled_file(object_file_path);

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold(),
                &file.path.to_string_lossy()
            ),
        );
    }

    fn emit_after_frontend(
        &self,
        llvm_backend: &LLVMBackend,
        build_dir: &Path,
        file: &CompilerFile,
        emited: Emited<'thrushc>,
    ) -> bool {
        if llvm_backend.contains_emitable(Emitable::Tokens) {
            if let Emited::Tokens(tokens) = emited {
                let _ = write(
                    build_dir.join(format!("{}.tokens", file.name)),
                    format!("{:#?}", tokens),
                );

                return true;
            }
        }

        if llvm_backend.contains_emitable(Emitable::AST) {
            if let Emited::Statements(stmts) = emited {
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
        &self,
        llvm_backend: &LLVMBackend,
        llvm_module: &Module,
        target_machine: &TargetMachine,
        build_dir: &Path,
        file: &CompilerFile,
    ) -> bool {
        if llvm_backend.contains_emitable(Emitable::RawLLVMIR) {
            let llvm_ir_path: PathBuf = build_dir.join(format!("{}.ll", &file.name));

            llvm_module
                .print_to_file(&llvm_ir_path)
                .unwrap_or_else(|_| {
                    logging::log(
                        logging::LoggingType::Panic,
                        &format!("'{}' cannot be emitted.", llvm_ir_path.display()),
                    );
                    unreachable!()
                });

            return true;
        }

        if llvm_backend.contains_emitable(Emitable::RawLLVMBitcode) {
            let llvm_ir_path: PathBuf = build_dir.join(format!("{}.bc", &file.name));

            if !llvm_module.write_bitcode_to_path(&llvm_ir_path) {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", llvm_ir_path.display()),
                );
                unreachable!()
            }

            return true;
        }

        if llvm_backend.contains_emitable(Emitable::RawAssembly) {
            let llvm_ir_path: PathBuf = build_dir.join(format!("{}.s", &file.name));

            if target_machine
                .write_to_file(llvm_module, FileType::Assembly, &llvm_ir_path)
                .is_err()
            {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", llvm_ir_path.display()),
                );

                unreachable!()
            }

            return true;
        }

        false
    }

    fn emit_after_optimization(
        &self,
        llvm_backend: &LLVMBackend,
        llvm_module: &Module,
        target_machine: &TargetMachine,
        build_dir: &Path,
        file: &CompilerFile,
    ) -> bool {
        if llvm_backend.contains_emitable(Emitable::LLVMBitcode) {
            let bitcode_path: PathBuf = build_dir.join(format!("{}.bc", &file.name));

            if !llvm_module.write_bitcode_to_path(&bitcode_path) {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", bitcode_path.display()),
                );

                unreachable!()
            }

            return true;
        }

        if llvm_backend.contains_emitable(Emitable::LLVMIR) {
            let llvm_ir_path: PathBuf = build_dir.join(format!("{}.ll", &file.name));

            llvm_module
                .print_to_file(&llvm_ir_path)
                .unwrap_or_else(|_| {
                    logging::log(
                        logging::LoggingType::Panic,
                        &format!("'{}' cannot be emitted.", llvm_ir_path.display()),
                    );

                    unreachable!()
                });

            return true;
        }

        if llvm_backend.contains_emitable(Emitable::Assembly) {
            let object_file_path: PathBuf = build_dir.join(format!("{}.s", &file.name));

            if target_machine
                .write_to_file(llvm_module, FileType::Assembly, &object_file_path)
                .is_err()
            {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", object_file_path.display()),
                );

                unreachable!()
            }

            return true;
        }

        if llvm_backend.contains_emitable(Emitable::Object) {
            let object_file_path: PathBuf = build_dir.join(format!("{}.o", &file.name));

            if target_machine
                .write_to_file(llvm_module, FileType::Object, &object_file_path)
                .is_err()
            {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", object_file_path.display()),
                );

                unreachable!()
            }

            return true;
        }

        false
    }

    fn get_compiled_files(&self) -> &[PathBuf] {
        &self.compiled_files
    }

    fn add_compiled_file(&mut self, path: PathBuf) {
        self.compiled_files.push(path);
    }

    fn get_source_code(&self, file_path: &Path) -> Vec<u8> {
        match self.read_file_to_string_buffered(file_path) {
            Ok(code) => code,
            _ => {
                logging::log(
                    LoggingType::Panic,
                    &format!("'{}' file can't be read.", file_path.display()),
                );

                unreachable!()
            }
        }
    }

    fn read_file_to_string_buffered(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
        let file: File = File::open(path)?;
        let mut reader: BufReader<File> = BufReader::new(file);

        let mut buffer: Vec<u8> = Vec::with_capacity(100_000);
        reader.read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}
