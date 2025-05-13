use super::misc::{CompilerOptions, Emitable, Opt};

use super::{
    constants::TARGET_TRIPLES,
    logging::{self, LoggingType},
};

use {
    colored::Colorize,
    inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    std::{
        path::PathBuf,
        process::{self},
    },
};

pub struct CommandLine {
    options: CompilerOptions,
    args: Vec<String>,
    current: usize,
}

impl CommandLine {
    pub fn parse(args: Vec<String>) -> CommandLine {
        let mut command_line: CommandLine = Self {
            options: CompilerOptions::new(),
            args,
            current: 0,
        };

        command_line.build_command_line();

        command_line
    }

    fn build_command_line(&mut self) {
        self.args.remove(0);

        if self.args.is_empty() {
            self.show_help();
        }

        while !self.is_eof() {
            self.analyze(self.args[self.current].clone());
        }

        if !self.options.is_build_dir_setted() {
            self.report_error("Compiler build-dir is not setted. Use '-build-dir \"PATH\"'.");
        }
    }

    fn analyze(&mut self, argument: String) {
        match argument.trim() {
            "help" | "-h" | "--help" => {
                self.advance();

                self.show_help();
            }

            "version" | "-v" | "--version" => {
                self.advance();

                println!("{}", env!("CARGO_PKG_VERSION"));

                process::exit(0);
            }

            "-llvm" => {
                self.advance();

                self.options.set_use_llvm_backend(true);
            }

            "target-triples" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(
                        "Cannot use 'target-triples' without '-llvm' flag previously.",
                    );
                }

                TARGET_TRIPLES
                    .iter()
                    .for_each(|target| println!("{}", target));

                process::exit(0);
            }

