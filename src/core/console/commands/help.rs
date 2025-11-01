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
            "[-flags | --flags] [files..]"
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
            "llvm-print-targets".custom_color((141, 141, 142)).bold(),
            "Show the current LLVM target supported."
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
            "Show the current LLVM supported CPUs for the current LLVM target.",
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
            "Show the host LLVM target triple.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "llvm-print-opt-passes".custom_color((141, 141, 142)).bold(),
            "Show all available optimization passes through '--opt-passes=p{passname, passname}' in the compiler for the LLVM backend.",
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
            "-clang-link".custom_color((141, 141, 142)).bold(),
            "Enable embedded Clang for linking.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-gcc-link".custom_color((141, 141, 142)).bold(),
            "usr/bin/gcc",
            "Specifies GNU Compiler Collection (GCC) for linking.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-custom-clang-link".custom_color((141, 141, 142)).bold(),
            "/usr/bin/clang",
            "Specifies the path for use of an external Clang for linking.",
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
            "-llvm-backend".custom_color((141, 141, 142)).bold(),
            "Enable the usage of the LLVM backend infrastructure.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-target".custom_color((141, 141, 142)).bold(),
            "x86_64",
            "Set the LLVM target arquitecture.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-target-triple".custom_color((141, 141, 142)).bold(),
            "x86_64-pc-linux-gnu",
            "Set the LLVM backend target triple. For more information, see 'https://clang.llvm.org/docs/CrossCompilation.html'.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-cpu".custom_color((141, 141, 142)).bold(),
            "haswell",
            "Specify in LLVM the CPU to optimize.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} [{}] {}\n",
            "•".bold(),
            "-cpu-features".custom_color((141, 141, 142)).bold(),
            "+sse2,+cx16,+sahf,-tbm",
            "Specify in LLVM the new features of the CPU to use.",
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
            "-print".custom_color((141, 141, 142)).bold(),
            "llvm-ir|raw-llvm-ir|tokens",
            "Displays the final compilation on stdout.",
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
            "[-p{passname,passname}]",
            "Pass a list of custom optimization passes to the LLVM backend. For more information, see: 'https://releases.llvm.org/17.0.1/docs/CommandGuide/opt.html#cmdoption-opt-passname'.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {} {}\n",
            "•".bold(),
            "--modificator-passes".custom_color((141, 141, 142)).bold(),
            "[loopvectorization;loopunroll;loopinterleaving;loopsimplifyvectorization;mergefunctions;callgraphprofile;forgetallscevinloopunroll;licmmssaaccpromcap=0;licmmssaoptcap=0;]",
            "Pass a list of custom modificator optimization passes to the LLVM backend.",
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

    logging::write(logging::OutputIn::Stderr, "\nSpecial flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-llinker".custom_color((141, 141, 142)).bold(),
            "Transform the compiler into the LLVM linker.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "-llinker-flavor".custom_color((141, 141, 142)).bold(),
            "Specify the build flavor for the LLVM linker.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "\nUseful flags:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--debug-clang-command".custom_color((141, 141, 142)).bold(),
            "Displays the generated command for Clang in the phase of linking."
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--debug-gcc-commands".custom_color((141, 141, 142)).bold(),
            "Displays the generated command for GCC in the phase of linking.\n"
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

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•".bold(),
            "--no-obfuscate-ir".custom_color((141, 141, 142)).bold(),
            "Stop generating name obfuscation in the emitted IR code."
        ),
    );

    process::exit(1);
}
