use super::misc::{CompilerOptions, Emitable, ThrushOptimization};

use super::utils;
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

            "print-target-triples" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(
                        "Cannot use 'print-target-triples' without '-llvm' flag previously.",
                    );
                }

                TARGET_TRIPLES
                    .iter()
                    .for_each(|target| println!("{}", target));

                process::exit(0);
            }

            "print-host-target-triple" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(
                        "Cannot use 'print-host-target-triple' without '-llvm' flag previously.",
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

            "print-supported-cpus" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(
                        "Cannot use 'print-supported-cpus' without '-llvm' flag previously.",
                    );
                }

                utils::print_supported_cpus();

                process::exit(0);
            }

            "-build-dir" => {
                self.advance();

                self.options.set_build_dir(self.peek().into());

                self.advance();
            }

            "-lkflags" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let linker_flags: String = self.peek().to_string();

                self.options
                    .get_mut_llvm_backend_options()
                    .set_linker_flags(linker_flags);

                self.advance();
            }

            "-cpu" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let target_cpu: String = self.peek().to_string();

                if !utils::is_supported_cpu_target(&target_cpu) {
                    self.report_error(&format!(
                        "Unknown CPU target: '{}' ~ See 'print-supported-cpus'.",
                        target_cpu
                    ));
                }

                self.options
                    .get_mut_llvm_backend_options()
                    .set_target_cpu(target_cpu);

                self.advance();
            }

            "-opt" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let opt: ThrushOptimization = match self.peek() {
                    "O0" => ThrushOptimization::None,
                    "O1" => ThrushOptimization::Low,
                    "O2" => ThrushOptimization::Mid,
                    "size" => ThrushOptimization::Size,
                    "mcqueen" => ThrushOptimization::Mcqueen,
                    any => {
                        self.report_error(&format!("Unknown optimization level: '{}'.", any));
                        ThrushOptimization::default()
                    }
                };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_optimization(opt);

                self.advance();
            }

            "-emit" => {
                self.advance();

                if !self.options.use_llvm() && ["raw-llvm-ir", "llvm-bc"].contains(&self.peek()) {
                    self.report_error(&format!(
                        "Cannot use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                match self.peek() {
                    "llvm-bc" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::LLVMBitcode),
                    "llvm-ir" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::LLVMIR),
                    "asm" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::Assembly),
                    "raw-llvm-bc" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::RawLLVMBitcode),
                    "raw-llvm-ir" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::RawLLVMIR),
                    "raw-asm" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::RawAssembly),
                    "obj" => self
                        .options
                        .get_mut_llvm_backend_options()
                        .add_emit_option(Emitable::Object),
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

            "-target-triple" => {
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

            "-reloc" => {
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

            "-code-model" => {
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

            any => {
                self.advance();

                self.report_error(&format!(
                    "'{}' Unrecognized flag or command. Use '-help' for more information.",
                    any
                ));
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
                "{} {} {}\n",
                "•".bold(),
                "help".custom_color((141, 141, 142)).bold(),
                "Show help message.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n\n",
                "•".bold(),
                "version".custom_color((141, 141, 142)).bold(),
                "Show the version.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "print-target-triples".custom_color((141, 141, 142)).bold(),
                "Show the current LLVM target triples supported.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "print-supported-cpus".custom_color((141, 141, 142)).bold(),
                "Show the current LLVM supported CPUs.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n\n",
                "•".bold(),
                "print-host-target-triple"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Show the host LLVM target-triple.".bold()
            ),
        );

        logging::write(logging::OutputIn::Stderr, &"Compiler flags:\n\n".bold());

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-llvm".custom_color((141, 141, 142)).bold(),
                "Enable the usage of the LLVM backend infrastructure.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-build-dir".custom_color((141, 141, 142)).bold(),
                "Set the compiler build directory.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-target-triple".custom_color((141, 141, 142)).bold(),
                "\"target-triple\"".bold(),
                "Set the LLVM target triple.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-lkflags".custom_color((141, 141, 142)).bold(),
                "\"-lc;-lpthread\"".bold(),
                "Pass flags to the linker.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-emit".custom_color((141, 141, 142)).bold(),
                "llvm-bc|llvm-ir|asm|raw-llvm-ir|raw-llvm-bc|raw-asm|obj|ast|tokens".bold(),
                "Compile the code into specified representation.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-opt".custom_color((141, 141, 142)).bold(),
                "O0|O1|O2|mcqueen".bold(),
                "Optimization level.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &"\nExtra compiler flags:\n\n".bold(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--reloc".custom_color((141, 141, 142)).bold(),
                "[static|pic|dynamic]".bold(),
                "Indicate how references to memory addresses are handled.".bold()
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--codemodel".custom_color((141, 141, 142)).bold(),
                "[small|medium|large|kernel]".bold(),
                "Define how code is organized and accessed at machine code level.".bold()
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
