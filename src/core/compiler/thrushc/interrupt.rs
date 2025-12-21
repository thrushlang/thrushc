use colored::Colorize;

use crate::core::compiler::options::CompilationUnit;
use crate::core::compiler::thrushc::ThrushCompiler;
use crate::core::console::logging;
use crate::core::console::logging::LoggingType;

#[inline]
pub fn archive_compilation_unit(
    compiler: &mut ThrushCompiler,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<(), ()> {
    compiler.thrushc_time += file_time.elapsed();

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold(),
            &file.get_path().to_string_lossy()
        ),
    );

    Err(())
}

#[inline]
pub fn archive_compilation_unit_with_message(
    compiler: &mut ThrushCompiler,
    log_type: LoggingType,
    msg: &str,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<(), ()> {
    logging::print_error(log_type, msg);

    compiler.thrushc_time += file_time.elapsed();

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold(),
            &file.get_path().to_string_lossy()
        ),
    );

    Err(())
}
