use {
    super::{
        backend::compiler::{CompilerOptions, Linking, Opt},
        constants::TARGETS,
        BACKEND_COMPILER, NAME,
    },
    inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    std::{path::Path, process},
    stylic::{style, Color, Stylize},
};

pub struct CLIParser {
    pub compiler_options: CompilerOptions,
    args: Vec<String>,
}

impl CLIParser {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            compiler_options: CompilerOptions::default(),
            args,
        }
    }

    pub fn parse(&mut self) {
        self.args.remove(0);

        if self.args.is_empty() {
            self.help();
            return;
        }

        let mut depth: usize = 0;

        while depth != self.args.len() {
            let arg: String = self.args[depth].clone();
            self.analyze(&arg, &mut depth);
        }
    }

    fn analyze(&mut self, arg: &str, index: &mut usize) {
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
            }

            "native" => {
                *index += 1;

                if *index >= self.args.len() {
                    self.report_error(
                        "Missing native api for thrushc native ---> [native api] <--- [command] command.",
                    );
                }

                let native_api: &str = &self.args[self.extract_relative_index(*index)];

                if native_api != "vector" {
                    self.report_error(&format!(
                        "Unknown native API: {}",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                *index += 1;

                if *index > self.args.len() {
                    self.report_error(
                        "Missing subcommand for thrushc native [native api] ---> [subcommand] <--- command.",
                    );
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    "restore" => {
                        self.compiler_options.restore_natives_apis = true;
                        self.compiler_options.library = true;

                        if native_api == "vector" {
                            self.compiler_options.restore_vector_natives = true;
                        }

                        *index += 1;
                    }

                    _ => {
                        self.report_error(&format!(
                            "Unknown subcommand: {}",
                            self.args[self.extract_relative_index(*index)]
                        ));
                    }
                }
            }

            "--backend" | "-backend" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error("Missing argument for --backend");
                }

                let path: &Path = Path::new(&self.args[self.extract_relative_index(*index)]);

                if !path.exists() {
                    self.report_error(&format!(
                        "The path {} don't exists.",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                if !path.is_dir() {
                    self.report_error(&format!(
                        "The path {} is not a directory.",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                if !path.ends_with("bin") {
                    self.report_error(&format!(
                        "The path {} don't ends with 'bin' folder.",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                if !path.join("clang-18").exists() {
                    self.report_error(&format!(
                        "The path {} don't contains 'clang-18'.",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                if !path.join("opt").exists() {
                    self.report_error(&format!(
                        "The path {} don't contains LLVM Optimizer (opt).",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                if !path.join("lld").exists() {
                    self.report_error(&format!(
                        "The path {} don't contains LLVM Linker (lld).",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                if !path.join("llvm-config").exists() {
                    self.report_error(&format!(
                        "The path {} don't contains LLVM Configurator (llvm-config).",
                        self.args[self.extract_relative_index(*index)]
                    ));
                }

                BACKEND_COMPILER.lock().unwrap().clear();
                BACKEND_COMPILER
                    .lock()
                    .unwrap()
                    .push_str(&self.args[self.extract_relative_index(*index)]);

                *index += 1;
            }

            "-o" | "--output" => {
                *index += 2;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for {}", arg));
                }

                self.compiler_options.output =
                    self.args[self.extract_relative_index(*index)].to_string();
            }

            "-opt" | "--optimization" => {
                *index += 2;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for {}", arg));
                }

                self.compiler_options.optimization =
                    match self.args[self.extract_relative_index(*index)].as_str() {
                        "O0" => Opt::None,
                        "O1" => Opt::Low,
                        "O2" => Opt::Mid,
                        "O3" => Opt::Mcqueen,
                        _ => Opt::default(),
                    };
            }

            "-emit-only-llvm" | "--emit-only-llvm" => {
                *index += 1;
                self.compiler_options.emit_llvm = true;
            }

            "--library" | "-library" => {
                *index += 1;
                self.compiler_options.library = true;
            }

            "-target" | "--target" => {
                *index += 2;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for {}", arg));
                }

                match self.args[self.extract_relative_index(*index)].as_str() {
                    target if TARGETS.contains(&target) => {
                        self.compiler_options.target_triple = TargetTriple::create(target);
                    }

                    _ => {
                        self.report_error(&format!(
                            "Invalid target: {}",
                            self.args[self.extract_relative_index(*index)]
                        ));
                    }
                }
            }

            "--static" | "-static" => {
                *index += 1;

                self.compiler_options.linking = Linking::Static;
            }

            "--dynamic" | "-dynamic" => {
                *index += 1;

                self.compiler_options.linking = Linking::Dynamic;
            }

            "--reloc" | "-reloc" => {
                *index += 2;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for {}", arg));
                }

                self.compiler_options.reloc_mode =
                    match self.args[self.extract_relative_index(*index)].as_str() {
                        "dynamic-no-pic" => RelocMode::DynamicNoPic,
                        "pic" => RelocMode::PIC,
                        "static" => RelocMode::Static,
                        _ => RelocMode::Default,
                    }
            }

            "--code-model" | "-code-model" => {
                *index += 1;

                if *index > self.args.len() {
                    self.report_error(&format!("Missing argument for {}", arg));
                }

                self.compiler_options.code_model = match self.args[*index].as_str() {
                    "small" => CodeModel::Small,
                    "medium" => CodeModel::Medium,
                    "large" => CodeModel::Large,
                    "kernel" => CodeModel::Kernel,
                    _ => CodeModel::Default,
                }
            }

            "--executable" | "-executable" => {
                *index += 1;
                self.compiler_options.executable = true;
            }

            path if Path::new(path).exists() => {
                *index += 1;

                let file: &Path = Path::new(path);

                if file.is_dir() {
                    self.report_error(&format!("\"{}\" is a directory", path));
                }
                /* if path.chars().filter(|ch| *ch == '.').count() > 1 && file.canonicalize().is_ok() {
                    file = file.canonicalize().unwrap();
                } */
                else if file.extension().is_none() {
                    self.report_error(&format!("\"{}\" does not have extension.", path));
                } else if file.extension().unwrap() != "th" {
                    self.report_error(&format!("\"{}\" is not a Thrush Lang file.", path));
                } else if file.file_name().is_none() {
                    self.report_error(&format!("\"{}\" does not have a name.", path));
                }

                if file.file_name().unwrap() == "main.th" {
                    self.compiler_options.is_main = true;
                }

                self.compiler_options.output =
                    file.file_name().unwrap().to_string_lossy().to_string();

                self.compiler_options.file_path = path.to_string();

                NAME.lock().unwrap().clear();
                NAME.lock()
                    .unwrap()
                    .push_str(file.file_name().unwrap().to_str().unwrap());
            }

            _ => {
                self.help();
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
            "\n{}\n",
            style("The Thrush Compiler")
                .bold()
                .fg(Color::Rgb(141, 141, 142))
        );

        println!(
            "{} {} {}\n",
            style("Usage:").bold(),
            style("thrushc").bold().fg(Color::Rgb(141, 141, 142)),
            style("[--flags] [file]").bold()
        );

        println!("{}", style("Available Commands:\n").bold());

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("help").bold().fg(Color::Rgb(141, 141, 142)),
            style("-h").bold().fg(Color::Rgb(141, 141, 142)),
            style("Show help message.").bold()
        );

        println!(
            "{} ({} | {} | {}) {}",
            style("•").bold(),
            style("version").bold().fg(Color::Rgb(141, 141, 142)),
            style("-v").bold().fg(Color::Rgb(141, 141, 142)),
            style("--version").bold().fg(Color::Rgb(141, 141, 142)),
            style("Show the version.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("targets").bold().fg(Color::Rgb(141, 141, 142)),
            style("-t").bold().fg(Color::Rgb(141, 141, 142)),
            style("Print the list of supported targets machines.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("native-target").bold().fg(Color::Rgb(141, 141, 142)),
            style("-nt").bold().fg(Color::Rgb(141, 141, 142)),
            style("Print the native target of this machine.").bold()
        );

        println!(
            "{} ({}) {}",
            style("•").bold(),
            style("native [vector] [restore]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Restore a Core Native API of Thrush Compiler.").bold()
        );

        println!("{}", style("\nCompiler Flags:\n").bold());

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--backend").bold().fg(Color::Rgb(141, 141, 142)),
            style("-backend").bold().fg(Color::Rgb(141, 141, 142)),
            style("Specific the path to the backend compiler to use it (Clang && LLVM).").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--output [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("-output [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("Name of the executable or object file.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--target [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("-t [str]").bold().fg(Color::Rgb(141, 141, 142)),
            style("Target architecture for the executable or object file.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--optimization [opt-level]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-opt [opt-level]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Optimization level for the executable to emit or object file.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--emit-only-llvm")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-emit-only-llvm")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Compile the code only to LLVM IR.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--insert").bold().fg(Color::Rgb(141, 141, 142)),
            style("-insert").bold().fg(Color::Rgb(141, 141, 142)),
            style("Insert an Native API in the LLVM IR to be emited.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--static").bold().fg(Color::Rgb(141, 141, 142)),
            style("-s").bold().fg(Color::Rgb(141, 141, 142)),
            style("Link the executable statically.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--dynamic").bold().fg(Color::Rgb(141, 141, 142)),
            style("-s").bold().fg(Color::Rgb(141, 141, 142)),
            style("Link the executable dynamically.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--executable").bold().fg(Color::Rgb(141, 141, 142)),
            style("-executable").bold().fg(Color::Rgb(141, 141, 142)),
            style("Compile the code into native executable.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--library").bold().fg(Color::Rgb(141, 141, 142)),
            style("-library").bold().fg(Color::Rgb(141, 141, 142)),
            style("Compile to an object file.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--reloc [reloc-mode]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-reloc [reloc-mode]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Indicate how references to memory addresses are handled.").bold()
        );

        println!(
            "{} ({} | {}) {}",
            style("•").bold(),
            style("--codemodel [model]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("-codemd [model]")
                .bold()
                .fg(Color::Rgb(141, 141, 142)),
            style("Define how code is organized and accessed in the executable or object file.")
                .bold()
        );

        process::exit(0);
    }
}
