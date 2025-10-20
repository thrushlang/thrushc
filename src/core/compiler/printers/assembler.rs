use std::borrow::Cow;

use colored::Colorize;
use inkwell::{
    memory_buffer::MemoryBuffer,
    module::Module,
    support::LLVMString,
    targets::{FileType, TargetMachine},
};

use crate::core::{
    compiler::{options::CompilerOptions, thrushc::ThrushCompiler},
    console::logging,
    constants,
    utils::rand,
};

pub fn print_llvm_assembler(
    compiler: &ThrushCompiler,
    target_machine: &TargetMachine,
    llvm_module: &Module,
    file_name: &str,
    unoptimized: bool,
) -> Result<(), LLVMString> {
    let compiler_options: &CompilerOptions = compiler.get_options();
    let obfuscate: bool = compiler_options.need_obfuscate_archive_names();

    let optimization_name_modifier: &str = if unoptimized { "raw_" } else { "" };

    let assembler_file_name: String = if obfuscate {
        format!(
            "{}{}_{}.s",
            optimization_name_modifier,
            rand::generate_random_string(constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.s", optimization_name_modifier, file_name)
    };

    let memory_buffer: MemoryBuffer =
        target_machine.write_to_memory_buffer(llvm_module, FileType::Assembly)?;

    let assembler_in_bytes: &[u8] = memory_buffer.as_slice();
    let assembler: Cow<'_, str> = String::from_utf8_lossy(assembler_in_bytes);

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{}{}\n\n",
            "ASSEMBLER FILE - ".bold(),
            assembler_file_name.bright_green().bold(),
        ),
    );

    logging::write(logging::OutputIn::Stdout, &assembler);
    logging::write(logging::OutputIn::Stdout, "\n");

    Ok(())
}
