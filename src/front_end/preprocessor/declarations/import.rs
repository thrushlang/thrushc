use std::path::PathBuf;

use crate::core::compiler;
use crate::core::compiler::options::CompilationUnit;
use crate::core::diagnostic::span::Span;

use crate::front_end::lexer::Lexer;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::preprocessor::errors::PreprocessorIssue;
use crate::front_end::preprocessor::module::Module;
use crate::front_end::preprocessor::parser::ModuleParser;
use crate::front_end::types::lexer::types::Tokens;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

pub fn build_import(ctx: &mut ModuleParser, module: &mut Module) -> Result<(), ()> {
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
                "Unable to extract filename with extension.".into(),
                span,
            ));

            return Err(());
        }
    };

    let base_name: String = match path.file_stem() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            ctx.add_error(PreprocessorIssue::new(
                unit.get_path().to_path_buf(),
                "Import error".into(),
                "Unable to extract filename.".into(),
                span,
            ));

            return Err(());
        }
    };

    let file: CompilationUnit = CompilationUnit::new(name, path, content, base_name);

    let tokens: Tokens = Lexer::lex(&file, ctx.get_options());

    let mut parser: ModuleParser = ModuleParser::new(tokens, ctx.get_options());

    let other_module: Module = match parser.parse(file) {
        Ok(module) => module,
        Err(errors) => {
            ctx.merge_errors(errors);
            return Err(());
        }
    };

    module.add_submodule(other_module);

    Ok(())
}
