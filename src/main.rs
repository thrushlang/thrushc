mod back_end;
mod cbindgen;
mod core;
mod front_end;
mod linkage;
mod middle_end;

fn main() -> ! {
    core::console::set_up_basic();

    let command_line: core::console::cli::CommandLine =
        core::console::cli::CommandLine::parse(std::env::args().collect());

    core::console::ansi(command_line.get_options());

    let start_time: std::time::Instant = std::time::Instant::now();

    let comptime: (u128, u128) = core::compiler::thrushc::ThrushCompiler::new(
        command_line.get_options().get_files(),
        command_line.get_options(),
    )
    .compile();

    core::console::report_comptime(start_time, comptime)
}
