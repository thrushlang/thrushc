use std::path::PathBuf;

use crate::{
    core::compiler::{self, options::CompilationUnit},
    frontend::{
        lexer::{Lexer, span::Span, token::Token, tokentype::TokenType},
        preprocessor::{errors::PreprocessorIssue, module::Module, parser::ModuleParser},
        types::parser::stmts::traits::TokenExtensions,
    },
};

pub fn build_import<'preprocessor>(
    ctx: &mut ModuleParser<'preprocessor>,
    module: &mut Module<'preprocessor>,
) -> Result<(), ()> {
    let unit: &CompilationUnit = module.get_unit();

    ctx.consume(TokenType::Import)?;

    let path_tk: &Token = ctx.consume(TokenType::Str)?;
    let span: Span = path_tk.get_span();

    let raw_path: String = path_tk.get_lexeme().to_string();

    ctx.consume(TokenType::SemiColon)?;

    let path: PathBuf = PathBuf::from(raw_path);

    if !path.exists() {
        ctx.add_error(PreprocessorIssue::new(
            unit.get_path().to_path_buf(),
            "Import error".into(),
            "Cannot resolve module, specified path does not exist in filesystem.".into(),
            span,
        ));

        return Err(());
    }

    if !path.is_file() {
        ctx.add_error(PreprocessorIssue::new(
            unit.get_path().to_path_buf(),
            "Import error".into(),
            "Invalid module target, path resolves to directory, expected file.".into(),
            span,
        ));

        return Err(());
    }

    if path.extension().unwrap_or_default() != "thrush"
        && path.extension().unwrap_or_default() != "🐦"
    {
        ctx.add_error(PreprocessorIssue::new(
            unit.get_path().to_path_buf(),
            "Import error".into(),
            "Expected '.thrush' or '.🐦' module.".into(),
            span,
        ));

        return Err(());
    }

    let content: String = compiler::reader::get_file_source_code(&path);

    let name: String = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            ctx.add_error(PreprocessorIssue::new(
                unit.get_path().to_path_buf(),
                "Import error".into(),
                "Unable to extract filename component.".into(),
                span,
            ));

            return Err(());
        }
    };

    let file: CompilationUnit = CompilationUnit::new(name, path, content);

    let tokens: Vec<Token> = match Lexer::lex(&file) {
        Ok(tokens) => tokens,
        Err(_) => {
            ctx.add_error(PreprocessorIssue::new(
                unit.get_path().to_path_buf(),
                "Import error".into(),
                "Imported module contains invalid syntax.".into(),
                span,
            ));

            return Err(());
        }
    };

    let mut parser: ModuleParser = ModuleParser::new(tokens);

    let other_module: Module = match parser.parse(file) {
        Ok(module) => module,
        Err(errors) => {
            ctx.merge_errors(errors);
            return Err(());
        }
    };

    module.merge(other_module);

    Ok(())
}
