#![allow(clippy::upper_case_acronyms)]

use inkwell::targets::TargetTriple;

use crate::backend::llvm;
use crate::frontend::parser::ParserContext;
use crate::middle::instruction::Instruction;
use crate::standard::diagnostic::Diagnostician;
use crate::standard::misc::{CompilerFile, CompilerOptions, Emitable, Opt};

use super::super::super::{
    LLVM_BACKEND, Lexer, Parser, Token,
    logging::{self, LoggingType},
};

use {
    colored::Colorize,
    inkwell::{
        OptimizationLevel,
        builder::Builder,
        context::Context,
        module::Module,
        targets::{Target, TargetMachine},
    },
};

use std::{
    fs::{self, write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

pub struct Thrushc<'a> {
    compiled: Vec<PathBuf>,
    files: &'a [CompilerFile],
    options: &'a CompilerOptions,
    llvm_comptime: Duration,
    thrushc_comptime: Duration,
}

pub struct LLVMLinker<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

pub struct LLVMStaticCompiler<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

pub struct LLVMDissambler<'a> {
    files: &'a [PathBuf],
}

pub struct LLVMOptimizer;

impl<'a> Thrushc<'a> {
    pub fn new(files: &'a [CompilerFile], options: &'a CompilerOptions) -> Self {
        Self {
            compiled: Vec::with_capacity(files.len()),
            files,
            options,
            llvm_comptime: Duration::default(),
            thrushc_comptime: Duration::default(),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        self.files.iter().for_each(|file| {
            self.compile_file(file);
        });

        let static_compiler_llvm_time: Duration =
            LLVMStaticCompiler::new(&self.compiled, self.options).compile();

        let llvm_linker_time: Duration = LLVMLinker::new(&self.compiled, self.options).link();

        self.llvm_comptime += static_compiler_llvm_time;
        self.llvm_comptime += llvm_linker_time;

        (
            self.thrushc_comptime.as_millis(),
            self.llvm_comptime.as_millis(),
        )
    }

    fn compile_file(&mut self, file: &'a CompilerFile) {
        let _ = fs::create_dir_all("build/");

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} {}\n",
                "Compiling".custom_color((141, 141, 142)).bold(),
                &file.path.to_string_lossy()
            )
            .as_bytes(),
        );

        let thrushc_time: Instant = Instant::now();

        let code: String = fs::read_to_string(&file.path).unwrap_or_else(|_| {
            logging::log(
                LoggingType::Panic,
                &format!("'{}' is invalid utf-8 file.", &file.path.display()),
            );
            unreachable!()
        });

        let tokens: Vec<Token> = Lexer::lex(code.as_bytes(), file);

        if self
            .options
            .get_llvm_backend_options()
            .contains_emitable(Emitable::Tokens)
        {
            let _ = write(
                format!("build/{}.tokens", &file.name),
                format!("{:#?}", tokens),
            );

            self.thrushc_comptime += thrushc_time.elapsed();

            return;
        }

        let parser_ctx: ParserContext = Parser::parse(&tokens, file);
        let instructions: &[Instruction] = parser_ctx.get_instructions();

        if self
            .options
            .get_llvm_backend_options()
            .contains_emitable(Emitable::AST)
        {
            let _ = write(
                format!("build/{}.ast", &file.name),
                format!("{:#?}", instructions),
            );

            self.thrushc_comptime += thrushc_time.elapsed();

            return;
        }

        let context: Context = Context::create();

        let builder: Builder = context.create_builder();
        let module: Module = context.create_module(&file.name);

        let target_triple: &TargetTriple =
            self.options.get_llvm_backend_options().get_target_triple();

        module.set_triple(target_triple);

        let thrush_opt: Opt = self.options.get_llvm_backend_options().get_optimization();

        let opt: OptimizationLevel = thrush_opt.to_llvm_opt();

        let machine: TargetMachine = Target::from_triple(target_triple)
            .unwrap()
            .create_target_machine(
                target_triple,
                "",
                "",
                opt,
                self.options.get_llvm_backend_options().get_reloc_mode(),
                self.options.get_llvm_backend_options().get_code_model(),
            )
            .unwrap();

        module.set_data_layout(&machine.get_target_data().get_data_layout());

        llvm::compiler::Compiler::compile(
            &module,
            &builder,
            &context,
            instructions,
            machine.get_target_data(),
            Diagnostician::new(file),
        );

        self.thrushc_comptime += thrushc_time.elapsed();

        if self
            .options
            .get_llvm_backend_options()
            .contains_emitable(Emitable::RawLLVMIR)
        {
            module
                .print_to_file(Path::new(&format!("build/{}.ll", &file.name)))
                .unwrap_or_else(|_| {
                    logging::log(
                        logging::LoggingType::Panic,
                        &format!(
                            "'build/{}.ll' cannot be emitted in the 'build/' directory.",
                            &file.name
                        ),
                    );
                    unreachable!()
                });

            return;
        }

        let compiled_path: &str = &format!("build/{}.bc", &file.name);

        module.write_bitcode_to_path(Path::new(compiled_path));

        let optimization_time: Instant = Instant::now();

        LLVMOptimizer::optimize(compiled_path, thrush_opt.to_llvm_17_passes());

        self.llvm_comptime += optimization_time.elapsed();

        if self
            .options
            .get_llvm_backend_options()
            .contains_emitable(Emitable::LLVMIR)
        {
            let dissamble_time: Instant = Instant::now();

            LLVMDissambler::new(&[PathBuf::from(compiled_path)]).dissamble();

            self.llvm_comptime += dissamble_time.elapsed();

            return;
        }

        self.compiled.push(PathBuf::from(compiled_path));
    }
}

impl<'a> LLVMStaticCompiler<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) -> Duration {
        let start_time: Instant = Instant::now();

        let mut llvm_link_command: Command =
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("llc"));

        llvm_link_command.args(
            self.options
                .get_llvm_backend_options()
                .get_static_compiler_arguments(),
        );

        llvm_link_command.args(self.files);

        handle_command(&mut llvm_link_command);

        start_time.elapsed()
    }
}

impl<'a> LLVMLinker<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn link(&self) -> Duration {
        let start_time: Instant = Instant::now();

        let mut llvm_link_command: Command =
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("lld"));

        llvm_link_command.args(
            self.options
                .get_llvm_backend_options()
                .get_linker_arguments(),
        );

        llvm_link_command.args(self.files);

        handle_command(&mut llvm_link_command);

        start_time.elapsed()
    }
}

impl LLVMOptimizer {
    pub fn optimize(path: &str, opt: &str) {
        handle_command(
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("tools/opt"))
                .arg(format!("-p={}", opt))
                .arg(path)
                .arg("-o")
                .arg(path),
        );
    }
}

impl<'a> LLVMDissambler<'a> {
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }

    pub fn dissamble(&self) {
        handle_command(
            Command::new(LLVM_BACKEND.as_ref().unwrap().join("tools/llvm-dis")).args(self.files),
        );
    }
}

#[inline]
fn handle_command(command: &mut Command) {
    if let Ok(child) = command.output() {
        if !child.status.success() {
            logging::log(
                logging::LoggingType::Error,
                &String::from_utf8_lossy(&child.stderr),
            );
        }
    }
}
