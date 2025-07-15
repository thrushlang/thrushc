#![allow(clippy::upper_case_acronyms)]

use std::path::Path;
use std::process::Command;

use crate::core::{
    compiler::{
        backends::llvm::LLVMBackend,
        jit::JITConfiguration,
        linking::LinkingCompilersConfiguration,
        options::{CompilerOptions, Emitable, ThrushOptimization},
        passes::LLVMModificatorPasses,
    },
    console::logging::{self, LoggingType},
};

use super::utils;

use {
    colored::Colorize,
    inkwell::targets::{CodeModel, RelocMode, TargetMachine, TargetTriple},
    std::{collections::HashMap, path::PathBuf, process},
};

#[derive(Debug)]
pub struct CLI {
    options: CompilerOptions,
    args: Vec<String>,
    current: usize,
    position: CommandLinePosition,
    validation_cache: HashMap<String, bool>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CommandLinePosition {
    #[default]
    ThrushCompiler,
    ExternalCompiler,
}

#[derive(Debug)]
struct ParsedArg {
    key: String,
    value: Option<String>,
}

impl ParsedArg {
    fn new(arg: &str) -> Self {
        if let Some(eq_pos) = arg.find('=') {
            let (key, value) = arg.split_at(eq_pos);
            Self {
                key: key.to_string(),
                value: Some(value[1..].to_string()),
            }
        } else {
            Self {
                key: arg.to_string(),
                value: None,
            }
        }
    }
}

impl CLI {
    pub fn parse(mut args: Vec<String>) -> CLI {
        let processed_args: Vec<String> = Self::preprocess_args(&mut args);

        let mut command_line: CLI = Self {
            options: CompilerOptions::new(),
            args: processed_args,
            current: 0,
            position: CommandLinePosition::default(),
            validation_cache: HashMap::with_capacity(100),
        };

        command_line.build();
        command_line
    }

    fn preprocess_args(args: &mut Vec<String>) -> Vec<String> {
        let mut processed: Vec<String> = Vec::with_capacity(args.len() * 2);

        if !args.is_empty() {
            args.remove(0);
        }

        for arg in args.iter() {
            let parsed = ParsedArg::new(arg);
            processed.push(parsed.key);

            if let Some(value) = parsed.value {
                processed.push(value);
            }
        }

        processed
    }

    fn build(&mut self) {
        if self.args.is_empty() {
            self.show_help();
        }

        while !self.is_eof() {
            let argument = self.args[self.current].clone();
            self.analyze(argument);
        }

        if !self.options.is_build_dir_setted() {
            self.report_error(
                "Compiler build-dir is not setted or not exist. Try again with '-build-dir \"PATH\"'.",
            );
        }
    }

    fn analyze(&mut self, argument: String) {
        let arg: &str = argument.as_str();

        match arg {
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
                self.validate_llvm_required(arg);
                self.validate_not_gcc_active();

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_linking_compilers_configuration()
                    .set_use_clang(true);
            }

            "-custom-clang" => {
                self.advance();
                self.validate_llvm_required(arg);

                let custom_clang: &str = self.peek();
                let custom_clang_path: PathBuf = PathBuf::from(custom_clang);

                if !self.validate_compiler_path(&custom_clang_path) {
                    self.report_error("Indicated external C compiler Clang doesn't exist.");
                }

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_linking_compilers_configuration()
                    .set_custom_clang(custom_clang_path);

                self.advance();
            }

            "-gcc" => {
                self.advance();
                self.validate_not_clang_active();

                let custom_gcc: &str = self.peek();
                let custom_gcc_path: PathBuf = PathBuf::from(custom_gcc);

                if !self.validate_compiler_path(&custom_gcc_path) {
                    self.report_error(
                        "Indicated external C compiler GNU Compiler Collection (GCC) doesn't exist.",
                    );
                }

                let backend_options: &mut LLVMBackend = self.options.get_mut_llvm_backend_options();
                let compiler_config: &mut LinkingCompilersConfiguration =
                    backend_options.get_mut_linking_compilers_configuration();

                compiler_config.set_custom_gcc(custom_gcc_path);
                compiler_config.set_use_gcc(true);

                self.advance();
            }

            "-jit" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.options
                    .get_mut_llvm_backend_options()
                    .set_jit_config(JITConfiguration::new());
            }

            "-jit-c" => {
                self.advance();

                self.validate_llvm_required(arg);
                self.validate_jit_required(arg);

                let raw_libc_path: &str = self.peek();
                let libc_path: PathBuf = PathBuf::from(raw_libc_path);

                self.validate_jit_path(&libc_path);

                if let Some(jit_config) = self
                    .options
                    .get_mut_llvm_backend_options()
                    .get_mut_jit_config()
                {
                    jit_config.set_libc_path(libc_path);
                } else {
                    self.report_error("Couldn't get llvm jit configuration.");
                }
            }

