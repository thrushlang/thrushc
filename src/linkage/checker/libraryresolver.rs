use std::path::PathBuf;

#[derive(Debug)]
pub struct LibraryResolver {
    search_paths: Vec<PathBuf>,
}

impl LibraryResolver {
    pub fn new() -> Self {
        let mut search_paths: Vec<PathBuf> = Vec::with_capacity(10);

        if std::env::consts::OS == "linux" {
            search_paths.extend([
                PathBuf::from("/usr/lib/x86_64-linux-gnu"),
                PathBuf::from("/usr/lib"),
                PathBuf::from("/usr/local/lib"),
                PathBuf::from("/lib/x86_64-linux-gnu"),
                PathBuf::from("/lib"),
                PathBuf::from("/lib64"),
                PathBuf::from("/usr/lib64"),
            ]);
        }

        Self { search_paths }
    }
}

impl LibraryResolver {
    pub fn resolve_library(&self, name: &str) -> Option<PathBuf> {
        let extensions: &[&str] = LibraryResolver::get_library_system_extensions();

        for search_path in &self.search_paths {
            for ext in extensions {
                let lib_filename: String = format!("lib{}.{}", name, ext);
                let full_path: PathBuf = search_path.join(&lib_filename);

                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }

        None
    }

    pub fn resolve_all(&self, lib_names: &[String]) -> Vec<PathBuf> {
        let mut resolved: Vec<PathBuf> = Vec::with_capacity(10);

        for lib_name in lib_names {
            if let Some(path) = self.resolve_library(lib_name) {
                resolved.push(path)
            }
        }

        resolved
    }
}

impl LibraryResolver {
    fn get_library_system_extensions() -> &'static [&'static str] {
        if std::env::consts::OS == "linux" {
            &["so", "a"]
        } else if std::env::consts::OS == "windows" {
            &["dll", "lib", "a"]
        } else {
            &["so", "a"]
        }
    }
}

impl LibraryResolver {
    #[inline]
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
}
