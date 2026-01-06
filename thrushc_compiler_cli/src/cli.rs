#![allow(clippy::upper_case_acronyms)]

use std::path::Path;
use std::path::PathBuf;

use inkwell::targets::CodeModel;
use inkwell::targets::RelocMode;
use inkwell::targets::TargetMachine;
use inkwell::targets::TargetTriple;

use thrushc_logging::LoggingType;
use thrushc_logging::OutputIn;
use thrushc_options::CompilerOptions;
use thrushc_options::EmitableUnit;
use thrushc_options::PrintableUnit;
use thrushc_options::ThrushOptimization;

use ahash::AHashMap as HashMap;

use thrushc_options::backends::llvm;
use thrushc_options::backends::llvm::Sanitizer;
use thrushc_options::backends::llvm::SanitizerConfiguration;
use thrushc_options::backends::llvm::debug::DwarfVersion;
use thrushc_options::backends::llvm::passes::LLVMModificatorPasses;
use thrushc_options::linkage::LinkingCompilersConfiguration;

use crate::help;

#[derive(Debug)]
pub struct CommandLine {
    options: CompilerOptions,
    args: Vec<String>,
    current: usize,
    position: CommandLinePosition,
    validation_cache: HashMap<String, bool>,
}

#[derive(Debug)]
pub struct ParsedArg {
    key: String,
    value: Option<String>,
}