            "-jit-lib" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_jit_required(arg);

                let raw_lib_path: &str = self.peek();
                let lib_path: PathBuf = PathBuf::from(raw_lib_path);

                self.validate_jit_path(&lib_path);

                if let Some(jit_config) = self
                    .options
                    .get_mut_llvm_backend_options()
                    .get_mut_jit_config()
                {
                    jit_config.add_jit_library(lib_path);
                } else {
                    self.report_error("Couldn't get llvm jit configuration.");
                }
            }

            "-cpu" => {
                self.advance();
                self.validate_llvm_required(arg);

                let target_cpu: String = self.peek().to_string();

                if !self.validate_llvm_cpu(&target_cpu) {
                    self.report_error(&format!(
                        "Unknown CPU target: '{}'. See 'llvm-print-supported-cpus' command.",
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
                self.validate_llvm_required(arg);

                let opt: ThrushOptimization = self.parse_optimization_level(self.peek());

                self.options
                    .get_mut_llvm_backend_options()
                    .set_optimization(opt);

                self.advance();
            }

            "-emit" => {
                self.advance();
                self.validate_emit_llvm_required(arg);

                let emitable: Emitable = self.parse_emit_option(self.peek());

                self.options.add_emit_option(emitable);

                self.advance();
            }

            "-target" => {
                self.advance();
                self.validate_llvm_required(arg);

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

            "--reloc" => {
                self.advance();
                self.validate_llvm_required(arg);

                let reloc_mode: RelocMode = self.parse_reloc_mode(self.peek());

                self.options
                    .get_mut_llvm_backend_options()
                    .set_reloc_mode(reloc_mode);

                self.advance();
            }

            "--code-model" => {
                self.advance();

                let code_model: CodeModel = self.parse_code_model(self.peek());

                self.options
                    .get_mut_llvm_backend_options()
                    .set_code_model(code_model);

                self.advance();
            }

            "--opt-passes" => {
                self.advance();
                self.validate_llvm_required(arg);

                let extra_opt_passes: String = self.peek().to_string();

                self.options
                    .get_mut_llvm_backend_options()
                    .set_opt_passes(extra_opt_passes);

                self.advance();
            }

            "--modificator-opt-passes" => {
                self.advance();
                self.validate_llvm_required(arg);

                let raw_modificator_passes = self.peek();
                let modificator_passes =
                    LLVMModificatorPasses::raw_str_into_llvm_modificator_passes(
                        raw_modificator_passes,
                    );

                self.options
                    .get_mut_llvm_backend_options()
                    .set_modificator_passes(modificator_passes);

                self.advance();
            }

            "--debug-clang-commands" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_linking_compilers_configuration()
                    .set_debug_clang_commands(true);
            }

            "--debug-gcc-commands" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_linking_compilers_configuration()
                    .set_debug_gcc_commands(true);
            }

            "--clean-tokens" => {
                self.advance();
                self.options.set_clean_tokens();
            }

            "--clean-assembler" => {
                self.advance();
                self.options.set_clean_assembler();
            }

            "--clean-llvm-ir" => {
                self.advance();
                self.options.set_clean_llvm_ir();
            }

            "--clean-llvm-bitcode" => {
                self.advance();
                self.options.set_clean_llvm_bitcode();
            }

            "--clean-objects" => {
                self.advance();
                self.options.set_clean_object();
            }

            "--no-obfuscate-archive-names" => {
                self.advance();
                self.options.no_ofuscate_archive_names();
            }

            possible_file_path if self.is_thrush_file(possible_file_path) => {
                self.advance();
                self.handle_thrush_file(possible_file_path);
            }

            any => {
                self.advance();
                self.handle_unknown_argument(any);
            }
        }
    }

    fn validate_llvm_required(&self, arg: &str) {
        if !self.options.get_use_llvm() {
            self.report_error(&format!(
                "Can't use '{}' without '-llvm' flag previously.",
                arg
            ));
        }
    }

    fn validate_jit_required(&self, arg: &str) {
        if self
            .options
            .get_llvm_backend_options()
            .get_jit_config()
            .is_none()
        {
            self.report_error(&format!(
                "Can't use '{}' without '-jit' flag previously.",
                arg
            ));
        }
    }

    fn validate_jit_path(&self, path: &Path) {
        if !path.exists() {
            self.report_error(&format!("The path '{}' does not exist.", path.display()))
        }

        if let Some(extension) = path.extension() {
            if extension != "so" && extension != "dll" {
                self.report_error(&format!(
                    "The path '{}' must end with a file with the .so or .dll extension.",
                    path.display()
                ))
            }
        } else {
            self.report_error(&format!(
                "The path '{}' must contain a path to a file with an extension.",
                path.display()
            ))
        }
    }

