use {
    super::{
        backend::compiler::options::ThrushFile,
        error::{ThrushError, ThrushErrorKind},
        logging::LogType,
    },
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
        let file: File = File::open(&thrushfile.path).unwrap();
        let lines: Vec<String> = BufReader::new(file)
            .lines()
            .map(|line| line.unwrap().to_string())
            .collect();

        Self {
            path: thrushfile.path.clone(),
            buffer: String::new(),
            drawer: String::new(),
            lines,
        }
    }

    pub fn report(&mut self, error: &ThrushError, logtype: LogType, show_only_example: bool) {
        if let ThrushError::Scope(
            ThrushErrorKind::UnreachableVariable | ThrushErrorKind::ObjectNotDefined,
            title,
            help,
            line,
        )
        | ThrushError::Lex(
            ThrushErrorKind::SyntaxError
            | ThrushErrorKind::ParsedNumber
            | ThrushErrorKind::UnreachableNumber
            | ThrushErrorKind::UnknownChar,
            title,
            help,
            line,
        ) = error
        {
            self.print_report(title, help, *line, logtype, None, show_only_example);
        }

        if let ThrushError::Parse(
            ThrushErrorKind::ParsedNumber
            | ThrushErrorKind::UnreachableNumber
            | ThrushErrorKind::SyntaxError
            | ThrushErrorKind::UnreachableVariable
            | ThrushErrorKind::ObjectNotDefined
            | ThrushErrorKind::VariableNotDeclared,
            title,
            help,
            line,
            example,
        ) = error
        {
            self.print_report(
                title,
                help,
                *line,
                logtype,
                (!example.is_empty()).then_some(example),
                show_only_example,
            );
        }
    }

    fn print_report(
        &mut self,
        title: &str,
        help: &str,
        line: usize,
        logtype: LogType,
        example: Option<&String>,
        show_only_example: bool,
    ) {
        self.print_header(line, title, logtype, show_only_example);

        if !show_only_example {
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
        }

        if let Some(example) = example {
            if !show_only_example {
                println!("\n{}\n", style("Example").bold().bright_green());
            } else {
                println!("{}\n", style("Example").bold().bright_green());
            }

            self.buffer.push_str(" > ");

            if !show_only_example {
                self.drawer.push_str(&format!("{} | ", line));
            } else {
                self.drawer.push_str("  | ");
            }

            self.buffer.push_str(&format!("{}\n", example));

            println!("|\n|");

            for _ in 0..example.len() + 5 {
                self.drawer
                    .push_str(style("─").bright_green().to_string().as_str());
            }

            self.buffer.push_str(&self.drawer);

            println!("{}", self.buffer);

            self.drawer.clear();
            self.buffer.clear();
        }

        println!(
            "\n{}{} {}\n",
            style("Help").bold().bright_green(),
            style(":").bold(),
            style(help).bold()
        );
    }

    fn print_header(
        &mut self,
        line: usize,
        title: &str,
        logtype: LogType,
        show_only_example: bool,
    ) {
        if !show_only_example {
            println!(
                "{} - {}",
                format_args!(
                    "{}",
                    style(&self.path.to_string_lossy()).bold().bright_red()
                ),
                line
            );
        } else {
            println!(
                "{}",
                format_args!(
                    "{}",
                    style(&self.path.to_string_lossy()).bold().bright_red()
                )
            );
        }

        println!("\n{} {}\n", logtype.to_styled(), title);
    }
}