impl ParsedArg {
    fn new(arg: &str) -> Self {
        if let Some(eq_pos) = arg.find('=') {
            let (key, value) = arg.split_at(eq_pos);

            return Self {
                key: key.to_string(),
                value: Some(value[1..].to_string()),
            };
        }

        if let Some(eq_pos) = arg.find(':') {
            let (key, value) = arg.split_at(eq_pos);

            return Self {
                key: key.to_string(),
                value: Some(value[1..].to_string()),
            };
        }

        Self {
            key: arg.to_string(),
            value: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CommandLinePosition {
    #[default]
    ThrushCompiler,
    External,
}

impl CommandLinePosition {
    #[inline]
    pub fn at_external(&self) -> bool {
        matches!(self, CommandLinePosition::External)
    }
}

impl CommandLine {
    pub fn parse(mut args: Vec<String>) -> CommandLine {
        let processed_args: Vec<String> = Self::preprocess_args(&mut args);

        let mut command_line: CommandLine = Self {
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

        args.iter().for_each(|arg| {
            let parsed: ParsedArg = ParsedArg::new(arg);

            processed.push(parsed.key);

            if let Some(value) = parsed.value {
                processed.push(value);
            }
        });

        processed
    }
}

impl CommandLine {
    fn build(&mut self) {
        if self.args.is_empty() {
            help::show_help();
        }

        while !self.is_eof() {
            let argument: String = self.args[self.current].clone();
            self.analyze(argument);
        }

        self.validate();
    }
}

impl CommandLine {
    fn validate(&mut self) {
        if !self.get_options().get_llvm_backend_options().is_jit() {
            self.get_mut_options()
                .get_mut_linking_compilers_configuration()
                .comprobate_status();
        }
    }
}

impl CommandLine {
    fn analyze(&mut self, argument: String) {
        let arg: &str = argument.as_str();

        match arg {
            "-h" | "--help" => {
                self.advance();

                match self.peek_optional() {
                    Some("opt") => {
                        self.advance();
                        help::show_optimization_help();
                    }
                    Some("emit") => {
                        self.advance();
                        help::show_emission_help();
                    }
                    Some("print") => {
                        self.advance();
                        help::show_printing_help();
                    }
                    Some("code-model") => {
                        self.advance();
                        help::show_code_model_help();
                    }
                    Some("reloc-model") => {
                        self.advance();
                        help::show_reloc_model();
                    }

                    _ => help::show_help(),
                }
            }

            "-v" | "--version" => {
                self.advance();
                thrushc_logging::write(OutputIn::Stdout, thrushc_constants::COMPILER_VERSION);
                std::process::exit(0);
            }

            "-build-dir" => {
                self.advance();

                let build_dir: PathBuf = self.peek().into();

                self.get_mut_options().set_build_dir(build_dir);

                self.advance();
            }

            "-jit" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_jit(true);
            }

            "-jit-libc" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_jit_required(arg);

                let libc: PathBuf = self.peek().into();

                if (libc.to_string_lossy().contains("/") || libc.to_string_lossy().contains("\\"))
                    && (!libc.exists() || !libc.is_file())
                {
                    self.report_error("A indicated C runtime doesn't exist.");
                }

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_jit_config()
                    .set_libc_path(libc);

                self.advance();
            }

            "-jit-link" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_jit_required(arg);

                let library: PathBuf = self.peek().into();

                if (library.to_string_lossy().contains("/")
                    || library.to_string_lossy().contains("\\"))
                    && (!library.exists() || !library.is_file())
                {
                    self.report_error("A indicated dynamic library doesn't exist.");
                }

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_jit_config()
                    .add_library(library);

                self.advance();
            }

            "-jit-entry" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_jit_required(arg);

                let entrypoint: Vec<u8> = self.peek().as_bytes().to_vec();

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_jit_config()
                    .set_entry(entrypoint);

                self.advance();
            }

            "-start" => {
                self.advance();
                self.position = CommandLinePosition::External;
            }

            "-end" => {
                self.advance();
                self.position = CommandLinePosition::ThrushCompiler;
            }

            "-clang-link" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_not_gcc_active();

                let path: PathBuf = self.peek().into();

                if !self.validate_compiler_path(&path) {
                    self.report_error("Indicated external C & C++ compiler Clang doesn't exist.");
                }

                let compiler_config: &mut LinkingCompilersConfiguration = self
                    .get_mut_options()
                    .get_mut_linking_compilers_configuration();

                compiler_config.set_custom_clang(path);
                compiler_config.set_use_clang(true);

                self.advance();
            }

            "-gcc-link" => {
                self.advance();
                self.validate_not_clang_active();

                let path: PathBuf = self.peek().into();

                if !self.validate_compiler_path(&path) {
                    self.report_error(
                        "Indicated external GNU Compiler Collection (GCC) doesn't exist.",
                    );
                }

                let compiler_config: &mut LinkingCompilersConfiguration = self
                    .get_mut_options()
                    .get_mut_linking_compilers_configuration();

                compiler_config.set_custom_gcc(path);
                compiler_config.set_use_gcc(true);

                self.advance();
            }

            "-target" => {
                self.advance();
                self.validate_llvm_required(arg);

                let target: String = self.peek().to_string();

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target()
                    .set_arch(target);

                self.advance();
            }

            "-target-triple" => {
                self.advance();
                self.validate_llvm_required(arg);

                let raw_target_triple: &str = self.peek();

                let target_triple: TargetTriple = TargetTriple::create(raw_target_triple);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target()
                    .set_target_triple(target_triple);

                self.advance();
            }

            "-cpu" => {
                self.advance();
                self.validate_llvm_required(arg);

                let name: String = self.peek().to_string();

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target_cpu()
                    .set_cpu_name(name);

                self.advance();
            }

            "-cpu-features" => {
                self.advance();
                self.validate_llvm_required(arg);

                let features: String = self.peek().to_string();

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target_cpu()
                    .set_processador_features(features);

                self.advance();
            }

            "-opt" => {
                self.advance();
                self.validate_llvm_required(arg);

                let opt: ThrushOptimization = self.parse_optimization_level(self.peek());

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_optimization(opt);

                self.advance();
            }

            "-emit" => {
                self.advance();
                self.validate_llvm_required(arg);

                let emitable: EmitableUnit = self.parse_emit_option(self.peek());

                self.get_mut_options().add_emit_option(emitable);

                self.advance();
            }

            "-print" => {
                self.advance();
                self.validate_llvm_required(arg);

                let pritable_unit: PrintableUnit = self.parse_print_option(self.peek());

                self.get_mut_options().add_print_option(pritable_unit);

                self.advance();
            }

            "-dbg" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_debug_config()
                    .set_debug_mode();
            }