    fn validate_emit_llvm_required(&self, arg: &str) {
        if !self.options.get_use_llvm() {
            let llvm_emit_options: [&'static str; 7] = [
                "raw-llvm-ir",
                "raw-llvm-bc",
                "raw-asm",
                "obj",
                "llvm-bc",
                "llvm-ir",
                "asm",
            ];

            if llvm_emit_options.contains(&self.peek()) {
                self.report_error(&format!(
                    "Can't use '{}' without '-llvm' flag previously.",
                    arg
                ));
            }
        }
    }

    fn validate_not_gcc_active(&self) {
        if self
            .options
            .get_llvm_backend_options()
            .get_linking_compilers_configuration()
            .get_use_gcc()
        {
            self.report_error("Can't use '-clang' with -gcc activated.");
        }
    }

    fn validate_not_clang_active(&self) {
        if self
            .options
            .get_llvm_backend_options()
            .get_linking_compilers_configuration()
            .get_use_clang()
        {
            self.report_error("Can't use '-gcc' with -clang activated.");
        }
    }

    fn validate_compiler_path(&mut self, path: &Path) -> bool {
        let path_str: String = path.to_string_lossy().to_string();

        if let Some(&result) = self.validation_cache.get(&path_str) {
            return result;
        }

        let exists: bool = path.exists() || self.probe_as_command(path);

        self.validation_cache.insert(path_str, exists);

        exists
    }

    fn validate_llvm_cpu(&mut self, cpu: &str) -> bool {
        if let Some(&result) = self.validation_cache.get(cpu) {
            return result;
        }

        let is_supported: bool = utils::is_supported_llvm_cpu_target(cpu);

        self.validation_cache.insert(cpu.to_string(), is_supported);

        is_supported
    }

    fn parse_optimization_level(&self, opt_str: &str) -> ThrushOptimization {
        match opt_str {
            "O0" => ThrushOptimization::None,
            "O1" => ThrushOptimization::Low,
            "O2" => ThrushOptimization::Mid,
            "size" => ThrushOptimization::Size,
            "mcqueen" => ThrushOptimization::Mcqueen,
            any => {
                self.report_error(&format!("Unknown LLVM optimization level: '{}'.", any));
            }
        }
    }

    fn parse_emit_option(&self, emit_str: &str) -> Emitable {
        match emit_str {
            "llvm-bc" => Emitable::LLVMBitcode,
            "llvm-ir" => Emitable::LLVMIR,
            "asm" => Emitable::Assembly,
            "raw-llvm-bc" => Emitable::RawLLVMBitcode,
            "raw-llvm-ir" => Emitable::RawLLVMIR,
            "raw-asm" => Emitable::RawAssembly,
            "obj" => Emitable::Object,
            "ast" => Emitable::AST,
            "tokens" => Emitable::Tokens,
            any => {
                self.report_error(&format!("Unknown LLVM emit option: '{}'.", any));
            }
        }
    }

    fn parse_reloc_mode(&self, reloc_str: &str) -> RelocMode {
        match reloc_str {
            "dynamic-no-pic" => RelocMode::DynamicNoPic,
            "pic" => RelocMode::PIC,
            "static" => RelocMode::Static,
            any => {
                self.report_error(&format!("Unknown LLVM reloc mode: '{}'.", any));
            }
        }
    }

    fn parse_code_model(&self, model_str: &str) -> CodeModel {
        match model_str {
            "small" => CodeModel::Small,
            "medium" => CodeModel::Medium,
            "large" => CodeModel::Large,
            "kernel" => CodeModel::Kernel,
            any => {
                self.report_error(&format!("Unknown LLVM code model: '{}'.", any));
            }
        }
    }

    fn is_thrush_file(&self, path: &str) -> bool {
        let path: PathBuf = PathBuf::from(path);

        if let Some(extension) = path.extension() {
            if path.exists() && path.is_file() && (extension.eq("th") || extension.eq("thrush")) {
                return true;
            }
        }

        false
    }

    fn handle_thrush_file(&mut self, file_path: &str) {
        let mut path: PathBuf = PathBuf::from(file_path);

        let file_name: String = path.file_name().map_or_else(
            || {
                logging::log(
                    LoggingType::Panic,
                    &format!("Unknown file name '{}'.", path.display()),
                );

                String::default()
            },
            |name| name.to_string_lossy().to_string(),
        );

        if let Ok(canonicalized_path) = path.canonicalize() {
            path = canonicalized_path;
        }

        self.options.new_file(file_name, path);
    }

