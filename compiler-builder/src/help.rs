use crate::logging;

pub fn show_help() -> ! {
    logging::write(logging::OutputIn::Stderr, "The Compiler Builder");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "\n\n{} {} {}\n\n",
            "Usage:", "compiler-builder", "[--flags]"
        ),
    );

    logging::write(logging::OutputIn::Stderr, "Commands:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {}, {}, {} {}\n",
            "•", "-h", "--help", "help", "Show help message.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {}, {}, {} {}\n\n",
            "•", "-v", "--version", "version", "Show the version.",
        ),
    );

    logging::write(logging::OutputIn::Stderr, "LLVM Backend:\n\n");

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n",
            "•",
            "--llvm-host-triple",
            "Specify the target-triple to download and install the LLVM backend to link it to the compiler.",
        ),
    );

    logging::write(
        logging::OutputIn::Stderr,
        &format!(
            "{} {} {}\n\n",
            "•",
            "--print-llvm-host-triples",
            "Displays all available targets of the host for which the llvm backend was prepared.",
        ),
    );

    std::process::exit(1);
}
