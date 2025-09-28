use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{parser::ParserContext, types::parser::stmts::traits::TokenExtensions},
};

#[inline]
pub fn check_unreacheable_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if ctx.is_unreacheable_code() {
        ctx.only_advance()?;

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "Unreachable for execution.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}

#[inline]
pub fn check_inside_loop_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    if !ctx.get_control_ctx().is_inside_loop() {
        ctx.only_advance()?;

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "It must be contained within a loop block.".into(),
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
            "It must be contained within a function block.".into(),
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
            "It must be contained within the main scope.".into(),
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
            "The global assembler is per-file.".into(),
            None,
            ctx.previous().get_span(),
        ));
    }

    Ok(())
}