            "-dbg-for-inlining" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_debug_config()
                    .set_split_debug_inlining();
            }

            "-dbg-for-profiling" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_debug_config()
                    .set_debug_for_profiling();
            }

            "-dbg-dwarf-version" => {
                self.advance();
                self.validate_llvm_required(arg);

                let dwarf_v: DwarfVersion = self.parse_dwarf_version(self.peek());

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_debug_config()
                    .set_dwarf_version(dwarf_v);

                self.advance();
            }

            "--link-check" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_aot_is_enable(arg);
            }

            "--sanitizer" => {
                self.advance();
                self.validate_llvm_required(arg);

                let sanitizer: Sanitizer = self.parse_sanitizer(self.peek());

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_sanitizer(sanitizer);

                self.advance();
            }

            "--no-sanitize" => {
                self.advance();
                self.validate_llvm_required(arg);
                self.validate_sanitizer_required(arg);

                let (nosanitize_bounds, nosanitize_coverage) =
                    self.parse_sanitizer_config(self.peek());

                match self
                    .get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_sanitizer()
                {
                    Sanitizer::Address(config) => {
                        config.set_nosanitize_bounds(nosanitize_bounds);
                        config.set_nosanitize_coverage(nosanitize_coverage);
                    }
                    Sanitizer::Hwaddress(config) => {
                        config.set_nosanitize_bounds(nosanitize_bounds);
                        config.set_nosanitize_coverage(nosanitize_coverage);
                    }
                    Sanitizer::Memory(config) => {
                        config.set_nosanitize_bounds(nosanitize_bounds);
                        config.set_nosanitize_coverage(nosanitize_coverage);
                    }
                    Sanitizer::Memtag(config) => {
                        config.set_nosanitize_bounds(nosanitize_bounds);
                        config.set_nosanitize_coverage(nosanitize_coverage);
                    }
                    Sanitizer::Thread(config) => {
                        config.set_nosanitize_bounds(nosanitize_bounds);
                        config.set_nosanitize_coverage(nosanitize_coverage);
                    }

                    Sanitizer::None => {
                        self.report_error("Cannot modify a sanitizer settings without this option enabled. First, use \"--sanitize.\".");
                    }
                }

                self.advance();
            }

            "--macos-version" => {
                self.advance();
                self.validate_llvm_required(arg);

                let version: String = self.peek().to_string();

                if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    self.report_error("MacOS version must contain only numbers and dots.");
                }

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target()
                    .set_macos_version(version);

                self.advance();
            }

            "--ios-version" => {
                self.advance();
                self.validate_llvm_required(arg);

                let version: String = self.peek().to_string();

                if !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    self.report_error("iOS version must contain only numbers and dots.");
                }

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target()
                    .set_ios_version(version);

                self.advance();
            }

            "--target-triple-darwin-variant" => {
                self.advance();
                self.validate_llvm_required(arg);

                let raw_target_triple: &str = self.peek();

                let target_triple: TargetTriple = TargetTriple::create(raw_target_triple);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .get_mut_target()
                    .set_target_triple_darwin_variant(target_triple);

                self.advance();
            }

            "--omit-frame-pointer" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_omit_frame_pointer();
            }

            "--omit-uwtable" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_omit_uwtable();
            }

            "--omit-direct-access-external-data" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_omit_direct_access_external_data();
            }

            "--omit-rtlib-got" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_omit_rtlibusegot();

                self.advance();
            }

            "--omit-default-opt" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options().set_omit_default_optimizations();
            }

            "--reloc-model" => {
                self.advance();
                self.validate_llvm_required(arg);

                let reloc_mode: RelocMode = self.parse_reloc_mode(self.peek());

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_reloc_mode(reloc_mode);

                self.advance();
            }

            "--code-model" => {
                self.advance();

                let code_model: CodeModel = self.parse_code_model(self.peek());

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_code_model(code_model);

                self.advance();
            }

            "--opt-passes" => {
                self.advance();
                self.validate_llvm_required(arg);

                let extra_opt_passes: String = self.peek().to_string();

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_opt_passes(extra_opt_passes);

                self.advance();
            }

            "--modificator-opt-passes" => {
                self.advance();
                self.validate_llvm_required(arg);

                let raw_modificator_passes: &str = self.peek();
                let modificator_passes: Vec<LLVMModificatorPasses> =
                    LLVMModificatorPasses::into_llvm_modificator_passes(raw_modificator_passes);

                self.get_mut_options()
                    .get_mut_llvm_backend_options()
                    .set_modificator_passes(modificator_passes);

                self.advance();
            }

            "--debug-clang-commands" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_linking_compilers_configuration()
                    .set_debug_clang_commands(true);
            }

            "--debug-gcc-commands" => {
                self.advance();
                self.validate_llvm_required(arg);

                self.get_mut_options()
                    .get_mut_linking_compilers_configuration()
                    .set_debug_gcc_commands(true);
            }

            "--export-compiler-errors" => {
                self.advance();

                self.get_mut_options()
                    .set_export_compiler_error_diagnostics();
            }

            "--export-compiler-warnings" => {
                self.advance();

                self.get_mut_options()
                    .set_export_compiler_warning_diagnostics();
            }

            "--export-diagnostics-path" => {
                self.advance();

                let path: PathBuf = PathBuf::from(self.peek());

                self.get_mut_options().set_export_diagnostic_path(path);

                self.advance();
            }

            "--clean-exported-diagnostics" => {
                self.advance();

                self.get_mut_options()
                    .set_compiler_exported_diagnostics_clean();
            }

            "--clean-build" => {
                self.advance();
                self.get_mut_options().set_clean_build();
            }

            "--clean-tokens" => {
                self.advance();
                self.get_mut_options().set_clean_tokens();
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
                self.get_mut_options().set_clean_llvm_bitcode();
            }

            "--clean-objects" => {
                self.advance();
                self.get_mut_options().set_clean_object();
            }

            "--no-obfuscate-archive-names" => {
                self.advance();
                self.get_mut_options().set_no_obfuscate_archive_names();
            }

            "--no-obfuscate-ir" => {
                self.advance();
                self.get_mut_options().set_no_obfuscate_ir();
            }

            "--enable-ansi-color" => {
                self.advance();
                self.get_mut_options().set_enable_ansi_colors();
            }

            "--print-targets" => {
                self.advance();
                llvm::info::print_all_targets();
            }

            "--print-host-target-triple" => {
                self.advance();

                thrushc_logging::write(
                    OutputIn::Stdout,
                    TargetMachine::get_default_triple()
                        .as_str()
                        .to_string_lossy()
                        .trim(),
                );

                std::process::exit(0);
            }

            "--print-supported-cpus" => {
                self.advance();

                llvm::info::print_specific_cpu_support(
                    self.get_options()
                        .get_llvm_backend_options()
                        .get_target()
                        .get_arch(),
                );
            }

            "--print-opt-passes" => {
                self.advance();
                llvm::info::print_all_available_opt_passes();
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
}

