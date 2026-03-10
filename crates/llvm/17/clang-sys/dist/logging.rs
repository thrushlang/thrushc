#[derive(Debug, PartialEq)]
pub enum LoggingType {
    Error,
    Panic,
    Debug,
}

#[derive(Debug)]
pub enum OutputIn {
    Stdout,
    Stderr,
}

impl std::fmt::Display for LoggingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggingType::Error => write!(f, "ERROR"),
            LoggingType::Panic => write!(f, "PANIC"),
            LoggingType::Debug => write!(f, "DEBUG"),
        }
    }
}

impl LoggingType {
    #[inline]
    pub fn is_panic(&self) -> bool {
        matches!(self, LoggingType::Panic)
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        matches!(self, LoggingType::Error)
    }
}

#[inline]
pub fn write(output_in: OutputIn, text: &str) {
    match output_in {
        OutputIn::Stdout => {
            let _ = std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes());
        }

        OutputIn::Stderr => {
            let _ = std::io::Write::write_all(&mut std::io::stderr(), text.as_bytes());
        }
    };
}

pub fn log(ltype: LoggingType, msg: &str) {
    if ltype.is_panic() {
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            format!("{} {}", ltype, msg).as_bytes(),
        );

        std::process::exit(1);
    }

    if ltype.is_err() {
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            format!("{} {}", ltype, msg).as_bytes(),
        );

        return;
    }

    let _ = std::io::Write::write_all(
        &mut std::io::stdout(),
        format!("{} {}", ltype, msg).as_bytes(),
    );
}
