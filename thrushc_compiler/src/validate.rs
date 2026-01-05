use colored::Colorize;

use inkwell::module::Module;
use thrushc_options::CompilationUnit;

pub fn llvm_codegen(llvm_module: &Module, file: &CompilationUnit) -> Result<(), ()> {
    if let Err(codegen_error) = llvm_module.verify() {
        thrushc_logging::print_backend_panic_not_exit(
            thrushc_logging::LoggingType::BackendPanic,
            codegen_error.to_string().trim_end(),
        );

        thrushc_logging::write(
            thrushc_logging::OutputIn::Stderr,
            &format!(
                "\r{} {} {}\n",
                "Compilation".custom_color((141, 141, 142)).bold(),
                "FAILED".bright_red().bold(),
                file.get_path().display()
            ),
        );

        return Err(());
    }

    Ok(())
}
