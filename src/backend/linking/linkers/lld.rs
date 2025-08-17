use std::process;

use colored::Colorize;
use lld::{flavor::LLDFlavor, result::LLDResult};

use crate::core::{
    compiler::{backends::linkers::LinkerConfiguration, options::CompilerOptions},
    console::logging,
};

#[derive(Debug)]
pub struct LLVMLinker<'lld> {
    options: &'lld CompilerOptions,
}

impl<'lld> LLVMLinker<'lld> {
    #[inline]
    pub fn new(options: &'lld CompilerOptions) -> Self {
        Self { options }
    }
}

impl<'lld> LLVMLinker<'lld> {
    pub fn link(&self) {
        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "\r{} {} {}\n",
                "Linking".bold(),
                "LLD".custom_color((141, 141, 142)).bold(),
                "RUNNING".bright_green().bold()
            ),
        );

        if let LinkerConfiguration::LLVMLinker(flavor) = self.options.get_linker_mode().get_config()
        {
            let args: &[String] = self.options.get_linker_mode().get_args();
            let target: LLDFlavor = flavor.to_lld_pure_flavor();

            let lld_result: LLDResult = lld::link(target, args);

            if let Err(error) = lld_result.ok() {
                logging::write(
                    logging::OutputIn::Stderr,
                    &format!(
                        "\r{} {} {}\n",
                        "LLD".custom_color((141, 141, 142)).bold(),
                        "FAILED".bright_red().bold(),
                        error,
                    ),
                );

                logging::write(
                    logging::OutputIn::Stderr,
                    &format!(
                        "\r{} {} {}\n",
                        "Linking".bold(),
                        "LLD".custom_color((141, 141, 142)).bold(),
                        "FAILED".bright_red().bold()
                    ),
                );

                process::exit(1);
            }
        }

        logging::write(
            logging::OutputIn::Stdout,
            &format!(
                "\r{} {} {}\n",
                "Linking".bold(),
                "LLD".custom_color((141, 141, 142)).bold(),
                "FINISHED".bright_green().bold()
            ),
        );

        process::exit(0);
    }
}
