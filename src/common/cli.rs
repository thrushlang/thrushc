use super::super::LLVM_BACKEND;

use super::misc::{CompilerFile, CompilerOptions, Opt};

use super::{
    constants::TARGET_TRIPLES,
    logging::{self, LoggingType},
};

use {
    colored::Colorize,
    inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    std::{
        path::PathBuf,
        process::{self, Command, Output},
    },
};

pub struct Cli {
    pub options: CompilerOptions,
    args: Vec<String>,
}

impl Cli {
    pub fn parse(args: Vec<String>) -> Cli {
        let mut cli: Cli = Self {
            options: CompilerOptions::default(),
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

        let mut depth: usize = 0;

        while depth != self.args.len() {
            self.analyze(self.args[depth].clone(), &mut depth);
        }
    }

    fn analyze(&mut self, argument: String, index: &mut usize) {
        match argument.trim() {
            "help" | "-h" | "--help" => {
                *index += 1;
                self.help();
            }

            "llvm-help" => {
                *index += 1;
                self.llvm_help();
            }

            "version" | "-v" | "--version" => {
                *index += 1;
                println!("v{}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }

            "target-triples" => {
                *index += 1;
                TARGET_TRIPLES
                    .iter()
                    .for_each(|target| println!("{}", target));
                process::exit(0);
            }

            "host-triple" => {
                *index += 1;
                println!(
                    "{}",
                    TargetMachine::get_default_triple()
                        .as_str()
                        .to_str()
                        .unwrap_or("invalid-utf8")
                );
                process::exit(0);
            }

            "-opt" | "--optimization" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                self.options.optimization =
                    match self.args[self.extract_relative_index(*index)].as_str() {
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

                *index += 1;
            }

            "--emit" | "-emit" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    "llvm-ir" => self.options.emit_llvm_ir = true,
                    "llvm-bc" => self.options.emit_llvm_bitcode = true,
                    "ast" => self.options.emit_ast = true,
                    "asm" => self.options.emit_asm = true,
                    "tokens" => self.options.emit_tokens = true,
                    any => {
                        self.report_error(&format!(
                            "'{}' is invalid target to emit raw compiled code. Maybe '-emit llvm-ir || llvm-bc || thrush-ast || asm', is the command?",
                            any
                        ));
                    }
                }

                *index += 1;
            }

            "--target" | "-t" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    target if TARGET_TRIPLES.contains(&target) => {
                        self.options.target_triple = TargetTriple::create(target);
                        *index += 1;
                    }

                    _ => {
                        self.report_error(&format!(
                            "Invalid target: {}",
                            self.args[self.extract_relative_index(*index)]
                        ));
                    }
                }
            }

            "--emit-raw-llvm-ir" | "-emit-raw-llvm-ir" => {
                *index += 1;
                self.options.emit_raw_llvm_ir = true;
            }

            "--reloc" | "-reloc" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                self.options.reloc_mode =
                    match self.args[self.extract_relative_index(*index)].as_str() {
                        "dynamic-no-pic" => RelocMode::DynamicNoPic,
                        "pic" => RelocMode::PIC,
                        "static" => RelocMode::Static,
                        _ => RelocMode::Default,
                    };

                *index += 1;
            }

            "--code-model" | "-code-model" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for '{}' flag.", argument));
                }

                self.options.code_model =
                    match self.args[self.extract_relative_index(*index)].as_str() {
                        "small" => CodeModel::Small,
                        "medium" => CodeModel::Medium,
                        "large" => CodeModel::Large,
                        "kernel" => CodeModel::Kernel,
                        _ => CodeModel::Default,
                    };

                *index += 1;
            }

            path if PathBuf::from(path).exists() && path.ends_with(".th") => {
                *index += 1;

                let mut file: PathBuf = PathBuf::from(path);

                if path.chars().filter(|ch| *ch == '.').count() > 2 && file.canonicalize().is_ok() {
                    file = file.canonicalize().unwrap_or_default();
                }

                self.options.files.push(CompilerFile {
                    name: file
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: file,
                });
            }

            arg => {
                *index += 1;
                self.options.args.push(arg.to_owned());
            }
        }
    }

    #[inline]
    fn extract_relative_index(&self, index: usize) -> usize {
        if index == self.args.len() {
            index - 1
        } else {
            index
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
                "llvm-help".custom_color((141, 141, 142)).bold(),
                "Show Clang & LLVM help message.".bold()
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
            "Special Compiler flags:\n\n".bold().as_bytes(),
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

        logging::write(
            logging::OutputIn::Stdout,
            format!(
                "\n{}{} {}\n",
                "Note".custom_color((141, 141, 142)).bold().underline(),
                ":".custom_color((141, 141, 142)).bold(),
                "The Thrush Compiler supports Clang & LLVM commands and flags. Execute 'thrushc llvm-help' command to see."
                    .bold(),
            )
            .as_bytes(),
        );

        process::exit(0);
    }

    fn llvm_help(&self) {
        let error = |_| {
            logging::log(
                logging::LoggingType::Panic,
                "Unable to execute 'llvm-help' command.",
            );

            unreachable!()
        };

        let mut clang_help_command: Command = Command::new(LLVM_BACKEND.join("clang-17"));

        clang_help_command.arg("--help");

        let output: Output = clang_help_command.output().unwrap_or_else(error);

        logging::write(logging::OutputIn::Stderr, &output.stdout);

        process::exit(0);
    }
}
