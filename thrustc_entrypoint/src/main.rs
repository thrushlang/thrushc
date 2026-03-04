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
    let options: &thrustc_options::CompilerOptions = cli.get_options();

    console::ansi(options);

    let start_time: std::time::Instant = std::time::Instant::now();

    let comptime: (u128, u128, u128, u128) =
        ThrustCompiler::new(options.get_files(), options).compile();

    console::report_comptime(options, start_time, comptime)
}
