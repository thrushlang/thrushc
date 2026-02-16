mod cli;
mod console;
mod help;

use thrustc_heap_allocator::ThrustCompilerHeapAllocator;

#[global_allocator]
static GLOBAL: ThrustCompilerHeapAllocator = ThrustCompilerHeapAllocator;

fn main() -> ! {
    use thrustc_core::ThrustCompiler;

    console::set_up_basic();

    let cli: cli::CommandLine = cli::CommandLine::parse(std::env::args().collect());

    console::ansi(cli.get_options());

    let start_time: std::time::Instant = std::time::Instant::now();

    let comptime: (u128, u128) =
        ThrustCompiler::new(cli.get_options().get_files(), cli.get_options()).compile();

    console::report_comptime(start_time, comptime)
}
