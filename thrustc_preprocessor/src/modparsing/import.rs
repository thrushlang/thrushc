use std::path::PathBuf;

use thrustc_errors::{CompilationIssue, CompilationIssueCode};
use thrustc_lexer::Lexer;
use thrustc_options::{CompilationUnit, CompilerOptions};
use thrustc_span::Span;
use thrustc_token::{Token, traits::TokenExtensions};
use thrustc_token_type::TokenType;

use crate::{module::Module, parser::ModuleParser};

pub fn parse_import<'module_parser>(parser: &mut ModuleParser<'module_parser>) -> Result<(), ()> {
    parser.consume(TokenType::Import)?;

    let module_path_tk: &Token =
        parser.consume_these(&[TokenType::CString, TokenType::CNString])?;
    let span: Span = module_path_tk.get_span();

    let mut module_path: PathBuf = PathBuf::from(module_path_tk.get_lexeme());

    parser.consume(TokenType::SemiColon)?;

    if let Ok(canocalized) = module_path.canonicalize() {
        module_path = canocalized;
    }

    if parser.has_visited(&module_path) {
        parser.add_warning(CompilationIssue::Warning(
            CompilationIssueCode::W0018,
            "A circular import was founded here. Omitting it by default. The recomendation is to remove it."
                .into(),
            span,
        ));

        return Ok(());
    } else {
        parser.mark_visited(module_path.clone());
    }

    if !module_path.exists() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "The path does not exist. Make sure it is a valid path.".into(),
            None,
            span,
        ));

        return Err(());
    }

    if !module_path.is_file() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "The path does not point to a file. Make sure it is a valid path to file.".into(),
            None,
            span,
        ));

        return Err(());
    }

    if module_path.file_stem().is_none() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "An name was expected in the path. Check that it points to a file with a valid the name.".into(),
            None,
            span,
        ));

        return Err(());
    }

    if module_path.extension().is_none() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "An extension was expected in the path. Check that it points to a file with a valid the extension.".into(),
            None,
            span,
        ));

        return Err(());
    }

    if !module_path.extension().is_some_and(|ext| {
        thrustc_constants::COMPILER_OWN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap_or("unknown"))
    }) {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "It was expected that it would target files with a '.thrust' or '.🐦' extension. Make sure they are valid thrust files.".into(),
            None,
            span,
        ));

        return Err(());
    }

    let name: String = match module_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            return Err(());
        }
    };

    let base_name: String = match module_path.file_stem() {
        Some(base_name) => base_name.to_string_lossy().to_string(),
        None => {
            return Err(());
        }
    };

    let options: &CompilerOptions = parser.get_options();

    let content: String = thrustc_reader::get_file_source_code(&module_path);
    let file: CompilationUnit = CompilationUnit::new(name, module_path, content, base_name.clone());

    let tokens: Vec<Token> = Lexer::lex_for_preprocessor(&file, options)?;
    let subparser: ModuleParser = ModuleParser::new(
        base_name,
        tokens,
        options,
        &file,
        parser.get_global_visited_modules(),
    );

    let submodule: Module = subparser.parse()?;

    parser.get_mut_module().add_submodule(submodule);

    Ok(())
}
