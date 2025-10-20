use colored::Colorize;
use inkwell::module::Module;

use crate::core::{
    compiler::{options::CompilerOptions, thrushc::ThrushCompiler},
    console::logging,
    constants,
    utils::rand,
};

pub fn print_llvm_ir(
    compiler: &ThrushCompiler,
    llvm_module: &Module,
    file_name: &str,
    unoptimized: bool,
) {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let obfuscate: bool = compiler_options.need_obfuscate_archive_names();

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let ir_file_name: String = if obfuscate {
        format!(
            "{}{}_{}.ll",
            optimization_name_modifier,
            rand::generate_random_string(constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.ll", optimization_name_modifier, file_name)
    };

    let module_print: String = llvm_module.print_to_string().to_string();

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{}{}\n\n",
            "LLVM IR FILE - ".bold(),
            ir_file_name.bright_green().bold(),
        ),
    );

    logging::write(logging::OutputIn::Stdout, &module_print);
    logging::write(logging::OutputIn::Stdout, "\n");
}
