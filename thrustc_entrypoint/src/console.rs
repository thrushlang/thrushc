/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

use colored::Colorize;
use thrustc_options::CompilerOptions;

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
pub fn report_comptime(
    options: &CompilerOptions,
    start_time: std::time::Instant,
    compile_time: (
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
    ),
) -> ! {
    let thrustc_time_ms: f64 = compile_time.0.as_millis_f64();
    let frontend_time_ms: f64 = compile_time.1.as_millis_f64();
    let backend_time_ms: f64 = compile_time.2.as_millis_f64();
    let linking_time_ms: f64 = compile_time.3.as_millis_f64();

    let backend_identifier: &str = if options.llvm() { "LLVM" } else { "GCC" };

    thrustc_logging::write(
        thrustc_logging::OutputIn::Stdout,
        &format!(
            "\n{}\n{}\n\n{}\n{}\n{}\n{}\n{}\n",
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
            "Compile time report".custom_color((141, 141, 142)).bold(),
            format_args!("Thrust Compiler: {}ms", thrustc_time_ms),
            format_args!("Thrust Compiler - Frontend: {}ms", frontend_time_ms),
            format_args!(
                "Thrust Compiler - Backend ({}): {}ms",
                backend_identifier, backend_time_ms
            ),
            format_args!("Linking: {}ms", linking_time_ms),
            "─────────────────────────────────────────"
                .custom_color((141, 141, 142))
                .bold(),
        ),
    );

    thrustc_logging::write(
        thrustc_logging::OutputIn::Stdout,
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

    std::process::exit(thrustc_constants::SUCCESFUL_CODE);
}
