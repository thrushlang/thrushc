use super::backends::LLVMModificatorPasses;
use super::misc::{CompilerOptions, Emitable, ThrushOptimization};

use super::logging::{self, LoggingType};
use super::utils;

use {
    colored::Colorize,
    inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    std::{
        path::PathBuf,
        process::{self},
    },
};

#[derive(Debug)]
pub struct CommandLine {
    options: CompilerOptions,
    args: Vec<String>,
    current: usize,
    position: CommandLinePosition,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CommandLinePosition {
    #[default]
    ThrushCompiler,

    ExternalCompiler,
}

impl CommandLine {
    pub fn parse(args: Vec<String>) -> CommandLine {
        let mut command_line: CommandLine = Self {
            options: CompilerOptions::new(),
            args,
            current: 0,
            position: CommandLinePosition::default(),
        };

        command_line.build();

        command_line
    }

    fn build(&mut self) {
        self.args.remove(0);

        if self.args.is_empty() {
            self.show_help();
        }

        while !self.is_eof() {
            self.analyze(self.args[self.current].clone());
        }

        if !self.options.is_build_dir_setted() {
            self.report_error(
                "Compiler build-dir is not setted or not exist. Try again with '-build-dir \"PATH\"'.",
            );
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

            "llvm-print-target-triples" => {
                self.advance();

                utils::print_llvm_supported_targets_triples();

                process::exit(0);
            }

            "llvm-print-host-target-triple" => {
                self.advance();

                println!(
                    "{}",
                    TargetMachine::get_default_triple()
                        .as_str()
                        .to_string_lossy()
                );

                process::exit(0);
            }

            "llvm-print-supported-cpus" => {
                self.advance();

                utils::print_llvm_supported_cpus();

                process::exit(0);
            }

            "llvm-print-executable-flavors" => {
                self.advance();

                utils::print_supported_llvm_executables_flavors();

                process::exit(0);
            }

            "-build-dir" => {
                self.advance();

                self.options.set_build_dir(self.peek().into());

                self.advance();
            }

            "-start" => {
                self.advance();

                self.position = CommandLinePosition::ExternalCompiler;
            }

            "-end" => {
                self.advance();

                self.position = CommandLinePosition::ThrushCompiler;
            }

            "-clang" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_compilers_configuration()
                    .set_use_clang(true);
            }

            "-custom-clang" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let custom_clang: &str = self.peek();
                let custom_clang_path: PathBuf = PathBuf::from(custom_clang);

                if !custom_clang_path.exists() {
                    self.report_error("Path to the indicated external Clang does not exist.");
                }

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_compilers_configuration()
                    .set_custom_clang(custom_clang_path);

                self.advance();
            }

            "-gcc" => {
                self.advance();

                let custom_gcc: &str = self.peek();
                let custom_gcc_path: PathBuf = PathBuf::from(custom_gcc);

                if !custom_gcc_path.exists() {
                    self.report_error("Path to the indicated external GNU Compiler Colllection (gcc) does not exist.");
                }

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_compilers_configuration()
                    .set_custom_gcc(custom_gcc_path);

                self.advance();
            }

            "--llvm-opt-passes" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let extra_opt_passes: String = self.peek().to_string();

                self.options
                    .get_mut_llvm_backend_options()
                    .set_opt_passes(extra_opt_passes);

                self.advance();
            }

            "--llvm-modificator-opt-passes" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let raw_modificator_passes: &str = self.peek();
                let modificator_passes: Vec<LLVMModificatorPasses> =
                    LLVMModificatorPasses::raw_str_into_llvm_modificator_passes(
                        raw_modificator_passes,
                    );

                self.options
                    .get_mut_llvm_backend_options()
                    .set_modificator_passes(modificator_passes);

