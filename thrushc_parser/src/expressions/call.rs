use thrushc_ast::Ast;
use thrushc_entities::parser::{FoundSymbolId, Function, Intrinsic};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::tokentype::TokenType;
use thrushc_typesystem::Type;

use crate::{
    ParserContext, expressions,
    traits::{FoundSymbolEitherExtensions, FoundSymbolExtensions},
};

pub fn build_call<'parser>(
    ctx: &mut ParserContext<'parser>,
    name: &'parser str,
    span: Span,
) -> Result<Ast<'parser>, CompilationIssue> {
    let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;

    let function_type: Type = if object.is_intrinsic() {
        let id: &str = object.expected_intrinsic(span)?;
        let intrinsic: Intrinsic = ctx.get_symbols().get_intrinsic_by_id(span, id)?;

        crate::traits::IntrinsicExtensions::get_type(&intrinsic)
    } else if object.is_function_asm() {
        let id: &str = object.expected_asm_function(span)?;
        let asm_function: thrushc_entities::parser::AssemblerFunction =
            ctx.get_symbols().get_asm_function_by_id(span, id)?;

        crate::traits::FunctionAssemblerExtensions::get_type(&asm_function)
    } else {
        let id: &str = object.expected_function(span)?;
        let function: Function = ctx.get_symbols().get_function_by_id(span, id)?;

        crate::traits::FunctionExtensions::get_type(&function)
    };

    let mut args: Vec<Ast> = Vec::with_capacity(10);

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        args.push(expressions::build_expr(ctx)?);

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
