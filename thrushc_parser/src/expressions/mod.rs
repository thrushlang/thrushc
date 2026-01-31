pub mod array;
pub mod asm;
pub mod call;
pub mod constructor;
pub mod deref;
pub mod enumv;
pub mod farray;
pub mod index;
pub mod precedences;
pub mod property;
pub mod reference;

use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_token::tokentype::TokenType;

use crate::{ParserContext, context::ParserSyncPosition};
pub fn build_expression<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Expression);

    let expression: Ast = precedences::or::or_precedence(ctx)?;

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

    let expr: Ast = precedences::or::or_precedence(ctx)?;

    ctx.get_mut_control_ctx().pop_sync_position();

    Ok(expr)
}
