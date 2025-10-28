use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::{span::Span, tokentype::TokenType};
use crate::frontend::parser::{ParserContext, expr};
use crate::frontend::types::ast::Ast;
use crate::frontend::types::parser::stmts::traits::{FoundSymbolEither, FoundSymbolExtension};
use crate::frontend::types::parser::symbols::traits::FunctionExtensions;
use crate::frontend::types::parser::symbols::types::{AssemblerFunction, FoundSymbolId, Function};
use crate::frontend::typesystem::types::Type;

pub fn build_call<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;

    let function_type: Type = if object.is_function_asm() {
        let asm_function_id: &str = object.expected_asm_function(span)?;
        let asm_function: AssemblerFunction = ctx
            .get_symbols()
            .get_asm_function_by_id(span, asm_function_id)?;

        asm_function.get_type()
    } else {
        let function_id: &str = object.expected_function(span)?;
        let function: Function = ctx.get_symbols().get_function_by_id(span, function_id)?;

        function.get_type()
    };

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let expression: Ast = expr::build_expr(ctx)?;

        args.push(expression);

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Call {
        name,
        args,
        kind: function_type,
        span,
    })
}
