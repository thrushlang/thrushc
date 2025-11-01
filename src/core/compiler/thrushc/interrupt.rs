use std::time::Instant;

use colored::Colorize;

use crate::core::{
    compiler::{options::CompilationUnit, thrushc::ThrushCompiler},
    console::logging::{self, LoggingType},
};

#[inline]
pub fn archive_compilation_unit(
    compiler: &mut ThrushCompiler,
    file: &CompilationUnit,
    file_time: Instant,
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
    file_time: Instant,
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
