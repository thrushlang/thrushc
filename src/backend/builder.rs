#![allow(clippy::upper_case_acronyms)]

use super::compiler::{
    Compiler,
    instruction::Instruction,
    misc::{CompilerFile, CompilerOptions},
};

use super::super::{
    LLVM_BACKEND, Lexer, Parser, Token, common::constants::CURRENT_CLANG_VERSION, logging,
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

pub struct Clang<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

pub struct LLVMOpt;

struct LLVMDissambler<'a> {
    files: &'a [PathBuf],
}

impl<'a> Thrushc<'a> {
    pub fn new(files: &'a [CompilerFile], options: &'a CompilerOptions) -> Self {
        Self {
            compiled: Vec::with_capacity(files.len()),
            files,
            options,
            llvm_comptime: Duration::new(0, 0),
            thrushc_comptime: Duration::new(0, 0),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        self.files.iter().for_each(|file| {
            self.compile_file(file);
        });

        let llvm_time: Duration = Clang::new(&self.compiled, self.options).compile();

        self.llvm_comptime += llvm_time;

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

        let start_time: Instant = Instant::now();

        let content: String = fs::read_to_string(&file.path).unwrap_or_else(|_| {
            logging::log(
                logging::LogType::Panic,
                &format!("`{}` is invalid utf-8 file.", &file.path.display()),
            );
            unreachable!()
        });

        let tokens: Vec<Token> = Lexer::lex(content.as_bytes(), file);

        let mut parser: Parser = Parser::new(&tokens, file);
        let instructions: &[Instruction] = parser.start();

        if self.options.emit_ast {
            let _ = write(
                format!("build/{}.ast", &file.name),
                format!("{:#?}", instructions),
            );

            return;
        }

        let context: Context = Context::create();
        let builder: Builder = context.create_builder();
        let module: Module = context.create_module(&file.name);

        module.set_triple(&self.options.target_triple);

        let opt: OptimizationLevel = self.options.optimization.to_llvm_opt();

        let machine: TargetMachine = Target::from_triple(&self.options.target_triple)
            .unwrap()
            .create_target_machine(
                &self.options.target_triple,
                "",
                "",
                opt,
                self.options.reloc_mode,
                self.options.code_model,
            )
            .unwrap();

        module.set_data_layout(&machine.get_target_data().get_data_layout());

        Compiler::compile(
            &module,
            &builder,
            &context,
            instructions,
            machine.get_target_data(),
        );

        self.thrushc_comptime += start_time.elapsed();

        if self.options.emit_raw_llvm_ir {
            module
                .print_to_file(Path::new(&format!("build/{}.ll", &file.name)))
                .unwrap_or_else(|_| {
                    logging::log(
                        logging::LogType::Panic,
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

        let start_time: Instant = Instant::now();

        LLVMOpt::optimize(compiled_path, self.options.optimization.to_llvm_17_passes());

        self.llvm_comptime += start_time.elapsed();

        self.compiled.push(PathBuf::from(compiled_path));
    }
}

impl<'a> Clang<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) -> Duration {
        let llvm_time: Instant = Instant::now();

        if self.options.emit_llvm_ir {
            LLVMDissambler::new(self.files).dissamble();
        }

        if self.options.emit_asm {
            self.emit_assembler();
        }

        if self.options.emit_llvm_bitcode || self.options.emit_asm || self.options.emit_llvm_ir {
            return llvm_time.elapsed();
        }

        let mut clang_command: Command = Command::new(LLVM_BACKEND.join("clang-17"));

        clang_command.args([
            "-v",
            &format!(
                "--target={}",
                self.options
                    .target_triple
                    .as_str()
                    .to_str()
                    .unwrap_or("invalid utf-8")
            ),
        ]);

        clang_command.args(&self.options.args);
        clang_command.args(self.files);

        let start_time: Instant = Instant::now();

        handle_command(&mut clang_command);

        start_time.elapsed()
    }

    fn emit_assembler(&self) {
        let mut clang_command: Command = Command::new(LLVM_BACKEND.join("clang-17"));

        clang_command.args(&self.options.args);

        clang_command.args([
            "-v",
            "-S",
            &format!(
                "--target={}",
                self.options
                    .target_triple
                    .as_str()
                    .to_str()
                    .unwrap_or("invalid utf-8")
            ),
        ]);

        clang_command.args(&self.options.args);
        clang_command.args(self.files);

        handle_command(&mut clang_command);
    }
}

impl LLVMOpt {
    pub fn optimize(path: &str, opt: &str) {
        handle_command(
            Command::new(LLVM_BACKEND.join("tools/opt"))
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
        handle_command(Command::new(LLVM_BACKEND.join("tools/llvm-dis")).args(self.files));
    }
}

#[inline]
fn handle_command(command: &mut Command) {
    if let Ok(child) = command.output() {
        if !child.status.success() {
            logging::log(
                logging::LogType::Error,
                &String::from_utf8_lossy(&child.stderr)
                    .replace("\n", "")
                    .replace(&format!("clang version {}", CURRENT_CLANG_VERSION), "")
                    .replace("clang-17:", ""),
            );
        }
    }
}
