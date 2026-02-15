mod cli;
mod console;
mod help;

use thrushc_heap_allocator::ThrushCompilerHeapAllocator;

#[global_allocator]
static GLOBAL: ThrushCompilerHeapAllocator = ThrushCompilerHeapAllocator;

fn main() -> ! {
    use thrushc_core::ThrushCompiler;

    console::set_up_basic();

    let cli: cli::CommandLine = cli::CommandLine::parse(std::env::args().collect());

    console::ansi(cli.get_options());

    let start_time: std::time::Instant = std::time::Instant::now();

    let comptime: (u128, u128) =
        ThrushCompiler::new(cli.get_options().get_files(), cli.get_options()).compile();

    console::report_comptime(start_time, comptime)
}
