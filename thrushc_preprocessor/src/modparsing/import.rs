use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_lexer::Lexer;
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;

use crate::{module::Module, parser::ModuleParser};

pub fn parse_import<'module_parser>(parser: &mut ModuleParser<'module_parser>) -> Result<(), ()> {
    let _ = parser.consume(TokenType::Import)?;

    let module_path_tk: &Token = parser.consume(TokenType::Str)?;
    let module_path_span: Span = module_path_tk.get_span();

    let mut module_path: std::path::PathBuf = std::path::PathBuf::from(module_path_tk.get_lexeme());

    let _ = parser.consume(TokenType::SemiColon)?;

    let _ = module_path.canonicalize().map(|path| module_path = path);

    if !module_path.exists() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "The path does not exist. Make sure it is a valid path.".into(),
            None,
            module_path_span,
        ));

        return Err(());
    }

    if !module_path.is_file() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "The path does not point to a file. Make sure it is a valid path to file.".into(),
            None,
            module_path_span,
        ));

        return Err(());
    }

    if module_path.file_stem().is_none() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "An name was expected in the path. Check that it points to a file with a valid the name.".into(),
            None,
            module_path_span,
        ));

        return Err(());
    }

    if module_path.extension().is_none() {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "An extension was expected in the path. Check that it points to a file with a valid the extension.".into(),
            None,
            module_path_span,
        ));

        return Err(());
    }

    if !module_path.extension().is_some_and(|ext| {
        thrushc_constants::COMPILER_OWN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap_or("unknown"))
    }) {
        parser.add_error(CompilationIssue::Error(
            CompilationIssueCode::E0035,
            "It was expected that it would target files with a '.thrush' or '.🐦' extension. Make sure they are valid thrush files.".into(),
            None,
            module_path_span,
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

    let content: String = thrushc_reader::get_file_source_code(&module_path);
    let file: CompilationUnit = CompilationUnit::new(name, module_path, content, base_name.clone());

    let tokens: Vec<Token> = Lexer::lex_for_preprocessor(&file, options)?;
    let subparser: ModuleParser = ModuleParser::new(base_name, tokens, options, &file);

    let submodule: Module<'module_parser> = match subparser.parse() {
        Ok(module) => module,
        Err(errors) => {
            parser.merge_errors(errors);
            return Err(());
        }
    };

    parser.get_mut_module().add_submodule(submodule);

    Ok(())
}
