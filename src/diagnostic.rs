use {
    super::{backend::compiler::misc::ThrushFile, error::ThrushError, logging::LogType},
    std::{
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
    },
    stylic::{style, Stylize},
};

#[derive(Debug)]
pub struct Diagnostic {
    path: PathBuf,
    buffer: String,
    drawer: String,
    lines: Vec<String>,
}

impl Diagnostic {
    pub fn new(thrushfile: &ThrushFile) -> Self {
        const TYPICAL_BUFFER_CAPACITY: usize = 256;
        const TYPICAL_DRAWER_CAPACITY: usize = 512;

        let file: File = File::open(&thrushfile.path).unwrap();

        const BUFFER_SIZE: usize = 8192;
        const INITIAL_LINES_CAPACITY: usize = 5000;

        let mut lines: Vec<String> = Vec::with_capacity(INITIAL_LINES_CAPACITY);
        let mut reader: BufReader<File> = BufReader::with_capacity(BUFFER_SIZE, file);
        let mut line: String = String::with_capacity(256);

        while reader.read_line(&mut line).unwrap() > 0 {
            lines.push(line.clone());
            line.clear();
        }

        Self {
            path: thrushfile.path.clone(),
            buffer: String::with_capacity(TYPICAL_BUFFER_CAPACITY),
            drawer: String::with_capacity(TYPICAL_DRAWER_CAPACITY),
            lines,
        }
    }

    pub fn report(&mut self, error: &ThrushError, logtype: LogType) {
        let ThrushError::Error(title, help, line) = error;

        self.print_report(title, help, *line, logtype);
    }

    fn print_report(&mut self, title: &str, help: &str, line: usize, logtype: LogType) {
        self.print_header(line, title, logtype);

        let content: &str = if line > self.lines.len() - 1 {
            self.lines.last().unwrap().trim()
        } else {
            self.lines[line - 1].trim()
        };

        self.buffer.push_str(" >> ");
        self.drawer.push_str(&format!("{} | ^ ", line));
        self.buffer.push_str(&format!("{}\n", content));

        println!("|\n|");

        for _ in 0..content.len() + 5 {
            self.drawer
                .push_str(style("─").bright_red().to_string().as_str());
        }

        self.buffer.push_str(&self.drawer);

        println!("{}", self.buffer);

        self.drawer.clear();
        self.buffer.clear();

        println!(
            "\n{}{} {}\n",
            style("Help").bold().bright_green(),
            style(":").bold(),
            style(help).bold()
        );
    }

    fn print_header(&mut self, line: usize, title: &str, logtype: LogType) {
        println!(
            "{} - {}",
            format_args!(
                "{}",
                style(&self.path.to_string_lossy()).bold().bright_red()
            ),
            line
        );

        println!("\n{} {}\n", logtype.to_styled(), title);
    }
}
