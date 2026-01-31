mod cleaner;
mod emit;
pub mod emitters;
mod finisher;
mod interrupt;
mod linkage;
mod print;
pub mod printers;
mod starter;
mod utils;
mod validate;

use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::targets::InitializationConfig;
use inkwell::targets::Target;
use inkwell::targets::TargetMachine;
use inkwell::targets::TargetTriple;

use thrushc_ast::Ast;
use thrushc_diagnostician::Diagnostician;
use thrushc_lexer::Lexer;
use thrushc_llvm_callconventions_checker::LLVMCallConventionsChecker;
use thrushc_llvm_codegen::context::LLVMCodeGenContext;
use thrushc_llvm_codegen::jit::LLVMJITCompiler;
use thrushc_llvm_codegen::optimizer::LLVMOptimizationConfig;
use thrushc_llvm_codegen::optimizer::LLVMOptimizerFlags;
use thrushc_llvm_codegen::optimizer::LLVMOptimizerPasses;
use thrushc_llvm_intrinsic_checker::LLVMIntrinsicChecker;
use thrushc_options::CompilationUnit;
use thrushc_options::CompilerOptions;
use thrushc_options::Emited;
use thrushc_options::ThrushOptimization;
use thrushc_options::backends::llvm::LLVMBackend;
use thrushc_options::backends::llvm::jit;
use thrushc_options::backends::llvm::jit::JITConfiguration;
use thrushc_options::backends::llvm::target::LLVMTarget;
use thrushc_options::linkage::LinkingCompilersConfiguration;
use thrushc_parser::Parser;
use thrushc_parser::ParserContext;
use thrushc_preprocessor::Preprocessor;
use thrushc_semantic::SemantiAnalysis;

#[derive(Debug)]
pub struct ThrushCompiler<'thrushc> {
    compiled: Vec<std::path::PathBuf>,
    uncompiled: &'thrushc [CompilationUnit],

    options: &'thrushc CompilerOptions,

    linking_time: std::time::Duration,
    thrushc_time: std::time::Duration,
}

impl<'thrushc> ThrushCompiler<'thrushc> {
    pub fn new(files: &'thrushc [CompilationUnit], options: &'thrushc CompilerOptions) -> Self {
        Self {
            compiled: Vec::with_capacity(files.len()),
            uncompiled: files,

            options,

            linking_time: std::time::Duration::default(),
            thrushc_time: std::time::Duration::default(),
        }
    }
}

impl ThrushCompiler<'_> {
    pub fn compile(&mut self) -> (u128, u128) {
        if self.get_options().uses_llvm() {
            Target::initialize_all(&InitializationConfig::default());

            if self.get_options().get_llvm_backend_options().is_full_jit() {
                return self.compile_jit_llvm();
            } else {
                return self.compile_aot_llvm();
            }
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }
}

impl<'thrushc> ThrushCompiler<'thrushc> {
    fn compile_aot_llvm(&mut self) -> (u128, u128) {
        cleaner::auto_clean(self.get_options());

        let mut interrumped: bool = false;

        self.uncompiled.iter().for_each(|file| {
            interrumped = self.compile_file_with_llvm_aot(file).is_err();
        });

        if interrumped
            || self.get_options().get_was_printed()
            || self.get_options().get_was_emited()
            || self.get_compiled_files().is_empty()
        {
            return (self.thrushc_time.as_millis(), self.linking_time.as_millis());
        }

        starter::linking_phase(self.get_compiled_files());

        let linking_compiler_config: &LinkingCompilersConfiguration =
            self.options.get_linking_compilers_configuration();

        if linking_compiler_config.get_use_clang() {
            linkage::link_with_clang(self);
        } else if linking_compiler_config.get_use_gcc() {
            linkage::link_with_gcc(self);
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }

    fn compile_file_with_llvm_aot(&mut self, file: &'thrushc CompilationUnit) -> Result<(), ()> {
        let file_time: std::time::Instant = std::time::Instant::now();

        starter::archive_compilation_unit(file);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &std::path::PathBuf = self.options.get_build_dir();

        let Ok(tokens) = Lexer::lex(file, self.options) else {
            return interrupt::archive_compilation_unit(self, file, file_time);
        };

        if print::after_frontend(self, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation(self, file_time, file);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation(self, file_time, file);
        }

        let mut preprocessor: Preprocessor = Preprocessor::new();
        let modules: Result<&[thrushc_preprocessor::module::Module<'_>], ()> =
            preprocessor.generate_modules(&tokens, self.options, file);

        if modules.is_err() {
            return interrupt::archive_compilation_unit(self, file, file_time);
        }

        let modules: &[thrushc_preprocessor::module::Module<'_>] = modules.map_err(|_| {
            let _ = interrupt::archive_compilation_unit_with_message(
                self,
                thrushc_logging::LoggingType::Error,
                "Failed to get all modules from the preprocessor. Maybe this is a issue.",
                file,
                file_time,
            );
        })?;

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file, self.options);

        let parser_result: (ParserContext, bool) = parser;
        let parser_throwed_errors: bool = parser_result.1;

        let parser_context: ParserContext = parser_result.0;

        let ast: &[Ast] = parser_context.get_ast();

        let semantic_analysis_throwed_errors: bool =
            SemantiAnalysis::new(ast, file, self.options).analyze(parser_throwed_errors);

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            return finisher::archive_compilation(self, file_time, file);
        }

