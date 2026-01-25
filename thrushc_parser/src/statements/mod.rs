pub mod block;
pub mod conditional;
pub mod controlflow;
pub mod defer;
pub mod lconstant;
pub mod lctype;
pub mod lenum;
pub mod local;
pub mod loops;
pub mod lstatic;
pub mod lstructure;
pub mod terminator;

use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_token::tokentype::TokenType;

use crate::{ParserContext, context::ParserSyncPosition, expressions};

pub fn parse<'parser>(ctx: &mut ParserContext<'parser>) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Statement);

    let statement: Result<Ast<'parser>, CompilationIssue> = match &ctx.peek().kind {
        TokenType::LBrace => Ok(block::build_block(ctx)?),
        TokenType::Return => Ok(terminator::build_return(ctx)?),

        TokenType::Static => Ok(lstatic::build_static(ctx)?),
        TokenType::Const => Ok(lconstant::build_const(ctx)?),
        TokenType::Struct => Ok(lstructure::build_structure(ctx)?),
        TokenType::Type => Ok(lctype::build_custom_type(ctx)?),
        TokenType::Enum => Ok(lenum::build_enum(ctx)?),

        TokenType::Local => Ok(local::build_local(ctx)?),

        TokenType::If => Ok(conditional::build_conditional(ctx)?),

        TokenType::For => Ok(loops::build_for_loop(ctx)?),
        TokenType::While => Ok(loops::build_while_loop(ctx)?),
        TokenType::Loop => Ok(loops::build_loop(ctx)?),

        TokenType::Continue => Ok(controlflow::build_continue(ctx)?),
        TokenType::ContinueAll => Ok(controlflow::build_continueall(ctx)?),
        TokenType::Break => Ok(controlflow::build_break(ctx)?),
        TokenType::BreakAll => Ok(controlflow::build_breakall(ctx)?),

        TokenType::Defer => Ok(defer::build_defer_executation(ctx)?),

        _ => Ok(expressions::build_expression(ctx)?),
    };

    ctx.get_mut_control_ctx().pop_sync_position();

    statement
}
