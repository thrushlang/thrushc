use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
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
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_statement_state(ctx)?;

    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Statement);

    let statement: Result<Ast<'parser>, ThrushCompilerIssue> = match &ctx.peek().kind {
        TokenType::LBrace => Ok(block::build_block(ctx)?),
        TokenType::Return => Ok(terminator::build_return(ctx)?),

        TokenType::Static => Ok(lstatic::build_static(ctx)?),
        TokenType::Const => Ok(constant::build_const(ctx)?),
        TokenType::Local => Ok(local::build_local(ctx)?),
        TokenType::Instr => Ok(lli::build_lli(ctx)?),

        TokenType::If => Ok(conditional::build_conditional(ctx)?),

        TokenType::For => Ok(loops::build_for_loop(ctx)?),
        TokenType::While => Ok(loops::build_while_loop(ctx)?),
        TokenType::Loop => Ok(loops::build_loop(ctx)?),

        TokenType::Continue => Ok(controlflow::build_continue(ctx)?),
        TokenType::Break => Ok(controlflow::build_break(ctx)?),

        _ => Ok(expr::build_expression(ctx)?),
    };

    ctx.get_mut_control_ctx().pop_sync_position();

    statement
}

fn check_statement_state(ctx: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(ctx)
}
