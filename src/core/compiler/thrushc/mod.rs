mod emit;
mod finisher;
mod interrupt;
mod linking;
mod print;
mod starter;
mod validate;

use either::Either;
use std::path::PathBuf;
use std::process;
use std::time::Duration;
use std::time::Instant;

use crate::back_end::llvm;
use crate::back_end::llvm::compiler::context::LLVMCodeGenContext;
use crate::back_end::llvm::compiler::jit::LLVMJITCompiler;
use crate::back_end::llvm::compiler::optimization::LLVMOptimizerFlags;

use crate::middle_end;

use crate::front_end::preprocessor;
use crate::front_end::preprocessor::Preprocessor;

use crate::core::compiler::backends::llvm::LLVMBackend;
use crate::core::compiler::backends::llvm::jit;
use crate::core::compiler::backends::llvm::jit::JITConfiguration;
use crate::core::compiler::backends::llvm::target::LLVMTarget;
use crate::core::compiler::emitters::cleaner;
use crate::core::compiler::linking::LinkingCompilersConfiguration;
use crate::core::compiler::options::CompilationUnit;
use crate::core::compiler::options::CompilerOptions;
use crate::core::compiler::options::Emited;
use crate::core::compiler::options::ThrushOptimization;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;
use crate::core::diagnostic::diagnostician::Diagnostician;

use crate::front_end::lexer::Lexer;
use crate::front_end::lexer::token::Token;
use crate::front_end::parser::Parser;
use crate::front_end::parser::ParserContext;
use crate::front_end::semantic::SemanticAnalyzer;
use crate::front_end::types::ast::Ast;

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

#[derive(Debug)]
pub struct ThrushCompiler<'thrushc> {
    compiled: Vec<PathBuf>,
    uncompiled: &'thrushc [CompilationUnit],

    options: &'thrushc CompilerOptions,

    linking_time: Duration,
    thrushc_time: Duration,
}

impl<'thrushc> ThrushCompiler<'thrushc> {
    pub fn new(files: &'thrushc [CompilationUnit], options: &'thrushc CompilerOptions) -> Self {
        Self {
            compiled: Vec::with_capacity(files.len()),
            uncompiled: files,

            options,

            linking_time: Duration::default(),
            thrushc_time: Duration::default(),
        }
    }
}