impl CommandLine {
    #[inline]
    fn peek_optional(&self) -> Option<&str> {
        if self.is_eof() {
            return None;
        }

        Some(&self.args[self.current])
    }

    #[inline]
    fn peek(&self) -> &str {
        if self.is_eof() {
            self.report_error("Expected value after flag or command.");
        }

        &self.args[self.current]
    }

    #[inline]
    fn advance(&mut self) {
        if self.is_eof() {
            self.report_error("Expected value after flag or command.");
        }

        self.current += 1;
    }

    #[inline]
    fn report_error(&self, msg: &str) -> ! {
        thrushc_logging::print_critical_error(LoggingType::Error, msg);
    }
}

impl CommandLine {
    fn handle_thrush_file(&mut self, file_path: &str) {
        let mut path: PathBuf = PathBuf::from(file_path);

        let name: String = path.file_name().map_or_else(
            || {
                thrushc_logging::print_critical_error(
                    LoggingType::Error,
                    &format!("Unknown file name '{}'.", path.display()),
                );
            },
            |name| name.to_string_lossy().to_string(),
        );

        let base_name: String = path.file_stem().map_or_else(
            || {
                thrushc_logging::print_critical_error(
                    LoggingType::Error,
                    &format!("Unknown base file name '{}'.", path.display()),
                );
            },
            |name| name.to_string_lossy().to_string(),
        );

        if let Ok(canonicalized_path) = path.canonicalize() {
            path = canonicalized_path;
        }

        let content: String = thrushc_reader::get_file_source_code(&path);

        self.options
            .add_compilation_unit(name, path, content, base_name);
    }

