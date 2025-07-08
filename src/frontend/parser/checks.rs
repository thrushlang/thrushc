use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{parser::ParserContext, types::parser::stmts::traits::TokenExtensions},
};

pub fn check_unreacheable_state(
    parser_context: &mut ParserContext,
) -> Result<(), ThrushCompilerIssue> {
    if parser_context.is_unreacheable_code() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Unreachable for execution."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    Ok(())
}

pub fn check_inside_function_state(
    parser_context: &mut ParserContext,
) -> Result<(), ThrushCompilerIssue> {
    if !parser_context.get_control_ctx().get_inside_function() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("It must be contained within a function block."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    Ok(())
}

pub fn check_main_scope_state(
    parser_context: &mut ParserContext,
) -> Result<(), ThrushCompilerIssue> {
    if !parser_context.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("It must be contained within the main scope."),
            None,
            parser_context.peek().get_span(),
        ));
    }

    Ok(())
}

pub fn check_double_entrypoint_state(
    parser_context: &mut ParserContext,
) -> Result<(), ThrushCompilerIssue> {
    if parser_context.get_control_ctx().get_entrypoint() {
        return Err(ThrushCompilerIssue::Error(
            "Duplicated entrypoint".into(),
            "The language not support two entrypoints.".into(),
            None,
            parser_context.previous().get_span(),
        ));
    }

    Ok(())
}

pub fn check_double_global_assembler_state(
    parser_context: &mut ParserContext,
) -> Result<(), ThrushCompilerIssue> {
    if parser_context.get_control_ctx().get_global_asm() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            "The global assembler is per-file.".into(),
            None,
            parser_context.previous().get_span(),
        ));
    }

    Ok(())
}
