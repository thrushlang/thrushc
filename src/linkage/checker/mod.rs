use std::path::PathBuf;

use crate::{
    core::{
        compiler::{
            linking::LinkingCompilersConfiguration,
            options::{CompilationUnit, CompilerOptions},
        },
        console::logging::LoggingType,
        diagnostic::diagnostician::Diagnostician,
        errors::standard::CompilationIssue,
    },
    front_end::types::ast::Ast,
    linkage::checker::{
        cliparser::CompilerLinkerParser, file::LinkageCheckerFile, libraryresolver::LibraryResolver,
    },
};

pub mod cliparser;
pub mod dynlib;
pub mod file;
pub mod libraryresolver;
pub mod signatures;
pub mod staticlib;

#[derive(Debug)]
pub struct LinkageChecker<'linkage_checker> {
    files: Vec<LinkageCheckerFile<'linkage_checker>>,
    resolver: LibraryResolver,
    parser: CompilerLinkerParser<'linkage_checker>,

    diagnostician: Diagnostician,
    errors: Vec<CompilationIssue>,

    current: usize,
}

impl<'linkage_checker> LinkageChecker<'linkage_checker> {
    pub fn new(
        files: Vec<LinkageCheckerFile<'linkage_checker>>,
        config: &'linkage_checker LinkingCompilersConfiguration,
        file: &'linkage_checker CompilationUnit,
        options: &CompilerOptions,
    ) -> Self {
        Self {
            files,
            resolver: LibraryResolver::new(),
            parser: CompilerLinkerParser::new(config),

            diagnostician: Diagnostician::new(file, options),
            errors: Vec::with_capacity(100),

            current: 0,
        }
    }
}

impl LinkageChecker<'_> {
    pub fn check(&mut self) -> Result<(), ()> {
        let libraries: Vec<PathBuf> = self.prepare();

        while !self.is_eof() {
            self.advance_file();
        }

        self.verify()?;

        Ok(())
    }
}

impl LinkageChecker<'_> {
    pub fn prepare(&mut self) -> Vec<PathBuf> {
        let (search_paths, lib_names) = self.parser.parse();

        search_paths.iter().for_each(|search_path| {
            self.resolver.add_search_path(PathBuf::from(search_path));
        });

        self.resolver.resolve_all(lib_names)
    }
}

impl<'linkage_checker> LinkageChecker<'linkage_checker> {
    fn advance_file(&mut self) {
        self.current += 1;
    }

    fn peek_file(&self) -> &LinkageCheckerFile<'linkage_checker> {
        &self.files[self.current]
    }
}

impl LinkageChecker<'_> {
    fn is_eof(&mut self) -> bool {
        self.current >= self.files.len()
    }
}

impl LinkageChecker<'_> {
    pub fn verify(&mut self) -> Result<(), ()> {
        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostician
                    .dispatch_diagnostic(error, LoggingType::Error);
            });

            return Err(());
        }

        Ok(())
    }
}

impl LinkageChecker<'_> {
    #[inline]
    pub fn add_error(&mut self, error: CompilationIssue) {
        self.errors.push(error);
    }
}
