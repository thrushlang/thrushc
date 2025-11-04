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

use crate::core::compiler::options::CompilationUnit;

use crate::core::diagnostic::diagnostician::Diagnostician;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::preprocessor::context::PreprocessorContext;
use crate::frontend::preprocessor::module::Module;

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
    pub fn generate_modules(&mut self) -> Result<Vec<Module<'preprocessor>>, ()> {
        let mut ctx: PreprocessorContext =
            PreprocessorContext::new(self.tokens, Diagnostician::new(self.file));

        let mut modules: Vec<Module> = Vec::with_capacity(100);

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
