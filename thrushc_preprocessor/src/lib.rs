#![allow(clippy::result_unit_err)]

use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_token::Token;
use thrushc_token_type::TokenType;

use crate::{context::PreprocessorContext, module::Module};

use ahash::AHashSet as HashSet;

mod context;
mod modparsing;
pub mod module;
mod moduletable;
mod parser;
mod preparsing;
mod signatures;

#[derive(Debug)]
pub struct Preprocessor {
    modules: Vec<Module>,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            modules: Vec::with_capacity(u8::MAX as usize),
        }
    }
}

impl<'preprocessor> Preprocessor {
    pub fn generate_modules(
        &mut self,
        tokens: &'preprocessor [Token],
        options: &'preprocessor CompilerOptions,
        file: &CompilationUnit,
    ) -> Result<&[Module], ()> {
        let file_path: std::path::PathBuf = file.get_path().to_path_buf();

        let mut visited: HashSet<std::path::PathBuf> = HashSet::with_capacity(u8::MAX as usize);
        visited.insert(file_path);

        let mut context: PreprocessorContext<'_> =
            PreprocessorContext::new(tokens, options, file, visited);

        while !context.is_eof() {
            if context.check(TokenType::Import) {
                if let Ok(Some(module)) = preparsing::import::parse_import(&mut context) {
                    self.modules.push(module);
                }
            }

            let _ = context.only_advance();
        }

        context.check_status()?;

        Ok(self.modules.as_slice())
    }
}
