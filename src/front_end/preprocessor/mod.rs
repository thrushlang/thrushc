pub mod attributes;
pub mod context;
pub mod declarations;
pub mod errors;
pub mod import;
pub mod module;
pub mod parser;
pub mod signatures;
pub mod table;
pub mod typegen;
pub mod types;

use crate::core::compiler::options::{CompilationUnit, CompilerOptions};

use crate::core::diagnostic::diagnostician::Diagnostician;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::preprocessor::context::PreprocessorContext;
use crate::front_end::preprocessor::module::Module;

#[derive(Debug)]
pub struct Preprocessor<'preprocessor> {
    tokens: &'preprocessor [Token],
    file: &'preprocessor CompilationUnit,
}

impl<'preprocessor> Preprocessor<'preprocessor> {
    #[inline]
    pub fn new(tokens: &'preprocessor [Token], file: &'preprocessor CompilationUnit) -> Self {
        Self { tokens, file }
    }
}

impl<'preprocessor> Preprocessor<'preprocessor> {
    pub fn generate_modules(
        &mut self,
        options: &'preprocessor CompilerOptions,
    ) -> Result<Vec<Module>, ()> {
        let mut ctx: PreprocessorContext =
            PreprocessorContext::new(self.tokens, Diagnostician::new(self.file, options), options);

        let mut modules: Vec<Module> = Vec::with_capacity(100_000);

        while !ctx.is_eof() {
            if ctx.check(TokenType::Import) {
                if let Ok(module) = import::build_import(&mut ctx) {
                    modules.push(module);
                }
            }

            let _ = ctx.only_advance();
        }

        ctx.verify()?;

        Ok(modules)
    }
}
