use colored::Colorize;

use crate::{
    backends::classical::linking::compilers::{clang::Clang, gcc::GCC},
    core::{
        compiler::{
            backends::llvm::LLVMBackend, linking::LinkingCompilersConfiguration,
            thrushc::TheThrushCompiler,
        },
        console::logging::{self},
    },
};

use std::path::PathBuf;

pub fn link_with_clang(compiler: &mut TheThrushCompiler) {
    let llvm_backend: &LLVMBackend = compiler.get_options().get_llvm_backend_options();

    let linking_compiler_config: &LinkingCompilersConfiguration =
        compiler.get_options().get_linking_compilers_configuration();

    let all_compiled_files: &[PathBuf] = compiler.get_compiled_files();

    if let Ok(clang_time) =
        Clang::new(all_compiled_files, linking_compiler_config, llvm_backend).link()
    {
        compiler.linking_time += clang_time;

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold()
            ),
        );

        return;
    }

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\r{} {}\n",
            "Linking".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold()
        ),
    );
}

pub fn link_with_gcc(compiler: &mut TheThrushCompiler) {
    let linking_compiler_configuration: &LinkingCompilersConfiguration =
        compiler.get_options().get_linking_compilers_configuration();

    let all_compiled_files: &[PathBuf] = compiler.get_compiled_files();

    if let Ok(gcc_time) = GCC::new(all_compiled_files, linking_compiler_configuration).link() {
        compiler.linking_time += gcc_time;

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "{} {}\n",
                "Linking".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold()
            ),
        );

        return;
    }

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\r{} {}\n",
            "Linking".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold()
        ),
    );
}
