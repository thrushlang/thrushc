use super::{ParserContext, contexts::sync::ParserSyncPosition};

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::declarations::asmfn;
use crate::frontend::parser::declarations::cstype;
use crate::frontend::parser::declarations::function;
use crate::frontend::parser::declarations::glasm;
use crate::frontend::parser::declarations::glconstant;
use crate::frontend::parser::declarations::glstatic;
use crate::frontend::parser::declarations::structure;
use crate::frontend::parser::declarations::union;
use crate::frontend::parser::statement;
use crate::frontend::types::ast::Ast;

pub fn decl<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Declaration);

    let declaration: Result<Ast<'parser>, ThrushCompilerIssue> = match &ctx.peek().kind {
        TokenType::Type => Ok(cstype::build_custom_type(ctx, false)?),
        TokenType::Struct => Ok(structure::build_structure(ctx, false)?),
        TokenType::Const => Ok(glconstant::build_global_const(ctx, false)?),
        TokenType::Static => Ok(glstatic::build_global_static(ctx, false)?),
        TokenType::Enum => Ok(union::build_enum(ctx, false)?),
        TokenType::Fn => Ok(function::build_function(ctx, false)?),
        TokenType::AsmFn => Ok(asmfn::build_assembler_function(ctx, false)?),
        TokenType::GlobalAsm => Ok(glasm::build_global_assembler(ctx)?),

        _ => Ok(statement::parse(ctx)?),
    };

    ctx.get_mut_control_ctx().pop_sync_position();

    declaration
}

pub fn parse_forward(ctx: &mut ParserContext) {
    let mut entered_at_block: bool = false;

    while !ctx.is_eof() {
        match &ctx.peek().kind {
            TokenType::Type if !entered_at_block => {
                let _ = cstype::build_custom_type(ctx, true);
            }

            TokenType::Struct if !entered_at_block => {
                let _ = structure::build_structure(ctx, true);
            }

            TokenType::Static if !entered_at_block => {
                let _ = glstatic::build_global_static(ctx, true);
            }

            TokenType::Const if !entered_at_block => {
                let _ = glconstant::build_global_const(ctx, true);
            }

            TokenType::Enum if !entered_at_block => {
                let _ = union::build_enum(ctx, true);
            }

            TokenType::Fn if !entered_at_block => {
                let _ = function::build_function(ctx, true);
            }

            TokenType::AsmFn if !entered_at_block => {
                let _ = asmfn::build_assembler_function(ctx, true);
            }

            TokenType::LBrace => {
                entered_at_block = true;
                let _ = ctx.only_advance();
            }

            TokenType::RBrace => {
                entered_at_block = false;
                let _ = ctx.only_advance();
            }

            _ => {
                let _ = ctx.only_advance();
            }
        }
    }

    ctx.current = 0;
}
