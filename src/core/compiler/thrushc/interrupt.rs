use std::time::Instant;

use colored::Colorize;

use crate::core::{
    compiler::{options::CompilationUnit, thrushc::ThrushCompiler},
    console::logging,
};

#[inline]
pub fn archive_compilation_unit(
    compiler: &mut ThrushCompiler,
    archive_time: Instant,
    file: &CompilationUnit,
) -> Result<(), ()> {
    compiler.thrushc_time += archive_time.elapsed();

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
