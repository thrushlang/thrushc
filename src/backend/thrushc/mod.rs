use std::{
    fs::write,
    path::PathBuf,
    time::{Duration, Instant},
};

use colored::Colorize;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    module::Module,
    targets::{Target, TargetMachine, TargetTriple},
};

use crate::{
    frontend::{
        lexer::{Lexer, Token},
        parser::{Parser, ParserContext},
    },
    middle::instruction::Instruction,
    standard::{
        diagnostic::Diagnostician,
        logging::{self},
        misc::{CompilerFile, CompilerOptions, Emitable, LLVMBackend, Opt},
    },
};

use super::llvm::{self, clang::Clang};

pub mod handler;
pub mod utils;

pub struct Thrushc<'a> {
    thrushc_compiled_files: Vec<PathBuf>,
    files: &'a [CompilerFile],
    options: &'a CompilerOptions,
    llvm_time: Duration,
    thrushc_time: Duration,
}

impl<'a> Thrushc<'a> {
    pub fn new(files: &'a [CompilerFile], options: &'a CompilerOptions) -> Self {
        Self {
            thrushc_compiled_files: Vec::with_capacity(files.len()),
            files,
            options,
            llvm_time: Duration::default(),
            thrushc_time: Duration::default(),
        }
    }

    pub fn compile(&mut self) -> (u128, u128) {
        self.files.iter().for_each(|file| {
            self.compile_file(file);
        });

        let options: &LLVMBackend = self.options.get_llvm_backend_options();

        if options.was_emited() {
            return (self.thrushc_time.as_millis(), self.llvm_time.as_millis());
        }

        let clang_time: Duration =
            Clang::new(&self.thrushc_compiled_files, options.get_arguments()).compile();

        self.llvm_time += clang_time;

        (self.thrushc_time.as_millis(), self.llvm_time.as_millis())
    }

    fn compile_file(&mut self, file: &'a CompilerFile) {
        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Compiling".custom_color((141, 141, 142)).bold(),
                &file.path.to_string_lossy()
            ),
        );

        let thrushc_time: Instant = Instant::now();

        let source_code: &[u8] = &utils::extract_code_from_file(&file.path);

        let tokens: Vec<Token> = Lexer::lex(source_code, file);

        let options: &LLVMBackend = self.options.get_llvm_backend_options();
        let build_dir: &PathBuf = self.options.get_build_dir();

        if options.contains_emitable(Emitable::Tokens) {
            let _ = write(
                build_dir.join(format!("{}.tokens", file.name)),
                format!("{:#?}", tokens),
            );

            self.thrushc_time += thrushc_time.elapsed();

            return;
        }

        let parser_ctx: ParserContext = Parser::parse(&tokens, file);
        let instructions: &[Instruction] = parser_ctx.get_instructions();

        if options.contains_emitable(Emitable::AST) {
            let _ = write(
                build_dir.join(format!("{}.ast", file.name)),
                format!("{:#?}", instructions),
            );

            self.thrushc_time += thrushc_time.elapsed();

            return;
        }

        let context: Context = Context::create();
        let builder: Builder = context.create_builder();
        let module: Module = context.create_module(&file.name);

        let target_triple: &TargetTriple = options.get_target_triple();

        module.set_triple(target_triple);

        let thrush_opt: Opt = options.get_opt();
        let llvm_opt: OptimizationLevel = thrush_opt.to_llvm_opt();

        let target_machine: TargetMachine = Target::from_triple(target_triple)
            .unwrap()
            .create_target_machine(
                target_triple,
                "",
                "",
                llvm_opt,
                options.get_reloc_mode(),
                options.get_code_model(),
            )
            .unwrap();

        module.set_data_layout(&target_machine.get_target_data().get_data_layout());

        llvm::compiler::Compiler::compile(
            &module,
            &builder,
            &context,
            instructions,
            target_machine.get_target_data(),
            Diagnostician::new(file),
        );

        self.thrushc_time += thrushc_time.elapsed();

        let bitcode_compiled_path: PathBuf = build_dir.join(format!("{}.bc", &file.name));

        if options.contains_emitable(Emitable::RawLLVMIR) {
            let output_path: PathBuf = build_dir.join(format!("{}.ll", &file.name));

            module.print_to_file(&output_path).unwrap_or_else(|_| {
                logging::log(
                    logging::LoggingType::Panic,
                    &format!("'{}' cannot be emitted.", output_path.display()),
                );
                unreachable!()
            });

            return;
        }

        if options.contains_emitable(Emitable::LLVMBitcode) {
            module.write_bitcode_to_path(&bitcode_compiled_path);
            return;
        }

        module.write_bitcode_to_path(&bitcode_compiled_path);

        self.thrushc_compiled_files.push(bitcode_compiled_path);
    }
}
