use crate::{
    ParserContext,
    expressions::{self, precedences},
};
use thrushc_ast::{Ast, traits::AstGetType};
use thrushc_errors::CompilationIssue;
use thrushc_token_type::TokenType;
use thrushc_typesystem::{Type, traits::TypeIsExtensions};

pub fn indirect_precedence<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.enter_expression()?;

    let mut expr: Ast = precedences::lower::lower_precedence(ctx)?;

    if ctx.check(TokenType::LParen) {
        let expr_type: &Type = expr.get_value_type()?;

        if expr_type.is_function_reference_type() {
            expr = expressions::call::build_anonymous_call(ctx, expr)?;
        }
    }

    ctx.leave_expression();

    Ok(expr)
}
