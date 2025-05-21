mod backend;
mod frontend;
mod middle;
mod standard;

use {
    colored::{Colorize, control},
    frontend::thrushc::TheThrushCompiler,
    inkwell::targets::{InitializationConfig, Target},
    lazy_static::lazy_static,
    standard::{cli::CommandLine, logging},
    std::{env, process, time::Instant},
};

fn main() {
    if cfg!(target_os = "windows") {
        control::set_override(true);
    }

    let cli: CommandLine = CommandLine::parse(env::args().collect());

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

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "\n{}\n{}\n\n{}\n{}\n{}\n",
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
            "Compile time report".custom_color((141, 141, 142)).bold(),
            format_args!("Thrush Compiler: {}ms", compile_time.0.to_string().bold()),
            format_args!("Linker: {}ms", compile_time.1.to_string().bold()),
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