        let mut intrinsic_checker: LLVMIntrinsicChecker<'_> =
            LLVMIntrinsicChecker::new(ast, file, self.options);

        let mut call_conv_checker: LLVMCallConventionsChecker<'_> =
            LLVMCallConventionsChecker::new(ast, self.get_options(), file);

        let intrinsic_result: bool = intrinsic_checker.analyze();
        let call_conv_result: bool = call_conv_checker.analyze();

        if intrinsic_result || call_conv_result {
            return interrupt::archive_compilation_unit(self, file, file_time);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Ast(ast)) {
            return finisher::archive_compilation(self, file_time, file);
        }

        let llvm_context: Context = Context::create();
        let llvm_builder: Builder = llvm_context.create_builder();
        let llvm_module: Module = llvm_context.create_module(file.get_name());

        let target: &LLVMTarget = llvm_backend.get_target();

        let llvm_triple: &TargetTriple = target.get_triple();

        let llvm_cpu_name: &str = llvm_backend.get_target_cpu().get_cpu_name();
        let llvm_cpu_features: &str = llvm_backend.get_target_cpu().get_cpu_features();

        let compiler_optimization: ThrushOptimization = llvm_backend.get_optimization();
        let llvm_opt: OptimizationLevel = compiler_optimization.to_llvm_opt();

        llvm_module.set_triple(llvm_triple);

        let target: Target = Target::from_triple(llvm_triple).map_err(|_| {
            let _ = interrupt::archive_compilation_unit_with_message(
                self,
                thrushc_logging::LoggingType::Error,
                "The compiler couldn't be configured correctly. The target is possibly unrecognizable. Try again another target or try to fix it.",
                file,
                file_time,
            );
        })?;

        if !target.has_target_machine() {
            interrupt::archive_compilation_unit_with_message(
                self,
                thrushc_logging::LoggingType::Error,
                "The compiler couldn't be configured correctly. The specified target cannot be used for code generation. Try with another target.",
                file,
                file_time,
            )?;
        }

        let target_machine: TargetMachine = target
            .create_target_machine(
                llvm_triple,
                llvm_cpu_name,
                llvm_cpu_features,
                llvm_opt,
                llvm_backend.get_reloc_mode(),
                llvm_backend.get_code_model(),
            )
            .ok_or_else(|| {
                let _ = interrupt::archive_compilation_unit_with_message(
                    self,
                    thrushc_logging::LoggingType::Error,
                    "The compiler couldn't be configured correctly. Possibly the target is not supported for code generation.",
                    file,
                    file_time,
                );
            })?;

        llvm_module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        let mut llvm_codegen_context: LLVMCodeGenContext = LLVMCodeGenContext::new(
            &llvm_module,
            &llvm_context,
            &llvm_builder,
            target_machine.get_target_data(),
            target_machine.get_triple(),
            &target_machine,
            Diagnostician::new(file, self.options),
            self.options,
            file,
        );

        thrushc_llvm_codegen::LLVMCompiler::compile(&mut llvm_codegen_context, ast);

        validate::llvm_codegen(&llvm_module, file)?;

        if print::llvm_before_optimization(self, &llvm_module, &target_machine, file, file_time)? {
            return finisher::archive_compilation(self, file_time, file);
        }

        if emit::llvm_before_optimization(
            self,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
            file_time,
        )? {
            return finisher::archive_compilation(self, file_time, file);
        }

        let llvm_optimizer_config: LLVMOptimizationConfig = LLVMOptimizationConfig::new(
            compiler_optimization,
            *llvm_backend.get_sanitizer(),
            *llvm_backend.get_symbol_linkage_strategy(),
            *llvm_backend.get_denormal_fp_behavior(),
            *llvm_backend.get_denormal_fp_32_bits_behavior(),
        );

