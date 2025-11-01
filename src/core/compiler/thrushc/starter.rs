use std::path::PathBuf;

use colored::Colorize;

use crate::core::{compiler::options::CompilationUnit, console::logging};

#[inline]
pub fn archive_compilation_unit(file: &CompilationUnit) {
    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "STARTING".bright_green().bold(),
            &file.get_path().display()
        ),
    );
}

#[inline]
pub fn linking_phase(files: &[PathBuf]) {
    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{} {} {}\n",
            "Linking".custom_color((141, 141, 142)).bold(),
            "RUNNING".bright_green().bold(),
            files
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ),
    );
}
