use std::fmt::Display;

use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub enum CompilationPosition {
    Parser,
    TypeChecker,
    Linter,
}

impl Display for CompilationPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linter => write!(f, "{}", "Linter".bright_magenta().bold()),
            Self::Parser => write!(f, "{}", "Parser".red().bold()),
            Self::TypeChecker => write!(f, "{}", "Type Checking".bright_yellow().bold()),
        }
    }
}
