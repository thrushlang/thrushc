use crate::constants;
use crate::help;
use crate::logging;
use crate::logging::LoggingType;
use crate::options::BuildOptions;
use crate::targets;
use crate::utils;

use std::process;

#[derive(Debug)]
pub struct CommandLine {
    options: BuildOptions,
    args: Vec<String>,
    current: usize,
}

#[derive(Debug)]
pub struct ParsedArg {
    key: String,
    value: Option<String>,
}

impl ParsedArg {
    fn new(arg: &str) -> Self {
        if let Some(eq_pos) = arg.find('=') {
            let (key, value) = arg.split_at(eq_pos);

            return Self {
                key: key.to_string(),
                value: Some(value[1..].to_string()),
            };
        }

        if let Some(eq_pos) = arg.find(':') {
            let (key, value) = arg.split_at(eq_pos);

            return Self {
                key: key.to_string(),
                value: Some(value[1..].to_string()),
            };
        }

        Self {
            key: arg.to_string(),
            value: None,
        }
    }
}

impl CommandLine {
    pub fn parse(mut args: Vec<String>) -> CommandLine {
        let processed_args: Vec<String> = Self::preprocess_args(&mut args);

        let mut command_line: CommandLine = Self {
            options: BuildOptions::new(),
            args: processed_args,
            current: 0,
        };

        command_line.build();

        command_line
    }
}

impl CommandLine {
    fn build(&mut self) {
        self.check_requirements();

        while !self.is_eof() {
            let argument: String = self.args[self.current].clone();
            self.analyze(argument);
        }

        self.validate();
    }

    fn validate(&self) {
        if let Err(error) = self.options.verify() {
            self.report_error(&error);
        }
    }

    fn check_requirements(&self) {
        if !utils::tar_is_available() {
            logging::log(LoggingType::Panic, "tar is not installed.\n");
            return;
        }

        logging::log(LoggingType::Log, "Requirements are ok.\n\n");
    }
}

impl CommandLine {
    fn analyze(&mut self, argument: String) {
        let arg: &str = argument.as_str();

        match arg {
            "-h" | "--help" | "help" => {
                self.advance();
                help::show_help();
            }

            "-v" | "--version" | "version" => {
                self.advance();
                logging::write(
                    logging::OutputIn::Stdout,
                    constants::COMPILER_BUILDER_VERSION,
                );
                process::exit(0);
            }

            "--llvm-host-triple" => {
                self.advance();

                let triple: String = self.peek().to_string();

                if let Err(error) = self.get_mut_options().set_llvm_triple(triple) {
                    self.report_error(&error)
                }

                self.advance();
            }

            "--print-llvm-host-triples" => {
                self.advance();

                targets::LLVM_HOSTS_TARGETS_AVAILIABLE
                    .iter()
                    .for_each(|target| {
                        logging::write(logging::OutputIn::Stdout, target);
                    });

                process::exit(0);
            }

            _ => {
                help::show_help();
            }
        }
    }
}

impl CommandLine {
    #[inline]
    fn peek(&self) -> &str {
        if self.is_eof() {
            self.report_error("Expected value after flag.");
        }

        &self.args[self.current]
    }

    #[inline]
    fn advance(&mut self) {
        if self.is_eof() {
            self.report_error("Expected value after flag.");
        }

        self.current += 1;
    }

    #[inline]
    fn report_error(&self, msg: &str) -> ! {
        logging::log(LoggingType::Error, msg);
        process::exit(1)
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.current >= self.args.len()
    }
}

impl CommandLine {
    fn preprocess_args(args: &mut Vec<String>) -> Vec<String> {
        let mut processed: Vec<String> = Vec::with_capacity(args.len() * 2);

        if !args.is_empty() {
            args.remove(0);
        }

        args.iter().for_each(|arg| {
            let parsed: ParsedArg = ParsedArg::new(arg);

            processed.push(parsed.key);

            if let Some(value) = parsed.value {
                processed.push(value);
            }
        });

        processed
    }
}

impl CommandLine {
    #[inline]
    pub fn get_options(&self) -> &BuildOptions {
        &self.options
    }

    #[inline]
    pub fn get_mut_options(&mut self) -> &mut BuildOptions {
        &mut self.options
    }
}
