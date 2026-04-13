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
pub mod loops;
pub mod lstatic;
pub mod lstructure;
pub mod terminator;
pub mod var;

use thrustc_ast::Ast;
use thrustc_errors::CompilationIssue;
use thrustc_parser_context::{SynchronizationPosition, traits::ControlContextExtensions};
use thrustc_token::traits::TokenExtensions;
use thrustc_token_type::TokenType;

use crate::{ParserContext, expressions};

pub fn parse<'parser>(ctx: &mut ParserContext<'parser>) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_context()
        .add_sync_position(SynchronizationPosition::Statement);

    let statement: Result<Ast<'parser>, CompilationIssue> = match &ctx.peek().get_type() {
        TokenType::LBrace => Ok(block::parse_code_block_stmt(ctx)?),
        TokenType::Return => Ok(terminator::parse_return_stmt(ctx)?),
        TokenType::Static => Ok(lstatic::parse_static_stmt(ctx)?),
        TokenType::Const => Ok(lconstant::parse_constant_stmt(ctx)?),
        TokenType::Struct => Ok(lstructure::parse_structure_stmt(ctx)?),
        TokenType::Type => Ok(lctype::parse_custom_type_stmt(ctx)?),
        TokenType::Enum => Ok(lenum::parse_enum_stmt(ctx)?),
        TokenType::Var => Ok(var::build_variable_stmt(ctx)?),
        TokenType::If => Ok(conditional::build_conditional(ctx)?),
        TokenType::For => Ok(loops::parse_for_loop_stmt(ctx)?),
        TokenType::While => Ok(loops::parse_while_loop_stmt(ctx)?),
        TokenType::Loop => Ok(loops::parse_loop_stmt(ctx)?),
        TokenType::Continue => Ok(controlflow::parse_continue_stmt(ctx)?),
        TokenType::ContinueAll => Ok(controlflow::parse_continueall_stmt(ctx)?),
        TokenType::Break => Ok(controlflow::parse_break_stmt(ctx)?),
        TokenType::BreakAll => Ok(controlflow::parse_breakall_stmt(ctx)?),
        TokenType::Defer => Ok(defer::parse_post_executation_stmt(ctx)?),

        _ => Ok(expressions::parse_expression(ctx)?),
    };

    ctx.get_mut_control_context().pop_sync_position();

    statement
}
