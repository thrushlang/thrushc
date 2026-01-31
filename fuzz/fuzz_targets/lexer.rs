#![no_main]

use libfuzzer_sys::fuzz_target;
use thrushc_lexer::Lexer;
use thrushc_options::{CompilationUnit, CompilerOptions};

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let options = CompilerOptions::new();

        let unit = CompilationUnit::new(
            "lexer.fuzz".into(),
            std::path::PathBuf::from(file!()),
            source.to_string(),
            "lexer".into(),
        );

        let tokens = Lexer::lex(&unit, &options);
    }
});
