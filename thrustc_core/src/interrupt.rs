use colored::Colorize;
use inkwell::memory_buffer::MemoryBuffer;
use thrustc_logging::LoggingType;
use thrustc_options::CompilationUnit;

use crate::ThrustCompiler;

#[inline]
pub fn archive_compilation_unit(
    compiler: &mut ThrustCompiler,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<(), ()> {
    compiler.thrustc_time += file_time.elapsed();

    thrustc_logging::write(
        thrustc_logging::OutputIn::Stderr,
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
    compiler: &mut ThrustCompiler,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<either::Either<MemoryBuffer, ()>, ()> {
    compiler.thrustc_time += file_time.elapsed();

    thrustc_logging::write(
        thrustc_logging::OutputIn::Stderr,
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
    compiler: &mut ThrustCompiler,
    log_type: LoggingType,
    msg: &str,
    file: &CompilationUnit,
    file_time: std::time::Instant,
) -> Result<(), ()> {
    thrustc_logging::print_error(log_type, msg);

    compiler.thrustc_time += file_time.elapsed();

    thrustc_logging::write(
        thrustc_logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold(),
            &file.get_path().to_string_lossy()
        ),
    );

    Err(())
}
