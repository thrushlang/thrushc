use {
    super::{backend::compiler::misc::ThrushFile, error::ThrushError, logging::LogType},
    std::{fs, path::PathBuf},
    stylic::{style, Stylize},
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
        let contain: String = fs::read_to_string(&thrushfile.path).unwrap();

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
                if let Some((line_text, arrow_line)) =
                    self.generate_diagnostic(&self.contain, code_position)
                {
                    println!(
                        "|\n{}\n{}",
                        "|   ".to_string() + line_text.trim(),
                        arrow_line.trim_end()
                    );
                } else {
                    return self.print_not_spanned_report(help, line);
                }

                println!(
                    "\n{}{} {}\n",
                    style("Help").bold().bright_green(),
                    style(":").bold(),
                    style(help).bold()
                );

                return;
            }

            return self.print_not_spanned_report(help, line);
        }

        self.print_not_spanned_report(help, line);
    }

    fn print_not_spanned_report(&mut self, help: &str, line: usize) {
        let lines: Vec<&str> = self.contain.lines().collect();

        let line_text: String = "|   ".to_string() + lines[line - 1];
        let arrow_line: String =
            "|   ".to_string() + &style("─").bright_red().to_string().repeat(line_text.len());

        println!("|\n{}\n{}", line_text, arrow_line);

        println!(
            "\n{}{} {}\n",
            style("Help").bold().bright_green(),
            style(":").bold(),
            style(help).bold()
        );
    }

    fn report_header(&mut self, line: usize, title: &str, logtype: LogType) {
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

    fn generate_diagnostic(&self, text: &str, position: CodePosition) -> Option<(String, String)> {
        let lines: Vec<&str> = text.lines().collect();

        if position.line > lines.len() {
            return None;
        }

        let line_text: String = lines[position.line - 1].to_string();
        let mut arrow_line: String = " ".repeat(line_text.len());

        arrow_line.replace_range(
            position.start..position.end,
            &style("—").bright_red().to_string(),
        );

        Some((line_text, arrow_line))
    }
}
