use crate::builder::CompilerBuilderDependencies;
use crate::cli::CommandLine;

mod builder;
mod cli;
mod constants;
mod help;
mod logging;
mod options;
mod targets;
mod utils;

fn main() {
    unsafe { std::env::set_var("CARGO_TERM_VERBOSE", "true") };

    let cli: CommandLine = CommandLine::parse(std::env::args().collect());

    CompilerBuilderDependencies::new(cli.get_options()).install();
}
