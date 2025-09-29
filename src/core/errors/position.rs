use std::fmt::Display;

use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub enum CompilationPosition {
    Lexer,
    Parser,
    TypeChecker,
    Analyzer,
    Linter,

    LLVMBackend,
}

impl Display for CompilationPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer => write!(f, "{}", "Lexer".bright_blue().bold()),
            Self::Parser => write!(f, "{}", "Parser".red().bold()),
            Self::TypeChecker => write!(f, "{}", "Type Checker".bright_yellow().bold()),
            Self::Analyzer => write!(f, "{}", "Analyzer".bright_blue().bold()),
            Self::Linter => write!(f, "{}", "Linter".bright_magenta().bold()),

            Self::LLVMBackend => write!(f, "{}", "LLVMBackend".bright_red().bold()),
        }
    }
}