            "host-target-triple" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(
                        "Cannot use 'host-target-triple' without '-llvm' flag previously.",
                    );
                }

                println!(
                    "{}",
                    TargetMachine::get_default_triple()
                        .as_str()
                        .to_string_lossy()
                );

                process::exit(0);
            }

            "-build-dir" => {
                self.advance();

                self.options.set_build_dir(self.peek().into());

                self.advance();
            }

            "-opt" | "--optimization" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let opt: Opt = match self.peek() {
                    "O0" => Opt::None,
                    "O1" => Opt::Low,
                    "O2" => Opt::Mid,
                    "size" => Opt::Size,
                    "mcqueen" => Opt::Mcqueen,
                    any => {
                        self.report_error(&format!("Unknown optimization level: '{}'.", any));
                        Opt::default()
                    }
                };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_optimization(opt);

                self.advance();
            }

            "--emit" | "-emit" => {
                self.advance();

                if !self.options.use_llvm() && ["raw-llvm-ir", "llvm-bc"].contains(&self.peek()) {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                match self.peek() {
                    "raw-llvm-ir" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::RawLLVMIR),
                    "llvm-bc" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::LLVMBitcode),
                    "ast" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::AST),
                    "tokens" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::Tokens),

                    any => {
                        self.report_error(&format!("Unknown emit option: '{}'.", any));
                    }
                }

                self.advance();
            }

            "--target" | "-t" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let target_triple_argument: &str = self.peek();

                if TARGET_TRIPLES.contains(&target_triple_argument) {
                    let target_triple: TargetTriple = TargetTriple::create(target_triple_argument);

                    self.options
                        .get_mut_llvm_backend_options()
                        .set_target_triple(target_triple);

                    self.advance();

                    return;
                }

                self.report_error(&format!(
                    "Unknown target-triple: '{}'.",
                    target_triple_argument
                ));
            }

            "--reloc" | "-reloc" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let reloc_mode: RelocMode = match self.peek() {
                    "dynamic-no-pic" => RelocMode::DynamicNoPic,
                    "pic" => RelocMode::PIC,
                    "static" => RelocMode::Static,
                    any => {
                        self.report_error(&format!("Unknown reloc mode: '{}'.", any));
                        RelocMode::default()
                    }
                };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_reloc_mode(reloc_mode);

                self.advance();
            }

            "--code-model" | "-code-model" => {
                self.advance();

                let code_model: CodeModel = match self.peek() {
                    "small" => CodeModel::Small,
                    "medium" => CodeModel::Medium,
                    "large" => CodeModel::Large,
                    "kernel" => CodeModel::Kernel,
                    any => {
                        self.report_error(&format!("Unknown code model: '{}'.", any));
                        CodeModel::default()
                    }
                };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_code_model(code_model);

                self.advance();
            }

            possible_file_path
                if PathBuf::from(possible_file_path).exists()
                    && possible_file_path.ends_with(".th") =>
            {
                self.advance();

                let mut file_path: PathBuf = PathBuf::from(possible_file_path);

                let file_name: String = file_path.file_name().map_or_else(
                    || {
                        logging::log(
                            LoggingType::Panic,
                            &format!("Unknown file name '{}'.", file_path.display()),
                        );

                        String::from("unknown.th")
                    },
                    |name| name.to_string_lossy().to_string(),
                );

                if let Ok(canonicalized_path) = file_path.canonicalize() {
                    file_path = canonicalized_path;
                }

                self.options.new_file(file_name, file_path);
            }

            _ => {
                self.advance();

                if self.options.use_llvm() {
                    self.options
                        .get_mut_llvm_backend_options()
                        .add_compiler_argument(argument);

                    return;
                }

                self.report_error(
                    "Expected the arguments after of the declaration of an backend compiler, example '--llvm'.",
                );
            }
        }
    }

    fn show_help(&self) {
        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{}",
                "The Thrush Compiler".custom_color((141, 141, 142)).bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "\n\n{} {} {}\n\n",
                "Usage:".bold(),
                "thrushc".custom_color((141, 141, 142)).bold(),
                "[--flags] [file]".bold()
            ),
        );

        logging::write(logging::OutputIn::Stderr, &"Compiler Commands:\n\n".bold());

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{}] {}\n",
                "•".bold(),
                "help".custom_color((141, 141, 142)).bold(),
                "Show help message.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{}] {}\n",
                "•".bold(),
                "version".custom_color((141, 141, 142)).bold(),
                "Show the version.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{}] {}\n",
                "•".bold(),
                "target-triples".custom_color((141, 141, 142)).bold(),
                "Show the current LLVM target triples supported.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{}] {}\n\n",
                "•".bold(),
                "host-target-triple".custom_color((141, 141, 142)).bold(),
                "Show the host LLVM target-triple.".bold()
            ),
        );

        logging::write(logging::OutputIn::Stderr, &"Compiler flags:\n\n".bold());

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{} | {}] {}\n",
                "•".bold(),
                "-llvm".custom_color((141, 141, 142)).bold(),
                "-llvm".custom_color((141, 141, 142)).bold(),
                "Enable the LLVM backend infrastructure.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{}] {}\n",
                "•".bold(),
                "-build-dir".custom_color((141, 141, 142)).bold(),
                "Set the compiler build directory.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{}] {}\n",
                "•".bold(),
                "--emit | -emit [llvm-ir | llvm-bitcode | asm | ast | tokens]"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Compile the code into specified representation.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{} | {}] {}\n",
                "•".bold(),
                "--optimization [opt-level]"
                    .custom_color((141, 141, 142))
                    .bold(),
                "-opt [opt-level]".custom_color((141, 141, 142)).bold(),
                "Optimization level for the JIT Compiler.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{} | {}] {}\n",
                "•".bold(),
                "--reloc [reloc-mode]".custom_color((141, 141, 142)).bold(),
                "-reloc [reloc-mode]".custom_color((141, 141, 142)).bold(),
                "Indicate how references to memory addresses are handled for the JIT compiler."
                    .bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} [{} | {}] {}\n",
                "•".bold(),
                "--codemodel [model]".custom_color((141, 141, 142)).bold(),
                "-codemd [model]".custom_color((141, 141, 142)).bold(),
                "Define how code is organized and accessed at machine code level for the JIT compiler.".bold()
            ),
        );

        process::exit(1);
    }

    fn advance(&mut self) {
        if self.current >= self.args.len() {
            self.report_error("Expected value after flag or command.");
        }

        self.current += 1;
    }

    fn peek(&self) -> &str {
        if self.is_eof() {
            self.report_error("Expected value after flag or command.");
        }

        &self.args[self.current]
    }

    fn is_eof(&self) -> bool {
        self.current >= self.args.len()
    }

    fn report_error(&self, msg: &str) {
        logging::log(LoggingType::Panic, msg);
    }

    pub fn get_options(&self) -> &CompilerOptions {
        &self.options
    }
}
