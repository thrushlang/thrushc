use std::{path::Path, process::Command};

#[inline]
pub fn test_as_external_process(path: &Path) -> bool {
    Command::new(path).output().is_ok()
}
