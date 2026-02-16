use colored::Colorize;

use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Module;
use inkwell::support::LLVMString;
use inkwell::targets::FileType;
use inkwell::targets::TargetMachine;
use thrustc_options::CompilerOptions;

use crate::ThrustCompiler;
use crate::utils;

pub fn print_llvm_assembler(
    compiler: &ThrustCompiler,
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
            utils::generate_random_string(thrustc_constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.s", optimization_name_modifier, file_name)
    };

    let memory_buffer: MemoryBuffer =
        target_machine.write_to_memory_buffer(llvm_module, FileType::Assembly)?;

    let assembler_in_bytes: Vec<u8> = memory_buffer.as_slice().to_vec();
    let assembler: String = unsafe { String::from_utf8_unchecked(assembler_in_bytes) };

    #[cfg(feature = "utils")]
    {
        if compiler_options.need_copy_output_to_clipboard() {
            use clipboard::*;

            let ctx: Result<ClipboardContext, Box<dyn std::error::Error>> =
                ClipboardProvider::new();

            if let Ok(mut ctx) = ctx {
                ctx.set_contents(assembler.clone()).unwrap_or_else(|_| {
                    thrustc_logging::print_warn(
                        thrustc_logging::LoggingType::Warning,
                        "Unable to copy the assembler code into system clipboard.",
                    );
                });
            } else {
                thrustc_logging::print_warn(
                    thrustc_logging::LoggingType::Warning,
                    "Failed to initialize clipboard processes.",
                );
            }
        }
    }

    thrustc_logging::write(
        thrustc_logging::OutputIn::Stdout,
        &format!(
            "{}{}\n\n",
            "ASSEMBLER FILE - ".bold(),
            assembler_file_name.bright_green().bold(),
        ),
    );

    thrustc_logging::write(thrustc_logging::OutputIn::Stdout, &assembler);
    thrustc_logging::write(thrustc_logging::OutputIn::Stdout, "\n");

    Ok(())
}
