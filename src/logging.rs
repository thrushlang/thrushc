use {
    std::{
        io::{self, Write},
        process,
    },
    stylic::{style, Styled, Stylize},
};

#[derive(PartialEq, Clone, Copy)]
pub enum LogType {
    INFO,
    WARN,
    ERROR,
    PANIC,
}

impl LogType {
    pub fn to_styled(self) -> Styled<&'static str> {
        match self {
            LogType::INFO => style("INFO").bold().bright_green(),
            LogType::WARN => style("WARN").bold().bright_yellow(),
            LogType::ERROR => style("ERROR").bold().bright_red(),
            LogType::PANIC => style("(!) PANIC").bold().bright_red().blink(),
        }
    }
}

#[inline]
pub fn log(ltype: LogType, msg: &str) {
    if ltype == LogType::ERROR || ltype == LogType::PANIC {
        io::stderr()
            .write_all(format!(">  {} {}\n  ", ltype.to_styled(), style(msg).bold()).as_bytes())
            .unwrap();

        if ltype == LogType::ERROR {
            return;
        } else {
            process::exit(1);
        };
    }

    io::stdout()
        .write_all(format!("  {} {}", ltype.to_styled(), style(msg).bold()).as_bytes())
        .unwrap();
}