    fn handle_unknown_argument(&mut self, arg: &str) {
        if self.position.at_any_other_compiler() && self.options.get_use_llvm() {
            self.options
                .get_mut_llvm_backend_options()
                .get_mut_linking_compilers_configuration()
                .add_compiler_arg(arg.to_string());

            return;
        }

        logging::log(
            LoggingType::Panic,
            &format!("Unknown argument: \"{}\".", arg),
        );
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

    fn probe_as_command(&self, path: &Path) -> bool {
        Command::new(path).output().is_ok()
    }

    fn report_error(&self, msg: &str) -> ! {
        logging::log(LoggingType::Panic, msg);
        unreachable!()
    }

    pub fn get_options(&self) -> &CompilerOptions {
        &self.options
    }

    fn show_help(&self) -> ! {
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
                "[--flags] [files..]"
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

        logging::write(logging::OutputIn::Stderr, "\nGeneral flags:\n\n");

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
                "{} {} [{}] {}\n",
                "•".bold(),
                "-gcc".custom_color((141, 141, 142)).bold(),
                "\"/usr/bin/gcc\"",
                "Speciefies GNU Compiler Collection (GCC) to link.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-custom-clang".custom_color((141, 141, 142)).bold(),
                "\"/usr/bin/clang\"",
                "Specifies the path for use of an external Clang to link.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-start".custom_color((141, 141, 142)).bold(),
                "Marks the start of arguments to the active external or built-in linking compiler.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-end".custom_color((141, 141, 142)).bold(),
                "Marks the end of arguments to the active external or built-in linker compiler.",
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
                "Set the target triple.",
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
                "Optimization level.",
            ),
        );

        /* logging::write(logging::OutputIn::Stderr, "\nJIT Compiler flags:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "-jit".custom_color((141, 141, 142)).bold(),
                "Enables the use of the Just-In-Time Compiler and its settings."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-jit-c".custom_color((141, 141, 142)).bold(),
                "\"/usr/lib/libc.so\"",
                "Specifies the path to the C Standard Library Interface for the JIT to use."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} [{}] {}\n",
                "•".bold(),
                "-jit-lib".custom_color((141, 141, 142)).bold(),
                "\"/usr/lib/my_lib.so\"",
                "Specifies a path to a custom library for use by the JIT."
            ),
        ); */

        logging::write(logging::OutputIn::Stderr, "\nExtra compiler flags:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--opt-passes".custom_color((141, 141, 142)).bold(),
                "[-p{passname}]",
                "Pass a list of custom optimization passes to the LLVM optimizator.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--modificator-passes".custom_color((141, 141, 142)).bold(),
                "[loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions]",
                "Pass a list of custom modificator passes to the LLVM optimizator.",
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--reloc".custom_color((141, 141, 142)).bold(),
                "[static|pic|dynamic]",
                "Indicate how references to memory addresses and linkage symbols are handled."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {} {}\n",
                "•".bold(),
                "--codemodel".custom_color((141, 141, 142)).bold(),
                "[small|medium|large|kernel]",
                "Define how code is organized and accessed at machine code level."
            ),
        );

        logging::write(logging::OutputIn::Stderr, "\nUseful flags:\n\n");

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--debug-clang-command".custom_color((141, 141, 142)).bold(),
                "Displays the generated command for Clang."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--debug-gcc-commands".custom_color((141, 141, 142)).bold(),
                "Displays the generated command for GCC.\n"
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--clean-tokens".custom_color((141, 141, 142)).bold(),
                "Clean the compiler folder that holds the lexical analysis tokens."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--clean-assembler".custom_color((141, 141, 142)).bold(),
                "Clean the compiler folder containing emitted assembler."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--clean-llvm-ir".custom_color((141, 141, 142)).bold(),
                "Clean the compiler folder containing the emitted LLVM IR."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--clean-llvm-bitcode".custom_color((141, 141, 142)).bold(),
                "Clean the compiler folder containing emitted LLVM Bitcode."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n\n",
                "•".bold(),
                "--clean-objects".custom_color((141, 141, 142)).bold(),
                "Clean the compiler folder containing emitted object files."
            ),
        );

        logging::write(
            logging::OutputIn::Stderr,
            &format!(
                "{} {} {}\n",
                "•".bold(),
                "--no-obfuscate-archive-names"
                    .custom_color((141, 141, 142))
                    .bold(),
                "Stop generating name obfuscation for each file; this does not apply to the final build."
            ),
        );

        process::exit(1);
    }
}

impl CommandLinePosition {
    pub fn at_any_other_compiler(&self) -> bool {
        matches!(self, CommandLinePosition::ExternalCompiler)
    }
}
