use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{
            lexer::ThrushType,
            parser::{
                stmts::{
                    stmt::ThrushStatement,
                    traits::{FoundSymbolEither, FoundSymbolExtension},
                },
                symbols::{
                    traits::FunctionExtensions,
                    types::{AssemblerFunction, FoundSymbolId, Function},
                },
            },
        },
    },
};

pub fn build_call<'instr>(
    parser_context: &mut ParserContext<'instr>,
    name: &'instr str,
    span: Span,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let object: FoundSymbolId = parser_context.get_symbols().get_symbols_id(name, span)?;

    let function_type: ThrushType = if object.is_function_asm() {
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

    let mut args: Vec<ThrushStatement> = Vec::with_capacity(10);

    loop {
        if parser_context.check(TokenType::RParen) {
            break;
        }

        let expression: ThrushStatement = expression::build_expr(parser_context)?;

        if expression.is_constructor() {
            return Err(ThrushCompilerIssue::Error(
                String::from("Syntax error"),
                String::from("Constructor should be stored in a local variable."),
                None,
                expression.get_span(),
            ));
        }

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

    Ok(ThrushStatement::Call {
        name,
        args,
        kind: function_type,
        span,
    })
}
