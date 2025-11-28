mod back_end;
mod core;
mod front_end;
mod linkage;
mod middle_end;

use crate::core::compiler::thrushc::ThrushCompiler;
use crate::core::console::{self, cli::CommandLine, logging};

use std::env;
use std::time::Instant;

fn main() -> ! {
    console::set_up_basic();

    let command_line: CommandLine = CommandLine::parse(env::args().collect());

    console::ansi(command_line.get_options());

    let start_time: Instant = Instant::now();

    let comptime: (u128, u128) = ThrushCompiler::new(
        command_line.get_options().get_files(),
        command_line.get_options(),
    )
    .compile();

    console::report_comptime(start_time, comptime)
}
