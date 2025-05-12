use std::process::Command;

use crate::standard::logging;

pub fn handle_command(command: &mut Command) {
    if let Ok(child) = command.output() {
        if !child.status.success() {
            logging::log(
                logging::LoggingType::Error,
                &String::from_utf8_lossy(&child.stderr),
            );
        }
    }
}
