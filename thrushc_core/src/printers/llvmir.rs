use colored::Colorize;

use inkwell::module::Module;
use thrushc_options::CompilerOptions;

use crate::ThrushCompiler;
use crate::utils;

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
            utils::generate_random_string(thrushc_constants::COMPILER_HARD_OBFUSCATION_LEVEL),
            file_name
        )
    } else {
        format!("{}{}.ll", optimization_name_modifier, file_name)
    };

    let module_print: String = llvm_module.print_to_string().to_string();

    thrushc_logging::write(
        thrushc_logging::OutputIn::Stdout,
        &format!(
            "{}{}\n\n",
            "LLVM IR FILE - ".bold(),
            ir_file_name.bright_green().bold(),
        ),
    );

    #[cfg(feature = "utils")]
    {
        if compiler_options.need_copy_output_to_clipboard() {
            use clipboard::*;

            let ctx: Result<ClipboardContext, Box<dyn std::error::Error>> =
                ClipboardProvider::new();

            if let Ok(mut ctx) = ctx {
                ctx.set_contents(module_print.clone()).unwrap_or_else(|_| {
                    thrushc_logging::print_warn(
                        thrushc_logging::LoggingType::Warning,
                        "Unable to copy the IR code into system clipboard.",
                    );
                });
            } else {
                thrushc_logging::print_warn(
                    thrushc_logging::LoggingType::Warning,
                    "Failed to initialize clipboard processes.",
                );
            }
        }
    }

    thrushc_logging::write(thrushc_logging::OutputIn::Stdout, &module_print);
    thrushc_logging::write(thrushc_logging::OutputIn::Stdout, "\n");
}
