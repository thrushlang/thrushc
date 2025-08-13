use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use colored::Colorize;
use inkwell::{
    module::Module,
    targets::{FileType, TargetMachine},
};

use crate::core::{
    compiler::{options::CompilerFile, thrushc::TheThrushCompiler},
    console::logging,
    utils::rand,
};

#[inline]
pub fn archive_compilation(
    compiler: &mut TheThrushCompiler,
    archive_time: Instant,
    file: &CompilerFile,
) -> Result<(), ()> {
    compiler.thrushc_time += archive_time.elapsed();

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FINISHED".bright_green().bold(),
            &file.path.to_string_lossy()
        ),
    );

    Ok(())
}

#[inline]
pub fn obj_compilation(
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
) -> PathBuf {
    let obj_file_path: PathBuf = build_dir.join(format!(
        "{}_{}.o",
        rand::generate_random_string(),
        file_name
    ));

    target_machine
        .write_to_file(llvm_module, FileType::Object, &obj_file_path)
        .unwrap_or_else(|_| {
            logging::log(
                logging::LoggingType::FrontEndPanic,
                &format!("'{}' cannot be emitted.", obj_file_path.display()),
            );

            unreachable!()
        });

    obj_file_path
}
