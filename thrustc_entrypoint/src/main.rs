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

#![feature(duration_millis_float)]

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

    let compile_time: (
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
        std::time::Duration,
    ) = ThrustCompiler::new(options.get_files(), options).compile();

    console::report_comptime(options, start_time, compile_time)
}
