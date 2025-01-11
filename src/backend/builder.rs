#![allow(clippy::upper_case_acronyms)]

use {
    super::{
        super::{logging, Lexer, Parser, Token, LLVM_BACKEND_COMPILER},
        compiler::{
            options::{CompilerOptions, ThrushFile},
            Compiler,
        },
        instruction::Instruction,
    },
    inkwell::{
        builder::Builder,
        context::Context,
        module::Module,
        targets::{Target, TargetMachine},
        OptimizationLevel,
    },
    std::{
        fs::{self, write, File},
        io::{self, BufReader, Read},
        path::{Path, PathBuf},
        process::Command,
    },
    stylic::{style, Color, Stylize},
};

pub struct ThrushCompiler<'a> {
    compiled: Vec<PathBuf>,
    files: &'a [ThrushFile],
    options: &'a CompilerOptions,
}

impl<'a> ThrushCompiler<'a> {
    pub fn new(files: &'a [ThrushFile], options: &'a CompilerOptions) -> Self {
        Self {
            compiled: Vec::new(),
            files,
            options,
        }
    }

    pub fn compile(&mut self) {
        self.files.iter().for_each(|file| {
            self.compile_file(file);
        });

        Clang::new(&self.compiled, self.options).compile();

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
    }

    fn compile_file(&mut self, file: &'a ThrushFile) {
        println!(
            "{} {}",
            style("Compiling").bold().fg(Color::Rgb(141, 141, 142)),
            &file.path.to_string_lossy()
        );

        let content: String = fs::read_to_string(&file.path).unwrap();

        let tokens: Vec<Token> = Lexer::lex(content.as_bytes(), file);

        let mut parser: Parser = Parser::new(&tokens, file);
        let instructions: &[Instruction] = parser.start();

        let context: Context = Context::create();
        let builder: Builder<'_> = context.create_builder();
        let module: Module<'_> = context.create_module(&file.name);

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

        Compiler::compile(&module, &builder, &context, self.options, instructions);

        let compiled_path: &str = &format!("output/{}.bc", &file.name);

        module.write_bitcode_to_path(Path::new(compiled_path));

        LLVMOptimizator::optimize(
            compiled_path,
            self.options.optimization.to_llvm_17_passes(),
            self.options.optimization.to_str(true, false),
        );

        self.compiled.push(PathBuf::from(compiled_path));
    }
}

pub struct Clang<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

impl<'a> Clang<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) {
        if self.options.emit_llvm_ir {
            LLVMDissambler::new(self.files).dissamble();

            if self.options.emit_llvm_ir {
                let natives: &[[PathBuf; 2]; 2] = &[
                    [
                        PathBuf::from("output/vector.ll"),
                        PathBuf::from("natives/vector.ll"),
                    ],
                    [
                        PathBuf::from("output/debug.ll"),
                        PathBuf::from("natives/debug.ll"),
                    ],
                ];

                natives.iter().for_each(|native| {
                    if !native[1].exists() {
                        let _ = fs::create_dir_all("natives");
                    }

                    if native[0].exists() {
                        let raw_content: String = fs::read_to_string(&native[0]).unwrap();
                        let content: &[u8] = raw_content.as_bytes();

                        let _ = write(&native[1], content);
                    }
                });
            }

            self.files.iter().for_each(|path| {
                let _ = fs::remove_file(path);
            });

            return;
        }

        if self.options.emit_asm {
            LLC::new(self.files, self.options).compile();

            self.files.iter().for_each(|path| {
                let _ = fs::remove_file(path);
            });
        }

        if self.options.emit_llvm_bitcode {
            return;
        }

        let mut clang_command: Command = Command::new(LLVM_BACKEND_COMPILER.join("clang-17"));

        if self.options.executable {
            clang_command.args([
                "-v",
                "-opaque-pointers",
                self.options.linking.to_str(),
                self.options.optimization.to_str(true, false),
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
                self.options.linking.to_str(),
                self.options.optimization.to_str(true, false),
                library_variant,
            ]);
        }

        clang_command.args(self.files);

        clang_command.args(&self.options.args);

        clang_command.args(["-o", &self.options.output]);

        handle_command(&mut clang_command);
    }
}

pub struct LLC<'a> {
    files: &'a [PathBuf],
    options: &'a CompilerOptions,
}

impl<'a> LLC<'a> {
    pub fn new(files: &'a [PathBuf], options: &'a CompilerOptions) -> Self {
        Self { files, options }
    }

    pub fn compile(&self) {
        let mut llc_command: Command = Command::new(LLVM_BACKEND_COMPILER.join("llc"));

        llc_command.args([
            self.options.optimization.to_str(true, false),
            "--asm-verbose",
            "--filetype=asm",
        ]);

        llc_command.args(self.files);

        handle_command(&mut llc_command);
    }
}

struct LLVMDissambler<'a> {
    files: &'a [PathBuf],
}

impl<'a> LLVMDissambler<'a> {
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self { files }
    }

    pub fn dissamble(&self) {
        handle_command(Command::new(LLVM_BACKEND_COMPILER.join("llvm-dis")).args(self.files));
    }
}

pub struct LLVMOptimizator;

impl LLVMOptimizator {
    pub fn optimize(path: &str, opt: &str, opt_lto: &str) {
        handle_command(
            Command::new(LLVM_BACKEND_COMPILER.join("opt"))
                .arg(format!("-p={}", opt))
                .arg(path)
                .arg("-o")
                .arg(path),
        );

        handle_command(
            Command::new(LLVM_BACKEND_COMPILER.join("llvm-lto"))
                .arg(opt_lto)
                .arg(path),
        );
    }
}

#[inline]
fn handle_command(command: &mut Command) {
    if let Ok(child) = command.output() {
        if !child.status.success() {
            logging::log(
                logging::LogType::ERROR,
                &String::from_utf8_lossy(&child.stderr).replace("\n", ""),
            );
        }
    }
}