        let llvm_optimizer_passes: LLVMOptimizerPasses<'_> = LLVMOptimizerPasses::new(
            llvm_backend.get_opt_passes(),
            llvm_backend.get_modificator_passes(),
        );

        let llvm_optimizer_flags: LLVMOptimizerFlags = LLVMOptimizerFlags::new(
            self.options.omit_default_optimizations(),
            llvm_backend.get_disable_all_sanitizers(),
        );

        thrushc_llvm_codegen::optimizer::LLVMOptimizer::new(
            &llvm_module,
            &llvm_context,
            &target_machine,
            llvm_optimizer_config,
            llvm_optimizer_flags,
            llvm_optimizer_passes,
        )
        .optimize();

        if print::llvm_after_optimization(self, &llvm_module, &target_machine, file, file_time)? {
            return finisher::archive_compilation(self, file_time, file);
        }

        if emit::llvm_after_optimization(
            self,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
            file_time,
        )? {
            return finisher::archive_compilation(self, file_time, file);
        }

        let obj_file: std::path::PathBuf = finisher::llvm_obj_compilation(
            &llvm_module,
            &target_machine,
            build_dir,
            file.get_name(),
        );

        self.add_compiled_unit(obj_file);

        finisher::archive_compilation(self, file_time, file)?;

        Ok(())
    }
}

