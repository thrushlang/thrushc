use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::tokentype::TokenType,
        parser::{
            checks,
            statements::{
                block, conditional, constant, controlflow, lli, local, loops, lstatic, terminator,
            },
        },
        types::ast::Ast,
    },
};

use super::{ParserContext, contexts::sync::ParserSyncPosition, expr};

pub fn parse<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_statement_state(parser_context)?;

    parser_context
        .get_mut_control_ctx()
        .set_sync_position(ParserSyncPosition::Statement);

    let statement: Result<Ast<'parser>, ThrushCompilerIssue> = match &parser_context.peek().kind {
        TokenType::LBrace => Ok(block::build_block(parser_context)?),
        TokenType::Return => Ok(terminator::build_return(parser_context)?),

        TokenType::Static => Ok(lstatic::build_static(parser_context)?),
        TokenType::Const => Ok(constant::build_const(parser_context)?),
        TokenType::Local => Ok(local::build_local(parser_context)?),
        TokenType::Instr => Ok(lli::build_lli(parser_context)?),

        TokenType::If => Ok(conditional::build_conditional(parser_context)?),

        TokenType::For => Ok(loops::build_for_loop(parser_context)?),
        TokenType::While => Ok(loops::build_while_loop(parser_context)?),
        TokenType::Loop => Ok(loops::build_loop(parser_context)?),

        TokenType::Continue => Ok(controlflow::build_continue(parser_context)?),
        TokenType::Break => Ok(controlflow::build_break(parser_context)?),

        _ => Ok(expr::build_expression(parser_context)?),
    };

    statement
}

fn check_statement_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)
}
