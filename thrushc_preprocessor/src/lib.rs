#![allow(clippy::result_unit_err)]

use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_token::{Token, tokentype::TokenType};

use crate::{context::PreprocessorContext, module::Module};

mod context;
mod modparsing;
pub mod module;
mod parser;
mod preparsing;
mod signatures;
mod table;

#[derive(Debug)]
pub struct Preprocessor<'preprocessor> {
    modules: Vec<Module<'preprocessor>>,
}

impl<'preprocessor> Preprocessor<'preprocessor> {
    pub fn new() -> Self {
        Self {
            modules: Vec::with_capacity(255),
        }
    }
}

impl<'preprocessor> Preprocessor<'preprocessor> {
    pub fn generate_modules(
        &mut self,
        tokens: &'preprocessor [Token],
        options: &'preprocessor CompilerOptions,
        file: &CompilationUnit,
    ) -> &[Module<'preprocessor>] {
        let mut context: PreprocessorContext<'_> = PreprocessorContext::new(tokens, options, file);

        while !context.is_eof() {
            if context.check(TokenType::Import) {
                if let Ok(module) = preparsing::import::parse_import(&mut context) {
                    self.modules.push(module);
                }
            }

            let _ = context.only_advance();
        }

        self.modules.as_slice()
    }
}
