mod backend;
mod core;
mod frontend;

use {
    crate::core::{
        compiler::thrushc::TheThrushCompiler,
        console::{cli::CLI, logging},
    },
    colored::{Colorize, control},
    inkwell::targets::{InitializationConfig, Target},
    lazy_static::lazy_static,
    std::{env, process, time::Instant},
};

fn main() {
    if cfg!(target_os = "windows") {
        control::set_override(true);
    }

    let cli: CLI = CLI::parse(env::args().collect());

    Target::initialize_all(&InitializationConfig::default());

    if !cli.get_options().use_llvm() {
        logging::log(
            logging::LoggingType::Panic,
            "Select a backend infrastructure for example: '-llvm'.",
        );
    }

    let start_time: Instant = Instant::now();

    let compile_time: (u128, u128) =
        TheThrushCompiler::new(cli.get_options().get_files(), cli.get_options()).compile();

    let thrushc_time: u128 = compile_time.0;
    let linker_time: u128 = compile_time.1;

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "\n{}\n{}\n\n{}\n{}\n{}\n",
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
            "Compile time report".custom_color((141, 141, 142)).bold(),
            format_args!("Thrush Compiler: {}ms", thrushc_time.to_string()),
            format_args!("Linking: {}ms", linker_time.to_string()),
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
        ),
    );

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "\r{} {}",
            "Finished".custom_color((141, 141, 142)).bold(),
            format!(
                "{}.{}s",
                start_time.elapsed().as_secs(),
                start_time.elapsed().as_millis()
            )
            .custom_color((141, 141, 142))
            .bold(),
        ),
    );

    process::exit(0);
}