                self.advance();
            }

            "-cpu" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let target_cpu: String = self.peek().to_string();

                if !utils::is_supported_llvm_cpu_target(&target_cpu) {
                    self.report_error(&format!(
                        "Unknown CPU target: '{}'. See 'print-supported-cpus' command.",
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
                        "Can't use '{}' without '-llvm' flag previously.",
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
                        self.report_error(&format!("Unknown LLVM optimization level: '{}'.", any));
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

                if !self.options.use_llvm()
                    && [
                        "raw-llvm-ir",
                        "raw-llvm-bc",
                        "raw-asm",
                        "obj",
                        "llvm-bc",
                        "llvm-ir",
                        "asm",
                    ]
                    .contains(&self.peek())
                {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
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
                        self.report_error(&format!("Unknown LLVM emit option: '{}'.", any));
                    }
                }

                self.advance();
            }

            "-target" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let raw_target_triple: &str = self.peek();

                if !utils::is_supported_llvm_target_triple(raw_target_triple) {
                    self.report_error(&format!(
                        "Unknown LLVM target triple: '{}'.",
                        raw_target_triple
                    ));
                }

                let target_triple: TargetTriple = TargetTriple::create(raw_target_triple);

                self.options
                    .get_mut_llvm_backend_options()
                    .set_target_triple(target_triple);

                self.advance();
            }

            "-llvm-reloc" => {
                self.advance();

                if !self.options.use_llvm() {
                    self.report_error(&format!(
                        "Can't use '{}' without '-llvm' flag previously.",
                        argument
                    ));
                }

                let reloc_mode: RelocMode = match self.peek() {
                    "dynamic-no-pic" => RelocMode::DynamicNoPic,
                    "pic" => RelocMode::PIC,
                    "static" => RelocMode::Static,
                    any => {
                        self.report_error(&format!("Unknown LLVM reloc mode: '{}'.", any));
                        RelocMode::default()
                    }
                };

                self.options
                    .get_mut_llvm_backend_options()
                    .set_reloc_mode(reloc_mode);

                self.advance();
            }

            "-llvm-code-model" => {
                self.advance();

                let code_model: CodeModel = match self.peek() {
                    "small" => CodeModel::Small,
                    "medium" => CodeModel::Medium,
                    "large" => CodeModel::Large,
                    "kernel" => CodeModel::Kernel,
                    any => {
                        self.report_error(&format!("Unknown LLVM code model: '{}'.", any));
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

                if self.position.at_any_other_compiler() && self.options.use_llvm() {
                    self.options
                        .get_mut_llvm_backend_options()
                        .get_mut_compilers_configuration()
                        .add_compiler_arg(any.to_string());

                    return;
                }

                logging::log(
                    LoggingType::Panic,
                    &format!("Unknown argument: \"{}\".", any),
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
                "[--flags] [file]"
            ),
        );

        logging::write(logging::OutputIn::Stderr, "General Commands:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "help".custom_color((141, 141, 142)).bold(),
                "Show help message.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n\n",
                "•".bold(),
                "version".custom_color((141, 141, 142)).bold(),
                "Show the version.",
            ),
        );

        logging::write(logging::OutputIn::Stderr, "LLVM Commands:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "llvm-print-target-triples"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Show the current LLVM target triples supported."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "llvm-print-supported-cpus"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Show the current LLVM supported CPUs.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "llvm-print-host-target-triple"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Show the host LLVM target-triple.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n\n",
                "•".bold(),
                "llvm-print-executable-flavors"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Show the LLVM executable flavors.",
            ),
        );

        logging::write(logging::OutputIn::Stderr, "General flags:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-build-dir".custom_color((141, 141, 142)).bold(),
                "Set the build directory.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-clang".custom_color((141, 141, 142)).bold(),
                "Enable embedded Clang to link.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-gcc".custom_color((141, 141, 142)).bold(),
                "Enable embedded GNU Compiler Collection (gcc) to link.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-custom-clang".custom_color((141, 141, 142)).bold(),
                "\"/usr/bin/clang\"",
                "Specifies the path for use of an external clang to link.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-custom-gcc".custom_color((141, 141, 142)).bold(),
                "\"/usr/bin/gcc\"",
                "Specifies the path of use of an external gcc to link.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-start".custom_color((141, 141, 142)).bold(),
                "Marks the start of the arguments for the active external or embedded compiler.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-end".custom_color((141, 141, 142)).bold(),
                "Marks the end of the arguments for the active external or embedded compiler.",
            ),
        );

        logging::write(logging::OutputIn::Stderr, "\nCompiler flags:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-llvm".custom_color((141, 141, 142)).bold(),
                "Enable the usage of the LLVM backend infrastructure.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-cpu".custom_color((141, 141, 142)).bold(),
                "\"haswell\"",
                "Specify the CPU to optimize.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-target".custom_color((141, 141, 142)).bold(),
                "\"x86_64-pc-linux-gnu\"",
                "Set the LLVM target triple.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-emit".custom_color((141, 141, 142)).bold(),
                "llvm-bc|llvm-ir|asm|raw-llvm-ir|raw-llvm-bc|raw-asm|obj|ast|tokens",
                "Compile the code into specified representation.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-opt".custom_color((141, 141, 142)).bold(),
                "O0|O1|O2|mcqueen",
                "LLVM optimization level.",
            ),
        );

        logging::write(logging::OutputIn::Stderr, "\nExtra compiler flags:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--llvm-opt-passes".custom_color((141, 141, 142)).bold(),
                "[-p{passname}]",
                "Pass a list of custom optimization passes to the LLVM optimizator.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--llvm-modificator-passes"
                    .custom_color((141, 141, 142))
                    .bold(),
                "[loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions]",
                "Pass a list of custom modificator passes to the LLVM optimizator.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--llvm-reloc".custom_color((141, 141, 142)).bold(),
                "[static|pic|dynamic]",
                "Indicate how references to memory addresses and linkage symbols are handled."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--llvm-codemodel".custom_color((141, 141, 142)).bold(),
                "[small|medium|large|kernel]",
                "Define how code is organized and accessed at machine code level."
            ),
        );

        process::exit(1);
    }

    fn advance(&mut self) {
        if self.is_eof() {
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

impl CommandLinePosition {
    pub fn at_any_other_compiler(&self) -> bool {
        matches!(self, CommandLinePosition::ExternalCompiler)
    }
}
