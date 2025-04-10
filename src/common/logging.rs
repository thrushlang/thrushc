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
pub enum LogType {
    Error,
    Panic,
}

impl LogType {
    pub fn to_styled(&self) -> ColoredString {
        match self {
            LogType::Error => "ERROR".bright_red().underline().bold(),
            LogType::Panic => "PANIC".bright_red().underline().bold(),
        }
    }

    #[inline(always)]
    pub const fn is_err(&self) -> bool {
        matches!(self, LogType::Error | LogType::Panic)
    }
}

#[inline]
pub fn log(ltype: LogType, msg: &str) {
    if ltype.is_err() {
        io::stderr()
            .write_all(format!("  {} {}\n  ", ltype.to_styled(), msg.bold()).as_bytes())
            .unwrap();

        if ltype == LogType::Error {
            return;
        } else {
            process::exit(1);
        };
    }

    io::stdout()
        .write_all(format!("  {} {}", ltype.to_styled(), msg.bold()).as_bytes())
        .unwrap();
}

#[inline]
pub fn write(output_in: OutputIn, bytes: &[u8]) {
    match output_in {
        OutputIn::Stdout => io::stdout().write_all(bytes).unwrap_or(()),
        OutputIn::Stderr => io::stderr().write_all(bytes).unwrap_or(()),
    };
}
