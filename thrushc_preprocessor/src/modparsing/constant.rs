use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_lexer::Lexer;
use thrushc_options::{CompilationUnit, CompilerOptions};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;

use crate::{module::Module, parser::ModuleParser};

pub fn parse_constant<'module_parser>(parser: &mut ModuleParser<'module_parser>) -> Result<(), ()> {
    parser.consume(TokenType::Const)?;
    parser.consume(TokenType::Identifier)?;
    parser.consume(TokenType::Colon)?;

    todo!()
}
