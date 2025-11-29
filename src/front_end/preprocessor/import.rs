use std::path::PathBuf;

use crate::{
    core::{
        compiler::{self, options::CompilationUnit},
        diagnostic::span::Span,
        errors::standard::CompilationIssue,
    },
    front_end::{
        lexer::{Lexer, token::Token, tokentype::TokenType},
        preprocessor::{context::PreprocessorContext, module::Module, parser::ModuleParser},
        types::parser::stmts::traits::TokenExtensions,
    },
};

pub fn build_import<'preprocessor>(
    ctx: &mut PreprocessorContext<'preprocessor>,
) -> Result<Module, ()> {
    ctx.consume(TokenType::Import)?;

    let path_tk: &Token = ctx.consume(TokenType::Str)?;

    let span: Span = path_tk.get_span();
    let raw_path: &str = path_tk.get_lexeme();

    ctx.consume(TokenType::SemiColon)?;

    let path: PathBuf = PathBuf::from(raw_path);

    if !path.exists() {
        ctx.add_error(CompilationIssue::Error(
            "Import error".into(),
            "Cannot resolve module, specified path does not exist in filesystem.".into(),
            None,
            span,
        ));

        return Err(());
    }

    if !path.is_file() {
        ctx.add_error(CompilationIssue::Error(
            "Import error".into(),
            "Invalid module target, path resolves to directory, expected file.".into(),
            None,
            span,
        ));

        return Err(());
    }

    if path.extension().unwrap_or_default() != "thrush"
        && path.extension().unwrap_or_default() != "🐦"
    {
        ctx.add_error(CompilationIssue::Error(
            "Import error".into(),
            "Expected '.thrush' or '.🐦' module.".into(),
            None,
            span,
        ));

        return Err(());
    }

    let content: String = compiler::reader::get_file_source_code(&path);

    let name: String = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            ctx.add_error(CompilationIssue::Error(
                "Import error".into(),
                "Unable to extract filename component.".into(),
                None,
                span,
            ));

            return Err(());
        }
    };

    let base_name: String = match path.file_stem() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            ctx.add_error(CompilationIssue::Error(
                "Import error".into(),
                "Unable to extract filename.".into(),
                None,
                span,
            ));

            return Err(());
        }
    };

    let file: CompilationUnit = CompilationUnit::new(name, path, content, base_name);

    let tokens: Vec<Token> = match Lexer::lex(&file) {
        Ok(tokens) => tokens,
        Err(_) => {
            ctx.add_error(CompilationIssue::Error(
                "Import error".into(),
                "Imported module contains invalid syntax.".into(),
                None,
                span,
            ));

            return Err(());
        }
    };

    let mut parser: ModuleParser = ModuleParser::new(tokens);

    let module: Module = match parser.parse(file) {
        Ok(module) => module,
        Err(errors) => {
            ctx.merge_module_errors(errors);
            return Err(());
        }
    };

    Ok(module)
}
