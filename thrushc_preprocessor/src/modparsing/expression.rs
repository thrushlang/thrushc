use thrushc_ast::Ast;
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{modparsing::reinterpret, parser::ModuleParser};

pub fn build_expr<'module_parser>(ctx: &mut ModuleParser) -> Result<Ast<'module_parser>, ()> {
    match ctx.peek().get_type() {
        TokenType::Integer => {
            let tk: &Token = ctx.advance()?;

            let integer: &str = tk.get_lexeme();
            let span: Span = tk.get_span();

            let parsed_integer: (Type, u64) = reinterpret::integer(integer, span)?;

            let integer_type: Type = parsed_integer.0;
            let integer_value: u64 = parsed_integer.1;

            Ok(Ast::new_integer(integer_type, integer_value, false, span))
        }

        _ => Err(()),
    }
}
