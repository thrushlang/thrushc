use super::misc::{CompilerOptions, Emitable, FlagsPosition, Opt};

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

pub struct Cli {
    options: CompilerOptions,
    args: Vec<String>,
}

impl Cli {
    pub fn parse(args: Vec<String>) -> Cli {
        let mut cli: Cli = Self {
            options: CompilerOptions::new(),
            args,
        };

        cli._parse();

        cli
    }

    fn _parse(&mut self) {
        self.args.remove(0);

        if self.args.is_empty() {
            self.help();
            return;
        }

        let mut position: usize = 0;

        self.options
            .set_flag_position(FlagsPosition::ThrushCompiler);

        while position != self.args.len() {
            self.analyze(self.args[position].clone(), &mut position);
        }
    }

    fn analyze(&mut self, arg: String, position: &mut usize) {
        let argument: &str = arg.trim();

        match argument {
            "help" | "-h" | "--help" => {
                *position += 1;
                self.help();
            }

            "version" | "-v" | "--version" => {
                *position += 1;
                println!("{}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }

            "-llvm" => {
                *position += 1;
                self.options.set_use_llvm_backend(true);
            }

            "-sc" => {
                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                *position += 1;

                self.options
                    .set_flag_position(FlagsPosition::LLVMStaticCompiler);
            }

            "-lk" => {
                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                *position += 1;

                self.options.set_flag_position(FlagsPosition::LLVMLinker);
            }

            "target-triples" => {
                if !self.options.use_llvm() {
                    self.report_error(
                        "Cannot use 'target-triples' without '-llvm' flag previously.",
                    );
                }

                *position += 1;

                TARGET_TRIPLES
                    .iter()
                    .for_each(|target| println!("{}", target));

                process::exit(0);
            }

            "host-triple" => {
                if !self.options.use_llvm() {
                    self.report_error("Cannot use 'host-triple' without '-llvm' flag previously.");
                }

                *position += 1;

                println!(
                    "{}",
                    TargetMachine::get_default_triple()
                        .as_str()
                        .to_string_lossy()
                );

                process::exit(0);
            }

            "-opt" | "--optimization" => {
                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                *position += 1;

                if *position > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                let opt: Opt = match self.args[self.extract_relative_position(*position)].as_str() {
                    "O0" => Opt::None,
                    "O1" => Opt::Low,
                    "O2" => Opt::Mid,
                    "size" => Opt::Size,
                    "mcqueen" => Opt::Mcqueen,
                    any => {
                        self.report_error(&format!("Unknown optimization level '{}'.", any));
                        Opt::default()
                    }
                };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_optimization(opt);

                *position += 1;
            }

            "--emit" | "-emit" => {
                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                *position += 1;

                if *position > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                match self.args[self.extract_relative_position(*position)].as_str() {
                    "llvm-ir" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::LLVMIR),
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
                    "asm" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::Assembly),
                    "tokens" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::Tokens),

                    any => {
                        self.report_error(&format!(
                            "'{}' is invalid target to emit raw compiled code. Maybe '-emit llvm-ir || raw-llvm-ir || llvm-bc || thrush-ast || asm', is the command?",
                            any
                        ));
                    }
                }

                *position += 1;
            }

            "--target" | "-t" => {
                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                *position += 1;

                if *position > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                match self.args[self.extract_relative_position(*position)].as_str() {
                    target if TARGET_TRIPLES.contains(&target) => {
                        self.options
                            .get_mut_llvm_backend_options()
                            .set_target_triple(TargetTriple::create(target));

                        *position += 1;
                    }

                    _ => {
                        self.report_error(&format!(
                            "Invalid target: {}",
                            self.args[self.extract_relative_position(*position)]
                        ));
                    }
                }
            }

            "--reloc" | "-reloc" => {
                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                *position += 1;

                if *position > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                let reloc_mode: RelocMode =
                    match self.args[self.extract_relative_position(*position)].as_str() {
                        "dynamic-no-pic" => RelocMode::DynamicNoPic,
                        "pic" => RelocMode::PIC,
                        "static" => RelocMode::Static,
                        _ => RelocMode::Default,
                    };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_reloc_mode(reloc_mode);

                *position += 1;
            }

            "--code-model" | "-code-model" => {
                *position += 1;

                if *position > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                let code_model: CodeModel =
                    match self.args[self.extract_relative_position(*position)].as_str() {
                        "small" => CodeModel::Small,
                        "medium" => CodeModel::Medium,
                        "large" => CodeModel::Large,
                        "kernel" => CodeModel::Kernel,
                        _ => CodeModel::Default,
                    };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_code_model(code_model);

                *position += 1;
            }

            path if PathBuf::from(path).exists() && path.ends_with(".th") => {
                *position += 1;

                let mut file_path: PathBuf = PathBuf::from(path);

                if let Ok(canonicalized_path) = file_path.canonicalize() {
                    file_path = canonicalized_path;
                }

                self.options.add_file(
                    file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    file_path,
                );
            }

            arg => {
                *position += 1;

                if self.options.use_llvm() && self.options.get_flag_position().llvm_linker() {
                    self.options
                        .get_mut_llvm_backend_options()
                        .add_linker_argument(arg.to_string());

                    return;
                }

                if self.options.use_llvm()
                    && self.options.get_flag_position().llvm_static_compiler()
                {
                    self.options
                        .get_mut_llvm_backend_options()
                        .add_static_compiler_argument(arg.to_string());

                    return;
                }

                self.report_error(
                    "Expected arguments between '-sc' (Static Compiler) or '-lk' (Linker) flag.",
                );
            }
        }
    }

    #[inline]
    fn extract_relative_position(&self, position: usize) -> usize {
        if position == self.args.len() {
            position - 1
        } else {
            position
        }
    }

    #[inline]
    fn report_error(&self, msg: &str) {
        logging::log(LoggingType::Error, msg);
    }

    fn help(&self) {
        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{}",
                "The Thrush Compiler".custom_color((141, 141, 142)).bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "\n\n{} {} {}\n\n",
                "Usage:".bold(),
                "thrushc".custom_color((141, 141, 142)).bold(),
                "[--flags] [file]".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            "Compiler Commands:\n\n".bold().as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n",
                "•".bold(),
                "help".custom_color((141, 141, 142)).bold(),
                "Show help message.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n",
                "•".bold(),
                "version".custom_color((141, 141, 142)).bold(),
                "Show the version.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n",
                "•".bold(),
                "target-triples".custom_color((141, 141, 142)).bold(),
                "Print the list of supported target triples.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n\n",
                "•".bold(),
                "host-triple".custom_color((141, 141, 142)).bold(),
                "Print the target-triple of this machine.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            "Compiler flags:\n\n".bold().as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({} | {}) {}\n",
                "•".bold(),
                "-llvm".custom_color((141, 141, 142)).bold(),
                "-llvm".custom_color((141, 141, 142)).bold(),
                "Enable the LLVM backend infrastructure.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n",
                "•".bold(),
                "-sc".custom_color((141, 141, 142)).bold(),
                "Pass arguments to the Static Compiler.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n",
                "•".bold(),
                "-lk".custom_color((141, 141, 142)).bold(),
                "Pass arguments to the Linker.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({} | {}) {}\n",
                "•".bold(),
                "--optimization [opt-level]"
                    .custom_color((141, 141, 142))
                    .bold(),
                "-opt [opt-level]".custom_color((141, 141, 142)).bold(),
                "Optimization level.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({}) {}\n",
                "•".bold(),
                "--emit | -emit [llvm-ir | llvm-bitcode | asm | ast | tokens]"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Compile the code into specified representation.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({} | {}) {}\n",
                "•".bold(),
                "--reloc [reloc-mode]".custom_color((141, 141, 142)).bold(),
                "-reloc [reloc-mode]".custom_color((141, 141, 142)).bold(),
                "Indicate how references to memory addresses are handled.".bold()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "{} ({} | {}) {}\n",
                "•".bold(),
                "--codemodel [model]".custom_color((141, 141, 142)).bold(),
                "-codemd [model]".custom_color((141, 141, 142)).bold(),
                "Define how code is organized and accessed at machine code level.".bold()
            )
            .as_bytes(),
        );

        process::exit(0);
    }

    pub fn get_options(&self) -> &CompilerOptions {
        &self.options
    }
}
