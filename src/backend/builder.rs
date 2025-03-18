#![allow(clippy::upper_case_acronyms)]

use {
    super::{
        super::{LLVM_BACKEND_COMPILER, Lexer, Parser, Token, logging},
        apis::{debug::DebugAPI, vector::VectorAPI},
        compiler::{
            Compiler,
            misc::{CompilerOptions, ThrushFile},
        },
        instruction::Instruction,
    },
    colored::Colorize,
    inkwell::{
        OptimizationLevel,
        builder::Builder,
        context::Context,
        module::Module,
        targets::{Target, TargetMachine},
    },
    std::{
        fs::{self, write},
        path::{Path, PathBuf},
        process::Command,
        time::{Duration, Instant},
    },
};

pub struct Thrushc<'a> {
    compiled: Vec<PathBuf>,
    files: &'a [ThrushFile],
    options: &'a CompilerOptions,
    llvm_comptime: Duration,
    thrushc_comptime: Duration,
}

pub struct Clang<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

pub struct LLC<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

pub struct LLVMOpt;

struct LLVMDissambler<'a> {
    files: &'a [PathBuf],
}

impl<'a> Thrushc<'a> {
    pub fn new(files: &'a [ThrushFile], options: &'a CompilerOptions) -> Self {
        Self {
            compiled: Vec::new(),
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

        let _ = fs::copy(
            &self.options.output,
            format!("output/{}", self.options.output),
        );

        let _ = fs::remove_file(&self.options.output);

        self.compiled.iter().for_each(|path| {
            let _ = fs::remove_file(path);
        });

        let _ = fs::remove_file("output/vector.o");
        let _ = fs::remove_file("output/debug.o");

        (
            self.thrushc_comptime.as_millis(),
            self.llvm_comptime.as_millis(),
        )
    }

    fn compile_file(&mut self, file: &'a ThrushFile) {
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

        if self.options.emit_thrush_ast {
            let _ = write(
                format!("output/{}.ast", &file.name),
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

        if self.options.include_vector_api {
            VectorAPI::include(&module, &builder, &context);
        } else {
            VectorAPI::define(&module, &builder, &context);
        }

        if self.options.include_debug_api {
            DebugAPI::include(&module, &builder, &context);
        } else {
            DebugAPI::define(&module, &builder, &context);
        }

        Compiler::compile(&module, &builder, &context, instructions);

        self.thrushc_comptime += start_time.elapsed();

        if self.options.emit_raw_llvm_ir {
            module
                .print_to_file(Path::new(&format!("output/{}.ll", &file.name)))
                .unwrap_or_else(|_| {
                    logging::log(
                        logging::LogType::Panic,
                        &format!(
                            "'output/{}.ll' cannot be emitted in this directory.",
                            &file.name
                        ),
                    );
                    unreachable!()
                });

            return;
        }

        let compiled_path: &str = &format!("output/{}.bc", &file.name);

        module.write_bitcode_to_path(Path::new(compiled_path));

        let start_time: Instant = Instant::now();

        LLVMOpt::optimize(
            compiled_path,
            self.options.optimization.to_llvm_17_passes(),
            self.options.optimization.as_llvm_lto_opt(),
        );

        self.llvm_comptime += start_time.elapsed();

        self.compiled.push(PathBuf::from(compiled_path));
    }
}

impl<'a> Clang<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) -> Duration {
        if self.options.emit_llvm_ir {
            LLVMDissambler::new(self.files).dissamble();

            self.files.iter().for_each(|path| {
                let _ = fs::remove_file(path);
            });
        }

        if self.options.emit_asm {
            LLC::new(self.files, self.options).compile();

            self.files.iter().for_each(|path| {
                let _ = fs::remove_file(path);
            });
        }

        if self.options.emit_natives_apart {
            self.emit_natives_apart()
        }

        if self.options.emit_llvm_bitcode || self.options.emit_asm || self.options.emit_llvm_ir {
            return Duration::new(0, 0);
        }

        let mut clang_command: Command =
            Command::new(LLVM_BACKEND_COMPILER.join("clang/bin/clang-17"));

        let start_time: Instant = Instant::now();

        if self.options.executable {
            clang_command.args([
                "-v",
                "-opaque-pointers",
                &format!(
                    "--target={}",
                    self.options
                        .target_triple
                        .as_str()
                        .to_str()
                        .unwrap_or("invalid utf-8")
                ),
                self.options.linking.to_str(),
                self.options.optimization.to_str(true),
            ]);
        } else {
            let library_variant: &str = if self.options.library {
                "-c"
            } else {
                "--emit-static-lib"
            };

            clang_command.args([
                "-v",
                "-opaque-pointers",
                &format!(
                    "--target={}",
                    self.options
                        .target_triple
                        .as_str()
                        .to_str()
                        .unwrap_or("invalid utf-8")
                ),
                self.options.linking.to_str(),
                self.options.optimization.to_str(true),
                library_variant,
            ]);
        }

        clang_command.args(&self.options.args);
        clang_command.args(self.files);
        clang_command.args(["-o", &self.options.output]);

        handle_command(&mut clang_command);

        start_time.elapsed()
    }

    fn emit_natives_apart(&self) {
        let natives: [[PathBuf; 2]; 2] = if self.options.emit_llvm_ir {
            [
                [
                    PathBuf::from("output/vector.ll"),
                    PathBuf::from("natives/llvm-ir/vector.ll"),
                ],
                [
                    PathBuf::from("output/debug.ll"),
                    PathBuf::from("natives/llvm-ir/debug.ll"),
                ],
            ]
        } else {
            [
                [
                    PathBuf::from("output/vector.s"),
                    PathBuf::from("natives/asm/vector.s"),
                ],
                [
                    PathBuf::from("output/debug.s"),
                    PathBuf::from("natives/asm/debug.s"),
                ],
            ]
        };

        natives.iter().for_each(|native| {
            if !native[1].exists() {
                let _ = fs::create_dir_all(native[1].parent().unwrap());
            }

            if native[0].exists() {
                let raw_content: String = fs::read_to_string(&native[0]).unwrap();
                let content: &[u8] = raw_content.as_bytes();

                let _ = write(&native[1], content);
            }
        });
    }
}

impl<'a> LLC<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) {
        let mut llc_command: Command = Command::new(LLVM_BACKEND_COMPILER.join("llvm/llc"));

        llc_command.args([
            self.options.optimization.to_str(true),
            "--asm-verbose",
            "--filetype=asm",
        ]);

        llc_command.args(self.files);

        handle_command(&mut llc_command);
    }
}

impl LLVMOpt {
    pub fn optimize(path: &str, opt: &str, opt_lto: &str) {
        handle_command(
            Command::new(LLVM_BACKEND_COMPILER.join("llvm/opt"))
                .arg(format!("-p={}", opt))
                .arg(path)
                .arg("-o")
                .arg(path),
        );

        handle_command(
            Command::new(LLVM_BACKEND_COMPILER.join("llvm/llvm-lto"))
                .arg(opt_lto)
                .arg(path),
        );
    }
}

impl<'a> LLVMDissambler<'a> {
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }

    pub fn dissamble(&self) {
        handle_command(Command::new(LLVM_BACKEND_COMPILER.join("llvm/llvm-dis")).args(self.files));
    }
}

#[inline]
fn handle_command(command: &mut Command) {
    if let Ok(child) = command.output() {
        if !child.status.success() {
            logging::log(
                logging::LogType::Error,
                &String::from_utf8_lossy(&child.stderr).replace("\n", ""),
            );
        }
    }
}
