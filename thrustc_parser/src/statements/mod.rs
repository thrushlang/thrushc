/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

*/

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

use thrustc_ast::Ast;
use thrustc_errors::CompilationIssue;
use thrustc_parser_context::{SynchronizationPosition, traits::ControlContextExtensions};
use thrustc_token_type::TokenType;

use crate::{ParserContext, expressions};

pub fn parse<'parser>(ctx: &mut ParserContext<'parser>) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_context()
        .add_sync_position(SynchronizationPosition::Statement);

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

    ctx.get_mut_control_context().pop_sync_position();

    statement
}
