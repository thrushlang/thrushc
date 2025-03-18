use {
    super::{
        backend::compiler::misc::ThrushFile,
        error::ThrushError,
        logging::{self, LogType},
    },
    colored::Colorize,
    std::{fs, path::PathBuf},
};

#[derive(Debug)]
pub struct Diagnostic {
    path: PathBuf,
    contain: String,
}

#[derive(Debug)]
struct CodePosition {
    line: usize,
    start: usize,
    end: usize,
}

impl Diagnostic {
    pub fn new(thrushfile: &ThrushFile) -> Self {
        let contain: String = fs::read_to_string(&thrushfile.path).unwrap_or_else(|_| {
            logging::log(
                LogType::Panic,
                &format!(
                    "Unable to read `{}` file for build a diagnostic.",
                    thrushfile.path.display()
                ),
            );

            unreachable!()
        });

        Self {
            path: thrushfile.path.clone(),
            contain,
        }
    }

    pub fn report(&mut self, error: &ThrushError, logtype: LogType) {
        let ThrushError::Error(title, help, line, span) = error;
        self.print_spanned_report(title, help, *line, span.as_ref(), logtype);
    }

    fn print_spanned_report(
        &mut self,
        title: &str,
        help: &str,
        line: usize,
        span: Option<&(usize, usize)>,
        logtype: LogType,
    ) {
        self.report_header(line, title, logtype);

        if let Some((start_span, end_span)) = span {
            if let Some(code_position) =
                self.find_line_and_range(&self.contain, *start_span, *end_span)
            {
                if let Some((line_text, arrow_line, line)) =
                    self.generate_diagnostic(&self.contain, code_position)
                {
                    println!(
                        "   |\n{}\n{}",
                        format_args!("{}  |  {}", line, &line_text),
                        "   |  ".to_string() + &arrow_line,
                    );
                } else {
                    return self.print_not_spanned_report(help, line);
                }

                self.print_helper(help);

                return;
            }

            return self.print_not_spanned_report(help, line);
        }

        self.print_not_spanned_report(help, line);
    }

    fn print_not_spanned_report(&mut self, help: &str, line: usize) {
        let lines: Vec<&str> = self.contain.lines().collect();

        let line_text: String = "|  ".to_string() + lines[line - 1].trim();
        let arrow_line: String = "|  ".to_string() + &"─".bright_red().repeat(line_text.len());

        logging::write(
            logging::OutputIn::Stderr,
            format!("|\n{}\n{}\n", line_text, arrow_line).as_bytes(),
        );

        self.print_helper(help);
    }

    fn print_helper(&self, help: &str) {
        logging::write(
            logging::OutputIn::Stderr,
            format!(
                "\n{}{} {}\n\n",
                "Help".bold().bright_green(),
                ":".bold(),
                help.bold()
            )
            .as_bytes(),
        );
    }

    fn report_header(&mut self, line: usize, title: &str, logtype: LogType) {
        logging::write(
            logging::OutputIn::Stderr,
            format!(
                "{} at line {}\n",
                format_args!("{}", &self.path.to_string_lossy().bold().bright_red()),
                (line - 1).to_string().bold().bright_red()
            )
            .as_bytes(),
        );

        logging::write(
            logging::OutputIn::Stderr,
            format!("\n{} {}\n\n", logtype.to_styled(), title).as_bytes(),
        );
    }

    fn find_line_and_range(&self, text: &str, start: usize, end: usize) -> Option<CodePosition> {
        let mut line_start: usize = 0;
        let mut line_num: usize = 1;

        for (i, c) in text.char_indices() {
            if i >= start {
                break;
            }
            if c == '\n' {
                line_start = i + 1;
                line_num += 1;
            }
        }

        if start >= text.len() || end > text.len() || start > end {
            return None;
        }

        Some(CodePosition {
            line: line_num,
            start: start - line_start,
            end: end - line_start,
        })
    }

    fn generate_diagnostic(
        &self,
        text: &str,
        position: CodePosition,
    ) -> Option<(String, String, usize)> {
        let lines: Vec<&str> = text.lines().collect();

        if position.line > lines.len() {
            return None;
        }

        let line_position: usize = position.line - 1;

        let line_text: String = lines[line_position].to_string();
        let mut arrow_line: String = " ".repeat(line_text.len());

        arrow_line.replace_range(position.start..position.end, &"—".bright_red());

        Some((line_text, arrow_line, line_position))
    }
}
