use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::checks;
use crate::frontend::parser::statements::block;
use crate::frontend::parser::statements::conditional;
use crate::frontend::parser::statements::controlflow;
use crate::frontend::parser::statements::lconstant;
use crate::frontend::parser::statements::lctype;
use crate::frontend::parser::statements::lenum;
use crate::frontend::parser::statements::lli;
use crate::frontend::parser::statements::local;
use crate::frontend::parser::statements::loops;
use crate::frontend::parser::statements::lstatic;
use crate::frontend::parser::statements::lstructure;
use crate::frontend::parser::statements::terminator;
use crate::frontend::types::ast::Ast;

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
        TokenType::Const => Ok(lconstant::build_const(ctx)?),
        TokenType::Struct => Ok(lstructure::build_structure(ctx)?),
        TokenType::Type => Ok(lctype::build_custom_type(ctx)?),
        TokenType::Enum => Ok(lenum::build_enum(ctx)?),

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
