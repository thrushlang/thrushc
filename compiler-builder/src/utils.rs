use std::process::Command;

pub fn tar_is_available() -> bool {
    Command::new("tar").arg("--version").output().is_ok()
}
