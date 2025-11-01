use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use colored::Colorize;

use either::Either;
use inkwell::{
    memory_buffer::MemoryBuffer,
    module::Module,
    targets::{FileType, TargetMachine},
};

use crate::core::{
    compiler::{options::CompilationUnit, thrushc::ThrushCompiler},
    console::logging,
    constants,
    utils::rand,
};

#[inline]
pub fn archive_compilation(
    compiler: &mut ThrushCompiler,
    file_time: Instant,
    file: &CompilationUnit,
) -> Result<(), ()> {
    compiler.thrushc_time += file_time.elapsed();

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FINISHED".bright_green().bold(),
            &file.get_path().to_string_lossy()
        ),
    );

    Ok(())
}

#[inline]
pub fn archive_compilation_module_jit(
    compiler: &mut ThrushCompiler,
    file_time: Instant,
    file: &CompilationUnit,
) -> Result<Either<MemoryBuffer, ()>, ()> {
    compiler.thrushc_time += file_time.elapsed();

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "{} {} {}\n",
            "Compilation".custom_color((141, 141, 142)).bold(),
            "FINISHED".bright_green().bold(),
            &file.get_path().to_string_lossy()
        ),
    );

    Ok(Either::Right(()))
}

#[inline]
pub fn llvm_obj_compilation(
    llvm_module: &Module,
    target_machine: &TargetMachine,
    build_dir: &Path,
    file_name: &str,
) -> PathBuf {
    let obj_file_path: PathBuf = build_dir.join(format!(
        "{}_{}.o",
        rand::generate_random_string(constants::COMPILER_HARD_OBFUSCATION_LEVEL),
        file_name
    ));

    target_machine
        .write_to_file(llvm_module, FileType::Object, &obj_file_path)
        .unwrap_or_else(|error| {
            logging::print_backend_panic(
                logging::LoggingType::BackendPanic,
                &format!(
                    "'{}' cannot be emited as object file because LLVM: '{}'.",
                    obj_file_path.display(),
                    error
                ),
            );
        });

    obj_file_path
}
