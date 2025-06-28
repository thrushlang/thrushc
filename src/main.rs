mod backend;
mod core;
mod frontend;

use {
    crate::core::{
        compiler::thrushc::TheThrushCompiler,
        console::{cli::CLI, logging},
    },
    colored::{Colorize, control},
    lazy_static::lazy_static,
    std::{env, process, time::Instant},
};

fn main() {
    #[cfg(target_os = "windows")]
    {
        control::set_virtual_terminal(true);
    }

    control::set_override(true);

    let cli: CLI = CLI::parse(env::args().collect());

    let start_time: Instant = Instant::now();

    let compile_time: (u128, u128) =
        TheThrushCompiler::new(cli.get_options().get_files(), cli.get_options()).compile();

    let thrushc_time: u128 = compile_time.0;
    let linking_time: u128 = compile_time.1;

    logging::write(
        logging::OutputIn::Stdout,
        &format!(
            "\n{}\n{}\n\n{}\n{}\n{}\n",
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
            "Compile time report".custom_color((141, 141, 142)).bold(),
            format_args!("Thrush Compiler: {}ms", thrushc_time),
            format_args!("Linking: {}ms", linking_time),
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