impl ThrushCompiler<'_> {
    pub fn compile(&mut self) -> (u128, u128) {
        if self.get_options().uses_llvm() {
            Target::initialize_all(&InitializationConfig::default());

            if self.get_options().get_llvm_backend_options().is_jit() {
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
            linking::link_with_clang(self);
        }

        if linking_compiler_config.get_use_gcc() {
            linking::link_with_gcc(self);
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }

    fn compile_file_with_llvm_aot(&mut self, file: &'thrushc CompilationUnit) -> Result<(), ()> {
        let file_time: Instant = Instant::now();

        starter::archive_compilation_unit(file);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        let tokens: Vec<Token> = Lexer::lex(file).unwrap_or_else(|error| {
            logging::print_frontend_panic(LoggingType::FrontEndPanic, &error.display())
        });

        if print::after_frontend(self, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation(self, file_time, file);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation(self, file_time, file);
        }

        let mut preprocessor: Preprocessor = Preprocessor::new(&tokens, file);
        let modules: Vec<preprocessor::module::Module> = preprocessor.generate_modules()?;

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file, modules);

        let parser_result: (ParserContext, bool) = parser;
        let parser_throwed_errors: bool = parser_result.1;

        let parser_context: ParserContext = parser_result.0;

        let ast: &[Ast] = parser_context.get_ast();

        let semantic_analysis_throwed_errors: bool =
            SemanticAnalyzer::new(ast, file).check(parser_throwed_errors);

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            return finisher::archive_compilation(self, file_time, file);
        }

        let mut intrinsic_checker: middle_end::llvm::intrinsic_checker::IntrinsicChecker =
            middle_end::llvm::intrinsic_checker::IntrinsicChecker::new(ast, file);

        intrinsic_checker.check()?;

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

        let thrush_opt: ThrushOptimization = llvm_backend.get_optimization();
        let llvm_opt: OptimizationLevel = thrush_opt.to_llvm_opt();

        llvm_module.set_triple(llvm_triple);

        let target: Target = Target::from_triple(llvm_triple).map_err(|_| {
            let _ = interrupt::archive_compilation_unit_with_message(
                self,
                LoggingType::LLVMBackend,
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
                    LoggingType::LLVMBackend,
                    "Target machine could not be built correctly.",
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
            Diagnostician::new(file),
            self.options,
        );

        llvm::compiler::LLVMCompiler::compile(&mut llvm_codegen_context, ast);

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

        llvm::compiler::optimization::LLVMOptimizer::new(
            &llvm_module,
            &llvm_context,
            LLVMOptimizerFlags::new(self.options.disable_default_opt()),
            &target_machine,
            llvm_opt,
            llvm_backend.get_opt_passes(),
            llvm_backend.get_modificator_passes(),
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

        let compiled_file: PathBuf = finisher::llvm_obj_compilation(
            &llvm_module,
            &target_machine,
            build_dir,
            file.get_name(),
        );

        self.add_compiled_unit(compiled_file);

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
            let compiled_file: Result<Either<MemoryBuffer, ()>, ()> =
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
                    logging::print_error(
                        LoggingType::LLVMBackend,
                        "The compiler just in time could not be created correctly.",
                    );

                    return (self.thrushc_time.as_millis(), self.linking_time.as_millis());
                }
            };

            let llvm_jit: LLVMJITCompiler =
                llvm::compiler::jit::LLVMJITCompiler::new(engine, config, modules);

            let llvm_jit_result: i32 = llvm_jit.compile_and_run().unwrap_or(1);

            process::exit(llvm_jit_result)
        }

        (self.thrushc_time.as_millis(), self.linking_time.as_millis())
    }

    fn compile_file_with_llvm_jit(
        &mut self,
        file: &'thrushc CompilationUnit,
    ) -> Result<Either<MemoryBuffer, ()>, ()> {
        let file_time: Instant = Instant::now();

        starter::archive_compilation_unit(file);

        let llvm_backend: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        let tokens: Vec<Token> = Lexer::lex(file).unwrap_or_else(|error| {
            logging::print_frontend_panic(LoggingType::FrontEndPanic, &error.display())
        });

        if print::after_frontend(self, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        if emit::after_frontend(self, build_dir, file, Emited::Tokens(&tokens)) {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        let mut preprocessor: Preprocessor = Preprocessor::new(&tokens, file);
        let modules: Vec<preprocessor::module::Module> = preprocessor.generate_modules()?;

        let parser: (ParserContext, bool) = Parser::parse(&tokens, file, modules);

        let parser_result: (ParserContext, bool) = parser;
        let parser_throwed_errors: bool = parser_result.1;

        let parser_context: ParserContext = parser_result.0;

        let ast: &[Ast] = parser_context.get_ast();

        let semantic_analysis_throwed_errors: bool =
            SemanticAnalyzer::new(ast, file).check(parser_throwed_errors);

        if parser_throwed_errors || semantic_analysis_throwed_errors {
            return finisher::archive_compilation_module_jit(self, file_time, file);
        }

        let mut intrinsic_checker: middle_end::llvm::intrinsic_checker::IntrinsicChecker =
            middle_end::llvm::intrinsic_checker::IntrinsicChecker::new(ast, file);

        intrinsic_checker.check()?;

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

        let thrush_opt: ThrushOptimization = llvm_backend.get_optimization();
        let llvm_opt: OptimizationLevel = thrush_opt.to_llvm_opt();

        llvm_module.set_triple(llvm_triple);

        let target: Target = Target::from_triple(llvm_triple).map_err(|_| {
            let _ = interrupt::archive_compilation_unit_with_message(
                self,
                LoggingType::LLVMBackend,
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
                    LoggingType::LLVMBackend,
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
            Diagnostician::new(file),
            self.options,
        );

        llvm::compiler::LLVMCompiler::compile(&mut llvm_codegen_context, ast);

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

        llvm::compiler::optimization::LLVMOptimizer::new(
            &llvm_module,
            &llvm_context,
            LLVMOptimizerFlags::new(self.options.disable_default_opt()),
            &target_machine,
            llvm_opt,
            llvm_backend.get_opt_passes(),
            llvm_backend.get_modificator_passes(),
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

        Ok(Either::Left(llvm_module.write_bitcode_to_memory()))
    }
}

impl ThrushCompiler<'_> {
    #[inline]
    pub fn get_compiled_files(&self) -> &[PathBuf] {
        &self.compiled
    }

    #[inline]
    pub fn get_options(&self) -> &CompilerOptions {
        self.options
    }
}

impl ThrushCompiler<'_> {
    pub fn add_compiled_unit(&mut self, path: PathBuf) {
        self.compiled.push(path);
    }
}
