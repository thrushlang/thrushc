use colored::Colorize;

use thrushc_options::CompilationUnit;

#[inline]
pub fn archive_compilation_unit(file: &CompilationUnit) {
    thrushc_logging::write(
        thrushc_logging::OutputIn::Stdout,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "STARTING".bright_green().bold(),
            &file.get_path().display()
        ),
    );
}

#[inline]
pub fn linking_phase(files: &[std::path::PathBuf]) {
    thrushc_logging::write(
        thrushc_logging::OutputIn::Stdout,
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
