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
    targets::{FileType, Target, TargetMachine, TargetTriple},
};

use crate::{
    frontend::{
        lexer::{Lexer, Token},
        parser::{Parser, ParserContext},
        warner::Warner,
    },
    middle::types::frontend::parser::stmts::instruction::Instruction,
    standard::{
        backends::{LLVMBackend, LLVMExecutableFlavor},
        diagnostic::Diagnostician,
        logging::{self},
        misc::{CompilerFile, CompilerOptions, Emitable, ThrushOptimization},
    },
};

use super::llvm::{self, linker::lld::LLVMLinker};

pub mod utils;

pub struct Thrushc<'a> {
    thrushc_compiled_files: Vec<PathBuf>,
    files: &'a [CompilerFile],
    options: &'a CompilerOptions,
    llvm_time: Duration,
    thrushc_time: Duration,
}

impl<'a> Thrushc<'a> {
    pub fn new(files: &'a [CompilerFile], options: &'a CompilerOptions) -> Self {
        Self {
            thrushc_compiled_files: Vec::with_capacity(files.len()),
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

        if llvm_backend.was_emited() {
            return (self.thrushc_time.as_millis(), self.llvm_time.as_millis());
        }

        let executable_flavor: LLVMExecutableFlavor = llvm_backend.get_executable_flavor();

        let linker_time: Duration = LLVMLinker::new(
            &self.thrushc_compiled_files,
            llvm_backend.get_linker_flags(),
            executable_flavor.into_llvm_linker_flavor(),
        )
        .link();

        self.llvm_time += linker_time;

        (self.thrushc_time.as_millis(), self.llvm_time.as_millis())
    }

    fn compile_file(&mut self, file: &'a CompilerFile) {
        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Compiling".custom_color((141, 141, 142)).bold(),
                &file.path.to_string_lossy()
            ),
        );

        let thrushc_time: Instant = Instant::now();

        let source_code: &[u8] = &utils::extract_code_from_file(&file.path);

        let tokens: Vec<Token> = Lexer::lex(source_code, file);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        if llvm_backend.contains_emitable(Emitable::Tokens) {
            let _ = write(
                build_dir.join(format!("{}.tokens", file.name)),
                format!("{:#?}", tokens),
            );

            self.thrushc_time += thrushc_time.elapsed();

            return;
        }

        let parser_ctx: ParserContext = Parser::parse(&tokens, file);
        let instructions: &[Instruction] = parser_ctx.get_instructions();

        Warner::new(instructions, file).check();

        if llvm_backend.contains_emitable(Emitable::AST) {
            let _ = write(
                build_dir.join(format!("{}.ast", file.name)),
                format!("{:#?}", instructions),
            );

            self.thrushc_time += thrushc_time.elapsed();

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

        llvm::compiler::Compiler::compile(
            &llvm_module,
            &llvm_builder,
            &llvm_context,
            instructions,
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

        self.thrushc_compiled_files.push(object_file_path);
    }

    pub fn emit_before_optimization(
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
}
