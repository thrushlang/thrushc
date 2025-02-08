use {
    super::{
        backend::compiler::options::{CompilerOptions, Linking, Opt, ThrushFile},
        constants::TARGETS,
    },
    inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    std::{path::PathBuf, process},
    stylic::{style, Color, Stylize},
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

    fn analyze(&mut self, arg: String, index: &mut usize) {
        match arg.trim() {
            "help" | "-h" | "--help" => {
                *index += 1;
                self.help();
            }

            "targets" => {
                *index += 1;
                TARGETS.iter().for_each(|target| println!("{}", target));
                process::exit(0);
            }

            "native-target" => {
                *index += 1;
                println!("{}", TargetMachine::get_default_triple());
                process::exit(0);
            }

            "version" | "-v" | "--version" => {
                *index += 1;
                println!("v{}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }

            "-o" | "--output" => {
                *index += 2;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for \"{}\" flag.", arg));
                }

                self.options.output = self.args[self.extract_relative_index(*index)].to_string();
            }

            "-opt" | "--optimization" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for \"{}\" flag.", arg));
                }

                self.options.optimization =
                    match self.args[self.extract_relative_index(*index)].as_str() {
                        "O0" => Opt::None,
                        "O1" => Opt::Low,
                        "O2" => Opt::Mid,
                        "mcqueen" => Opt::Mcqueen,
                        any => {
                            self.report_error(&format!("Unknown optimization level \"{}\".", any));
                            Opt::default()
                        }
                    };

                *index += 1;
            }

            "--emit" | "-emit" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for \"{}\" flag.", arg));
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    "llvm-ir" => self.options.emit_llvm_ir = true,
                    "llvm-bc" => self.options.emit_llvm_bitcode = true,
                    "asm" => self.options.emit_asm = true,
                    any => {
                        self.report_error(&format!(
                            "\"{}\" is invalid target to emit raw compiled code. Maybe \"(--emit || -emit) llvm-ir || llvm-bc || asm\", is the command?",
                            any
                        ));
                    }
                }

                *index += 1;
            }

            "--emit-natives-apart" | "-emit-natives-apart" => {
                *index += 1;
                self.options.emit_natives_apart = true;
            }

            "--library" | "-lib" => {
                *index += 1;

                if self.options.executable {
                    self.report_error(&format!(
                        "You can't use \"{}\" and \"{}\" together.",
                        "--executable", "--library"
                    ));
                }

                self.options.library = true;
            }

            "--static-library" | "-slib" => {
                *index += 1;

                if self.options.executable || self.options.library {
                    self.report_error(&format!(
                        "You can't use \"{}\" and \"{}\" together.",
                        "--executable || --library", "--static-library"
                    ));
                }

                self.options.static_library = true;
            }

            "--target" | "-t" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for \"{}\" flag.", arg));
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    target if TARGETS.contains(&target) => {
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

            "--static" | "-s" => {
                *index += 1;
                self.options.linking = Linking::Static;
            }

            "--dynamic" | "-d" => {
                *index += 1;
                self.options.linking = Linking::Dynamic;
            }

            "--reloc" | "-reloc" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for \"{}\" flag.", arg));
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
                    self.report_error(&format!("Missing argument for \"{}\" flag.", arg));
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

            "--include" | "-include" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!(
                        "Missing built-in API specification for \"{}\" flag.",
                        arg
                    ));
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    "vector-api" => {
                        self.options.include_vector_api = true;
                        *index += 1;
                    }
                    "debug-api" => {
                        self.options.include_debug_api = true;
                        *index += 1;
                    }
                    _ => {
                        self.report_error(&format!(
                            "Unknown built-in API name: \"{}\".",
                            self.args[self.extract_relative_index(*index)]
                        ));
                    }
                }
            }

            "--executable" | "-executable" => {
                *index += 1;
                self.options.executable = true;
            }

            path if PathBuf::from(path).exists() => {
                *index += 1;

                let mut file: PathBuf = PathBuf::from(path);

                if file.is_dir() {
                    self.report_error(&format!("\"{}\" is a directory", path));
                } else if file.extension().is_none() {
                    self.report_error(&format!("\"{}\" does not have extension.", path));
                } else if file.extension().unwrap() != "th" {
                    self.report_error(&format!("\"{}\" is not a thrush file.", path));
                } else if file.file_name().is_none() {
                    self.report_error(&format!("\"{}\" does not have a name.", path));
                }

                if path.chars().filter(|ch| *ch == '.').count() > 2 && file.canonicalize().is_ok() {
                    file = file.canonicalize().unwrap();
                }

                self.options.files.push(ThrushFile {
                    name: file.file_name().unwrap().to_string_lossy().to_string(),
                    path: file,
                });
            }

            arg => {
                self.options.args.push(arg.to_string());
                *index += 1;
            }
        }
    }

    fn extract_relative_index(&self, index: usize) -> usize {
        if index == self.args.len() {
            return index - 1;
        }

        index
    }

    fn report_error(&self, msg: &str) {
        println!(
            "{} {}",
            style("ERROR").bold().fg(Color::Rgb(255, 51, 51)),
            style(msg).bold()
        );

        process::exit(1);
    }

    fn help(&self) {
        println!(
            "\n{}\n\n{} {} {}\n\n{}\n{} ({} | {} | {}) {}\n{} ({} | {} | {}) {}\n{} ({}) {}\n{} ({} | {}) {}\n{}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{} ({} | {}) {}\n{}\n{} ({} | {}) {}",
            style("The Thrush Compiler")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Usage:").bold(),
            style("thrushc").bold().fg(Color::Rgb(141, 141, 142)),
            style("[--flags] [file]").bold(),
            style("Compiler Commands:\n").bold(),
            style("•").bold(),
            style("help").bold().fg(Color::Rgb(141, 141, 142)),
            style("-h").bold().fg(Color::Rgb(141, 141, 142)),
            style("--help").bold().fg(Color::Rgb(141, 141, 142)),
            style("Show help message.").bold(),
            style("•").bold(),
            style("version").bold().fg(Color::Rgb(141, 141, 142)),
            style("-v").bold().fg(Color::Rgb(141, 141, 142)),
            style("--version").bold().fg(Color::Rgb(141, 141, 142)),
            style("Show the version.").bold(),
            style("•").bold(),
            style("targets").bold().fg(Color::Rgb(141, 141, 142)),
            style("Print the list of supported targets machines.").bold(),
            style("•").bold(),
            style("native-target").bold().fg(Color::Rgb(141, 141, 142)),
            style("-nt").bold().fg(Color::Rgb(141, 141, 142)),
            style("Print the native target of this machine.").bold(),
            style("\nCompiler Flags:\n").bold(),
            style("•").bold(),
            style("--output [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("-o [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("Output file format.").bold(),
            style("•").bold(),
            style("--target [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("-t [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("Target architecture for the executable or object file.").bold(),
            style("•").bold(),
            style("--optimization [opt-level]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-opt [opt-level]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Optimization level for the executable to emit or object file.").bold(),
            style("•").bold(),
            style("--emit").bold().fg(Color::Rgb(141, 141, 142)),
            style("-emit").bold().fg(Color::Rgb(141, 141, 142)),
            style("Compile the code to Assembler or LLVM IR.").bold(),
            style("•").bold(),
            style("--include").bold().fg(Color::Rgb(141, 141, 142)),
            style("-include").bold().fg(Color::Rgb(141, 141, 142)),
            style("Include a native api code in the IR.").bold(),
            style("•").bold(),
            style("--static").bold().fg(Color::Rgb(141, 141, 142)),
            style("-s").bold().fg(Color::Rgb(141, 141, 142)),
            style("Link the executable statically.").bold(),
            style("•").bold(),
            style("--dynamic").bold().fg(Color::Rgb(141, 141, 142)),
            style("-d").bold().fg(Color::Rgb(141, 141, 142)),
            style("Link the executable dynamically.").bold(),
            style("•").bold(),
            style("--executable").bold().fg(Color::Rgb(141, 141, 142)),
            style("-executable").bold().fg(Color::Rgb(141, 141, 142)),
            style("Compile the code into native executable.").bold(),
            style("•").bold(),
            style("--library").bold().fg(Color::Rgb(141, 141, 142)),
            style("-lib").bold().fg(Color::Rgb(141, 141, 142)),
            style("Compile to an object file ('*.o').").bold(),
            style("•").bold(),
            style("--static-library")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-slib").bold().fg(Color::Rgb(141, 141, 142)),
            style("Compile to an static library ('*.a').").bold(),
            style("•").bold(),
            style("--reloc [reloc-mode]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-reloc [reloc-mode]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Indicate how references to memory addresses are handled.").bold(),
            style("•").bold(),
            style("--codemodel [model]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-codemd [model]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Define how code is organized and accessed in the executable or object file.")
                .bold(),
            style("•").bold(),
            style("--args [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("-args [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("Pass more arguments to the backend compiler.").bold(),
            style("\nUseful Flags:\n").bold(),
            style("•").bold(),
            style("--emit-natives-apart")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-emit-natives-apart")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Emit the llvm ir or assembler output of the Natives APIs in another folter called \"natives\".")
                .bold()
        );

        process::exit(1);
    }
}
