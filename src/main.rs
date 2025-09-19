mod backends;
mod core;
mod frontends;
mod middles;

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

    let cli: CLI = CLI::parse(env::args().collect());

    let start_time: Instant = Instant::now();

    let comptime: (u128, u128) =
        ThrushCompiler::new(cli.get_options().get_files(), cli.get_options()).compile();

    console::report_comptime(start_time, comptime)
}
