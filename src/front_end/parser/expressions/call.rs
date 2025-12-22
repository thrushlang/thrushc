use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::{FoundSymbolEither, FoundSymbolExtension};
use crate::front_end::types::parser::symbols::traits::FunctionExtensions;
use crate::front_end::types::parser::symbols::types::{
    AssemblerFunction, FoundSymbolId, Function, Intrinsic,
};
use crate::front_end::typesystem::types::Type;

pub fn build_call<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, CompilationIssue> {
    let object_result: Result<FoundSymbolId, CompilationIssue> =
        ctx.get_symbols().get_symbols_id(name, span);

    if let Err(issue) = object_result {
        ctx.add_error(issue);
        return Ok(Ast::new_nullptr(span));
    }

    let object: FoundSymbolId = object_result?;

    let function_type: Type = if object.is_intrinsic() {
        let id: &str = object.expected_intrinsic(span)?;
        let intrinsic: Intrinsic = ctx.get_symbols().get_intrinsic_by_id(span, id)?;

        intrinsic.get_type()
    } else if object.is_function_asm() {
        let id: &str = object.expected_asm_function(span)?;
        let asm_function: AssemblerFunction = ctx.get_symbols().get_asm_function_by_id(span, id)?;

        asm_function.get_type()
    } else {
        let id: &str = object.expected_function(span)?;
        let function: Function = ctx.get_symbols().get_function_by_id(span, id)?;

        function.get_type()
    };

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let expression: Ast = expressions::build_expr(ctx)?;

        args.push(expression);

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Call {
        name,
        args,
        kind: function_type,
        span,
    })
}
