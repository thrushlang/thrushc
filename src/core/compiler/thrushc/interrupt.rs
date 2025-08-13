use std::time::Instant;

use colored::Colorize;

use crate::core::{
    compiler::{options::CompilerFile, thrushc::TheThrushCompiler},
    console::logging,
};

#[inline]
pub fn archive_compilation_unit(
    compiler: &mut TheThrushCompiler,
    archive_time: Instant,
    file: &CompilerFile,
) -> Result<(), ()> {
    compiler.thrushc_time += archive_time.elapsed();

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FAILED".bright_red().bold(),
            &file.path.to_string_lossy()
        ),
    );

    Err(())
}
