use colored::{ColoredString, Colorize};

use std::{
    io::{self, Write},
    process,
};

#[derive(Debug)]
pub enum OutputIn {
    Stdout,
    Stderr,
}

#[derive(Debug, PartialEq)]
pub enum LoggingType {
    Error,
    Panic,
}

impl LoggingType {
    pub fn to_styled(&self) -> ColoredString {
        match self {
            LoggingType::Error => "ERROR".bright_red().bold(),
            LoggingType::Panic => "PANIC".bold().bright_red().underline(),
        }
    }

    pub fn is_err(&self) -> bool {
        matches!(self, LoggingType::Error | LoggingType::Panic)
    }
}

pub fn log(ltype: LoggingType, msg: &str) {
    if ltype.is_err() {
        io::stderr()
            .write_all(format!("  {} {}\n  ", ltype.to_styled(), msg.bold()).as_bytes())
            .unwrap();

        if ltype == LoggingType::Error {
            return;
        } else {
            process::exit(1);
        };
    }

    io::stdout()
        .write_all(format!("  {} {}", ltype.to_styled(), msg.bold()).as_bytes())
        .unwrap();
}

pub fn write(output_in: OutputIn, text: &str) {
    match output_in {
        OutputIn::Stdout => io::stdout().write_all(text.as_bytes()).unwrap_or(()),
        OutputIn::Stderr => io::stderr().write_all(text.as_bytes()).unwrap_or(()),
    };
}
