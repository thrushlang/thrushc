use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expr},
        types::ast::Ast,
        types::parser::{
            stmts::traits::{FoundSymbolEither, FoundSymbolExtension},
            symbols::{
                traits::FunctionExtensions,
                types::{AssemblerFunction, FoundSymbolId, Function},
            },
        },
        typesystem::types::Type,
    },
};

pub fn build_call<'parser>(
    parser_context: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    let function_type: Type = if object.is_function_asm() {
        let asm_function_id: &str = object.expected_asm_function(span)?;
        let asm_function: AssemblerFunction = parser_context
            .get_symbols()
            .get_asm_function_by_id(span, asm_function_id)?;

        asm_function.get_type()
    } else {
        let function_id: &str = object.expected_function(span)?;
        let function: Function = parser_context
            .get_symbols()
            .get_function_by_id(span, function_id)?;

        function.get_type()
    };

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    loop {
        if parser_context.check(TokenType::RParen) {
            break;
        }

        let expression: Ast = expr::build_expr(parser_context)?;

        args.push(expression);

        if parser_context.check(TokenType::RParen) {
            break;
        } else {
            parser_context.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Call {
        name,
        args,
        kind: function_type,
        span,
    })
}