impl<'thrushc> ThrushCompiler<'thrushc> {
    fn compile_jit_llvm(&mut self) -> (u128, u128) {
        cleaner::auto_clean(self.get_options());

        let context: Context = Context::create();

        let mut interrumped: bool = false;
        let mut modules: Vec<Module> = Vec::with_capacity(100_000);

        self.uncompiled.iter().for_each(|file| {
            let compiled_file: Result<either::Either<MemoryBuffer, ()>, ()> =
                self.compile_file_with_llvm_jit(file);

            interrumped = compiled_file.is_err();

            if let Some(module) = compiled_file
                .ok()
                .and_then(|either| either.left())
                .and_then(|memory_buffer| context.create_module_from_ir(memory_buffer).ok())
            {
                modules.push(module)
            }
        });

        if interrumped
            || self.get_options().get_was_printed()
            || self.get_options().get_was_emited()
            || modules.is_empty()
        {
            return (self.thrushc_time.as_millis(), self.linking_time.as_millis());
        }

        modules.reverse();

        if let Some(module) = modules.first() {
            let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
            let config: &JITConfiguration = llvm_backend.get_jit_config();
            let opt_level: OptimizationLevel = llvm_backend.get_optimization().to_llvm_opt();

            let engine: ExecutionEngine = match module.create_jit_execution_engine(opt_level) {
                Ok(engine) => engine,
                Err(_) => {
                    thrushc_logging::print_error(
                        thrushc_logging::LoggingType::Error,
                        "The JIT compiler couldn't be created correctly. Unexpected issue.",
                    );

                    return (self.thrushc_time.as_millis(), self.linking_time.as_millis());
                }
            };

            let llvm_jit: LLVMJITCompiler =
                thrushc_llvm_codegen::jit::LLVMJITCompiler::new(engine, config, modules);

            let llvm_jit_result: i32 = llvm_jit.compile_and_run().unwrap_or(1);

            std::process::exit(llvm_jit_result)
        } else {
            thrushc_logging::print_warn(
                thrushc_logging::LoggingType::Warning,
                "Nothing to compile for the JIT compiler. Skipping compilation.",
            );
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }

    fn compile_file_with_llvm_jit(
        &mut self,
        file: &'thrushc CompilationUnit,
    ) -> Result<either::Either<MemoryBuffer, ()>, ()> {
        let file_time: std::time::Instant = std::time::Instant::now();

        starter::archive_compilation_unit(file);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &std::path::PathBuf = self.options.get_build_dir();

        let Ok(tokens) = Lexer::lex(file, self.options) else {
            return interrupt::archive_compilation_unit_jit(self, file, file_time);
        };

        if print::after_frontend(self, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        let mut preprocessor: Preprocessor = Preprocessor::new();
        let modules: Result<&[thrushc_preprocessor::module::Module<'_>], ()> =
            preprocessor.generate_modules(&tokens, self.options, file);

        if modules.is_err() {
            return interrupt::archive_compilation_unit_jit(self, file, file_time);
        }

        let modules: &[thrushc_preprocessor::module::Module<'_>] = modules.map_err(|_| {
            let _ = interrupt::archive_compilation_unit_with_message(
                self,
                thrushc_logging::LoggingType::Error,
                "Failed to get all modules from the preprocessor. Maybe this is a issue.",
                file,
                file_time,
            );
        })?;

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file, self.options);

        let parser_result: (ParserContext, bool) = parser;
        let parser_throwed_errors: bool = parser_result.1;

        let parser_context: ParserContext = parser_result.0;

        let ast: &[Ast] = parser_context.get_ast();

        let semantic_analysis_throwed_errors: bool =
            SemantiAnalysis::new(ast, file, self.options).analyze(parser_throwed_errors);

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        let mut intrinsic_checker: LLVMIntrinsicChecker<'_> =
            LLVMIntrinsicChecker::new(ast, file, self.options);

        let mut call_conv_checker: LLVMCallConventionsChecker<'_> =
            LLVMCallConventionsChecker::new(ast, self.get_options(), file);

        let intrinsic_result: bool = intrinsic_checker.analyze();
        let call_conv_result: bool = call_conv_checker.analyze();

        if intrinsic_result || call_conv_result {
            return interrupt::archive_compilation_unit_jit(self, file, file_time);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Ast(ast)) {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        let llvm_context: Context = Context::create();
        let llvm_builder: Builder = llvm_context.create_builder();
        let llvm_module: Module = llvm_context.create_module(file.get_name());

        let target: &LLVMTarget = llvm_backend.get_target();

        let llvm_triple: &TargetTriple = target.get_triple();

        let llvm_cpu_name: &str = llvm_backend.get_target_cpu().get_cpu_name();
        let llvm_cpu_features: &str = llvm_backend.get_target_cpu().get_cpu_features();

        let compiler_optimization: ThrushOptimization = llvm_backend.get_optimization();
        let llvm_opt: OptimizationLevel = compiler_optimization.to_llvm_opt();

        llvm_module.set_triple(llvm_triple);

        let target: Target = Target::from_triple(llvm_triple).map_err(|_| {
            let _ = interrupt::archive_compilation_unit_with_message(
                self,
            thrushc_logging::LoggingType::Error,
                "Target-triple could not be built correctly. Maybe this target triple is invalid. Try another.",
                file,
                file_time,
            );
        })?;

        let target_machine: TargetMachine = target
            .create_target_machine(
                llvm_triple,
                llvm_cpu_name,
                llvm_cpu_features,
                llvm_opt,
                llvm_backend.get_reloc_mode(),
                llvm_backend.get_code_model(),
            )
            .ok_or_else(|| {
                let _ = interrupt::archive_compilation_unit_with_message(
                    self,
                    thrushc_logging::LoggingType::Error,
                    "Target machine could not be built correctly.",
                    file,
                    file_time,
                );
            })?;

        llvm_module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        jit::has_jit_available(&target)?;

        let mut llvm_codegen_context: LLVMCodeGenContext = LLVMCodeGenContext::new(
            &llvm_module,
            &llvm_context,
            &llvm_builder,
            target_machine.get_target_data(),
            target_machine.get_triple(),
            &target_machine,
            Diagnostician::new(file, self.options),
            self.options,
            file,
        );

        thrushc_llvm_codegen::LLVMCompiler::compile(&mut llvm_codegen_context, ast);

        validate::llvm_codegen(&llvm_module, file)?;

        if print::llvm_before_optimization(self, &llvm_module, &target_machine, file, file_time)? {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        if emit::llvm_before_optimization(
            self,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
            file_time,
        )? {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        let llvm_optimizer_config: LLVMOptimizationConfig = LLVMOptimizationConfig::new(
            compiler_optimization,
            *llvm_backend.get_sanitizer(),
            *llvm_backend.get_symbol_linkage_strategy(),
            *llvm_backend.get_denormal_fp_behavior(),
            *llvm_backend.get_denormal_fp_32_bits_behavior(),
        );

        let llvm_optimizer_passes: LLVMOptimizerPasses<'_> = LLVMOptimizerPasses::new(
            llvm_backend.get_opt_passes(),
            llvm_backend.get_modificator_passes(),
        );

        let llvm_optimizer_flags: LLVMOptimizerFlags = LLVMOptimizerFlags::new(
            self.options.omit_default_optimizations(),
            llvm_backend.get_disable_all_sanitizers(),
        );

        thrushc_llvm_codegen::optimizer::LLVMOptimizer::new(
            &llvm_module,
            &llvm_context,
            &target_machine,
            llvm_optimizer_config,
            llvm_optimizer_flags,
            llvm_optimizer_passes,
        )
        .optimize();

        if print::llvm_after_optimization(self, &llvm_module, &target_machine, file, file_time)? {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        if emit::llvm_after_optimization(
            self,
            &llvm_module,
            &target_machine,
            build_dir,
            file,
            file_time,
        )? {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        finisher::archive_compilation(self, file_time, file)?;

        Ok(either::Either::Left(llvm_module.write_bitcode_to_memory()))
    }
}

impl ThrushCompiler<'_> {
    #[inline]
    pub fn get_compiled_files(&self) -> &[std::path::PathBuf] {
        &self.compiled
    }

    #[inline]
    pub fn get_options(&self) -> &CompilerOptions {
        self.options
    }
}

impl ThrushCompiler<'_> {
    #[inline]
    pub fn add_compiled_unit(&mut self, path: std::path::PathBuf) {
        self.compiled.push(path);
    }
}