    fn handle_unknown_argument(&mut self, arg: &str) {
        if self.position.at_external() {
            if self.options.get_llvm_backend_options().is_jit() {
                self.options
                    .get_mut_llvm_backend_options()
                    .get_mut_jit_config()
                    .add_argument(arg.to_string());

                return;
            } else {
                self.options
                    .get_mut_linking_compilers_configuration()
                    .add_argument(arg.to_string());

                return;
            }
        }

        thrushc_logging::print_critical_error(
            LoggingType::Error,
            &format!("Unknown argument: \"{}\".", arg),
        );
    }
}

impl CommandLine {
    fn parse_sanitizer_config(&self, spec: &str) -> (bool, bool) {
        let splitted: std::str::Split<'_, &str> = spec.split(";");

        let mut bounds: bool = false;
        let mut coverage: bool = false;

        for config in splitted {
            let (b, c) = match config {
                "bounds" => (true, false),
                "coverage" => (false, true),

                any => {
                    self.report_error(&format!("Unknown sanitizer modificator: '{}'.", any));
                }
            };

            bounds = bounds || b;
            coverage = coverage || c;
        }

        (bounds, coverage)
    }

    #[inline]
    fn parse_sanitizer(&self, sanitizer: &str) -> Sanitizer {
        let config: SanitizerConfiguration = SanitizerConfiguration::new();

        match sanitizer {
            "address" => Sanitizer::Address(config),
            "hwaddress" => Sanitizer::Hwaddress(config),
            "memory" => Sanitizer::Memory(config),
            "thread" => Sanitizer::Thread(config),
            "memtag" => Sanitizer::Memtag(config),

            any => {
                self.report_error(&format!("Unknown sanitizer: '{}'.", any));
            }
        }
    }

    #[inline]
    fn parse_dwarf_version(&self, dwarf: &str) -> DwarfVersion {
        match dwarf.to_lowercase().as_str() {
            "v4" => DwarfVersion::V4,
            "v5" => DwarfVersion::V5,

            any => {
                self.report_error(&format!("Unknown dwarf version: '{}'.", any));
            }
        }
    }

    #[inline]
    fn parse_optimization_level(&self, opt: &str) -> ThrushOptimization {
        match opt {
            "O0" => ThrushOptimization::None,
            "O1" => ThrushOptimization::Low,
            "O2" => ThrushOptimization::Mid,
            "O3" => ThrushOptimization::High,
            "Os" => ThrushOptimization::Size,
            "Oz" => ThrushOptimization::Zize,

            any => {
                self.report_error(&format!("Unknown optimization level: '{}'.", any));
            }
        }
    }

    #[inline]
    fn parse_print_option(&self, emit: &str) -> PrintableUnit {
        match emit {
            "llvm-ir" => PrintableUnit::LLVMIR,
            "unopt-llvm-ir" => PrintableUnit::UnOptLLVMIR,
            "asm" => PrintableUnit::Assembly,
            "unopt-asm" => PrintableUnit::UnOptAssembly,
            "tokens" => PrintableUnit::Tokens,

            any => {
                self.report_error(&format!("Unknown print option: '{}'.", any));
            }
        }
    }

