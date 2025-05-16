use std::process::Command;

use crate::standard::logging;

pub fn handle_command(command: &mut Command) {
    if let Ok(child) = command.output() {
        if !child.stderr.is_empty() {
            logging::log(
                logging::LoggingType::Error,
                String::from_utf8_lossy(&child.stderr).trim_end(),
            );
        }

        if !child.stdout.is_empty() {
            logging::log(
                logging::LoggingType::Warning,
                String::from_utf8_lossy(&child.stdout).trim_end(),
            );
        }
    }
}
