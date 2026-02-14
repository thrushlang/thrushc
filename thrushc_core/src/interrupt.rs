use colored::Colorize;
use inkwell::memory_buffer::MemoryBuffer;
use thrushc_logging::LoggingType;
use thrushc_options::CompilationUnit;

use crate::ThrushCompiler;

#[inline]
pub fn archive_compilation_unit(
    compiler: &mut ThrushCompiler,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<(), ()> {
    compiler.thrushc_time += file_time.elapsed();

    thrushc_logging::write(
        thrushc_logging::OutputIn::Stderr,
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
pub fn archive_compilation_unit_jit(
    compiler: &mut ThrushCompiler,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<either::Either<MemoryBuffer, ()>, ()> {
    compiler.thrushc_time += file_time.elapsed();

    thrushc_logging::write(
        thrushc_logging::OutputIn::Stderr,
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
    thrushc_logging::print_error(log_type, msg);

    compiler.thrushc_time += file_time.elapsed();

    thrushc_logging::write(
        thrushc_logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold(),
            &file.get_path().to_string_lossy()
        ),
    );

    Err(())
}