    #[inline]
    fn parse_emit_option(&self, emit: &str) -> EmitableUnit {
        match emit {
            "llvm-bc" => EmitableUnit::LLVMBitcode,
            "llvm-ir" => EmitableUnit::LLVMIR,
            "asm" => EmitableUnit::Assembly,
            "unopt-llvm-bc" => EmitableUnit::UnOptLLVMBitcode,
            "unopt-llvm-ir" => EmitableUnit::UnOptLLVMIR,
            "unopt-asm" => EmitableUnit::UnOptAssembly,
            "obj" => EmitableUnit::Object,
            "ast" => EmitableUnit::AST,
            "tokens" => EmitableUnit::Tokens,

            any => {
                self.report_error(&format!("Unknown emit option: '{}'.", any));
            }
        }
    }

    #[inline]
    fn parse_reloc_mode(&self, reloc: &str) -> RelocMode {
        match reloc {
            "dynamic-no-pic" => RelocMode::DynamicNoPic,
            "pic" => RelocMode::PIC,
            "static" => RelocMode::Static,

            any => {
                self.report_error(&format!("Unknown reloc mode: '{}'.", any));
            }
        }
    }

    #[inline]
    fn parse_code_model(&self, model: &str) -> CodeModel {
        match model {
            "small" => CodeModel::Small,
            "medium" => CodeModel::Medium,
            "large" => CodeModel::Large,
            "kernel" => CodeModel::Kernel,

            any => {
                self.report_error(&format!("Unknown code model: '{}'.", any));
            }
        }
    }
}

impl CommandLine {
    fn validate_llvm_required(&self, arg: &str) {
        if !self.options.uses_llvm() {
            self.report_error(&format!(
                "Can't use '{}' without '-llvm-backend' flag previously.",
                arg
            ));
        }
    }

    fn validate_jit_required(&self, arg: &str) {
        if !self.options.get_llvm_backend_options().is_jit() {
            self.report_error(&format!(
                "Can't use '{}' without '-jit' flag previously.",
                arg
            ));
        }
    }

    fn validate_aot_is_enable(&self, arg: &str) {
        if self.options.get_llvm_backend_options().is_jit() {
            self.report_error(&format!(
                "Can't use '{}' if the '-jit' flag was enabled previously.",
                arg
            ));
        }
    }

    fn validate_not_gcc_active(&self) {
        if self
            .options
            .get_linking_compilers_configuration()
            .get_use_gcc()
        {
            self.report_error("Can't use '-clang-link' flag.");
        }
    }

    fn validate_not_clang_active(&self) {
        if self
            .options
            .get_linking_compilers_configuration()
            .get_use_clang()
        {
            self.report_error("Can't use '-gcc-link' flag.");
        }
    }

    fn validate_sanitizer_required(&self, arg: &str) {
        if !self
            .options
            .get_llvm_backend_options()
            .get_sanitizer()
            .is_none()
        {
            self.report_error(&format!(
                "Can't use '{}' without '--satinizer' flag previously.",
                arg
            ));
        }
    }

    fn validate_compiler_path(&mut self, path: &Path) -> bool {
        let path_str: String = path.to_string_lossy().to_string();

        if let Some(&result) = self.validation_cache.get(&path_str) {
            return result;
        }

        let exists: bool = path.exists() || std::process::Command::new(path).output().is_ok();

        self.validation_cache.insert(path_str, exists);

        exists
    }
}

impl CommandLine {
    fn is_thrush_file(&self, path: &str) -> bool {
        let path: PathBuf = PathBuf::from(path);

        if let Some(extension) = path.extension() {
            if path.exists() && path.is_file() && (extension.eq("thrush") || extension.eq("🐦")) {
                return true;
            }
        }

        false
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.current >= self.args.len()
    }
}

impl CommandLine {
    #[inline]
    pub fn get_options(&self) -> &CompilerOptions {
        &self.options
    }

    #[inline]
    pub fn get_mut_options(&mut self) -> &mut CompilerOptions {
        &mut self.options
    }
}
