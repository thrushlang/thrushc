use colored::Colorize;

use crate::core::diagnostic::diagnostician::Notificator;

impl std::fmt::Display for Notificator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommonHelp => write!(f, "{}", " HELP: ".bright_green().bold()),
            Self::CompilerFrontendBug | Self::CompilerBackendBug => {
                write!(f, "{}", " INFO: ".bright_red().bold())
            }
        }
    }
}
