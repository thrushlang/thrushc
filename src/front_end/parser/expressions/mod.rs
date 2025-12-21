pub mod array;
pub mod asm;
pub mod call;
pub mod constructor;
pub mod deref;
pub mod enumv;
pub mod farray;
pub mod index;
pub mod indirect;
pub mod lli;
pub mod precedences;
pub mod property;
pub mod reference;

use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::contexts::sync::ParserSyncPosition;
use crate::front_end::parser::expressions::precedences::or;
use crate::front_end::types::ast::Ast;

pub fn build_expression<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Expression);

    let expression: Ast = or::or_precedence(ctx)?;

    ctx.consume(
        TokenType::SemiColon,
        CompilationIssueCode::E0001,
        String::from("Expected ';'."),
    )?;

    ctx.get_mut_control_ctx().pop_sync_position();

    Ok(expression)
}

pub fn build_expr<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Expression);

    let expr: Ast = or::or_precedence(ctx)?;

    ctx.get_mut_control_ctx().pop_sync_position();

    Ok(expr)
}
