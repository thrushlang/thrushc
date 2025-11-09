mod back_end;
mod core;
mod front_end;
mod linkage;
mod middle_end;

use {
    crate::core::{
        compiler::thrushc::ThrushCompiler,
        console::{self, cli::CLI, logging},
    },
    lazy_static::lazy_static,
    std::{env, time::Instant},
};

fn main() -> ! {
    console::set_up();

    let command_line: CLI = CLI::parse(env::args().collect());

    let start_time: Instant = Instant::now();

    let comptime: (u128, u128) = ThrushCompiler::new(
        command_line.get_options().get_files(),
        command_line.get_options(),
    )
    .compile();

    console::report_comptime(start_time, comptime)
}
