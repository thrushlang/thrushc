use std::process;

use colored::Colorize;

use crate::core::console::logging;

pub fn show_help() -> ! {
    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{}",
            "The Thrush Compiler".custom_color((141, 141, 142)).bold()
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\n\n{} {} {}\n\n",
            "Usage:".bold(),
            "thrushc".custom_color((141, 141, 142)).bold(),
            "[--flags] [files..]"
        ),
    );

    logging::write(logging::OutputIn::Stderr, "General Commands:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {}, {}, {} {}\n",
            "•".bold(),
            "-h".custom_color((141, 141, 142)).bold(),
            "--help".custom_color((141, 141, 142)).bold(),
            "help".custom_color((141, 141, 142)).bold(),
            "Show help message.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {}, {}, {} {}\n\n",
            "•".bold(),
            "-v".custom_color((141, 141, 142)).bold(),
            "--version".custom_color((141, 141, 142)).bold(),
            "version".custom_color((141, 141, 142)).bold(),
            "Show the version.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "LLVM Commands:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-print-target-triples"
                .custom_color((141, 141, 142))
                .bold(),
            "Show the current LLVM target triples supported."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-print-supported-cpus"
                .custom_color((141, 141, 142))
                .bold(),
            "Show the current LLVM supported CPUs.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-print-host-target-triple"
                .custom_color((141, 141, 142))
                .bold(),
            "Show the host LLVM target-triple.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nGeneral flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-build-dir".custom_color((141, 141, 142)).bold(),
            "Set the build directory.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-clang".custom_color((141, 141, 142)).bold(),
            "Enable embedded Clang to link.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-gcc".custom_color((141, 141, 142)).bold(),
            "\"/usr/bin/gcc\"",
            "Speciefies GNU Compiler Collection (GCC) to link.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-custom-clang".custom_color((141, 141, 142)).bold(),
            "\"/usr/bin/clang\"",
            "Specifies the path for use of an external Clang to link.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-start".custom_color((141, 141, 142)).bold(),
            "Marks the start of arguments to the active external or built-in linking compiler.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-end".custom_color((141, 141, 142)).bold(),
            "Marks the end of arguments to the active external or built-in linker compiler.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nCompiler flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-llvm".custom_color((141, 141, 142)).bold(),
            "Enable the usage of the LLVM backend infrastructure.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-cpu".custom_color((141, 141, 142)).bold(),
            "\"haswell\"",
            "Specify the CPU to optimize.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-target".custom_color((141, 141, 142)).bold(),
            "\"x86_64-pc-linux-gnu\"",
            "Set the target triple.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-emit".custom_color((141, 141, 142)).bold(),
            "llvm-bc|llvm-ir|asm|raw-llvm-ir|raw-llvm-bc|raw-asm|obj|ast|tokens",
            "Compile the code into specified representation.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-opt".custom_color((141, 141, 142)).bold(),
            "O0|O1|O2|mcqueen",
            "Optimization level.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nExtra compiler flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--opt-passes".custom_color((141, 141, 142)).bold(),
            "[-p{passname}]",
            "Pass a list of custom optimization passes to the LLVM optimizator.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--modificator-passes".custom_color((141, 141, 142)).bold(),
            "[loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions]",
            "Pass a list of custom modificator passes to the LLVM optimizator.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--reloc".custom_color((141, 141, 142)).bold(),
            "[static|pic|dynamic]",
            "Indicate how references to memory addresses and linkage symbols are handled."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--codemodel".custom_color((141, 141, 142)).bold(),
            "[small|medium|large|kernel]",
            "Define how code is organized and accessed at machine code level."
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nUseful flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--debug-clang-command".custom_color((141, 141, 142)).bold(),
            "Displays the generated command for Clang."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--debug-gcc-commands".custom_color((141, 141, 142)).bold(),
            "Displays the generated command for GCC.\n"
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-tokens".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder that holds the lexical analysis tokens."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-assembler".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing emitted assembler."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-llvm-ir".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing the emitted LLVM IR."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--clean-llvm-bitcode".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing emitted LLVM Bitcode."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n\n",
            "•".bold(),
            "--clean-objects".custom_color((141, 141, 142)).bold(),
            "Clean the compiler folder containing emitted object files."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--no-obfuscate-archive-names"
                .custom_color((141, 141, 142))
                .bold(),
            "Stop generating name obfuscation for each file; this does not apply to the final build."
        ),
    );

    process::exit(1);
}
