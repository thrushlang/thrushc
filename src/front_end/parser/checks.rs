use crate::core::errors::standard::CompilationIssue;

use crate::front_end::parser::ParserContext;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

#[inline]
pub fn check_inside_function_state(ctx: &mut ParserContext) -> Result<(), CompilationIssue> {
    if !ctx.get_control_ctx().get_inside_function() {
        ctx.only_advance()?;

        return Err(CompilationIssue::Error(
            "Syntax error".into(),
            "It must be contained inside a function.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}

#[inline]
pub fn check_main_scope_state(ctx: &mut ParserContext) -> Result<(), CompilationIssue> {
    if !ctx.is_main_scope() {
        return Err(CompilationIssue::Error(
            "Syntax error".into(),
            "It must be contained in the main scope.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}
