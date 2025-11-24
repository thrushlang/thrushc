use crate::core::errors::standard::ThrushCompilerIssue;
use crate::front_end::parser::ParserContext;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;

#[inline]
pub fn check_inside_loop_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if !ctx.get_control_ctx().is_inside_loop() {
        ctx.only_advance()?;

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "It must be contained inside loop.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}

#[inline]
pub fn check_inside_function_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if !ctx.get_control_ctx().get_inside_function() {
        ctx.only_advance()?;

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "It must be contained inside a function.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}

#[inline]
pub fn check_main_scope_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if !ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "It must be contained in the main scope.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}

#[inline]
pub fn check_double_entrypoint_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if ctx.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            "Duplicated entrypoint".into(),
            "The language not support two entrypoints.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}

#[inline]
pub fn check_double_global_assembler_state(
    ctx: &mut ParserContext,
) -> Result<(), ThrushCompilerIssue> {
    if ctx.get_control_ctx().get_global_asm() {
        ctx.only_advance()?;

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Global assembler is one per-file.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}
