use thrushc_lexer::Lexer;
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;

use crate::{context::PreprocessorContext, module::Module, parser::ModuleParser};

pub fn parse_import<'preprocessor>(
    parser: &mut PreprocessorContext<'preprocessor>,
) -> Result<Module<'preprocessor>, ()> {
    let _ = parser.consume(TokenType::Import)?;

    let module_path_tk: &Token = parser.consume(TokenType::Str)?;
    let module_path_span: Span = module_path_tk.get_span();

    let mut module_path: std::path::PathBuf = std::path::PathBuf::from(module_path_tk.get_lexeme());

    let _ = parser.consume(TokenType::SemiColon)?;

    let _ = module_path.canonicalize().map(|path| module_path = path);

    if !module_path.exists() {
        return Err(());
    }

    if !module_path.is_file() {
        return Err(());
    }

    if module_path.file_stem().is_none() {
        return Err(());
    }

    if module_path.extension().is_none() {
        return Err(());
    }

    if !module_path.extension().is_some_and(|ext| {
        thrushc_constants::COMPILER_OWN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap_or("unknown"))
    }) {
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

    let submodule: Module<'preprocessor> = match subparser.parse() {
        Ok(submodule) => submodule,
        Err(errors) => {
            parser.merge_errors(errors);
            return Err(());
        }
    };

    Ok(submodule)
}
