pub mod cli;
pub mod commands;
pub mod logging;
pub mod position;
pub mod utils;

use colored::Colorize;

use std::{process, time::Instant};

use crate::core::compiler::options::CompilerOptions;

#[inline]
pub fn set_up_basic() {
    colored::control::set_override(false);
}

#[inline]
pub fn ansi(options: &CompilerOptions) {
    if options.need_ansi_colors() {
        #[cfg(target_os = "windows")]
        {
            colored::control::set_virtual_terminal(true);
            colored::control::set_override(true);
        }

        #[cfg(target_os = "linux")]
        {
            colored::control::set_override(true);
        }
    }
}

#[inline]
pub fn report_comptime(start_time: Instant, comptime: (u128, u128)) -> ! {
    let thrushc_time: u128 = comptime.0;
    let linking_time: u128 = comptime.1;

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
