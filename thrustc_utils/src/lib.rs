use std::path::{Path, PathBuf};

pub fn find_executable<'a>(root: &'a Path, base_name: &'a str) -> Option<PathBuf> {
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            entry.path().is_file() && !entry.path_is_symlink() && entry.path().file_stem().is_some()
        })
    {
        let entry_path: &Path = entry.path();

        if let Some(entry_base_name) = entry_path.file_stem() {
            if entry_base_name == base_name {
                return Some(entry_path.to_path_buf());
            }
        }
    }

    None
}
