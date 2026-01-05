use thrushc_ast::Ast;
use thrushc_errors::CompilationIssue;
use thrushc_token::{tokentype::TokenType, traits::TokenExtensions};

use crate::{ParserContext, context::ParserSyncPosition, statements};

pub mod asmfn;
pub mod function;
pub mod glasm;
pub mod glconstant;
pub mod glcstype;
pub mod glenum;
pub mod glstatic;
pub mod glstructure;
pub mod import;
pub mod import_c;
pub mod intrinsic;

pub fn parse<'parser>(ctx: &mut ParserContext<'parser>) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.get_mut_control_ctx()
        .add_sync_position(ParserSyncPosition::Declaration);

    let declaration: Result<Ast<'parser>, CompilationIssue> = match ctx.peek().get_type() {
        TokenType::Type => Ok(glcstype::build_custom_type(ctx, false)?),
        TokenType::Struct => Ok(glstructure::build_structure(ctx, false)?),
        TokenType::Const => Ok(glconstant::build_global_const(ctx, false)?),
        TokenType::Static => Ok(glstatic::build_global_static(ctx, false)?),
        TokenType::Enum => Ok(glenum::build_enum(ctx, false)?),
        TokenType::Fn => Ok(function::build_function(ctx, false)?),
        TokenType::AsmFn => Ok(asmfn::build_assembler_function(ctx, false)?),
        TokenType::Intrinsic => Ok(intrinsic::build_intrinsic(ctx, false)?),
        TokenType::GlobalAsm => Ok(glasm::build_global_assembler(ctx)?),
        TokenType::Import => Ok(import::build_import(ctx)?),

        _ => Ok(statements::parse(ctx)?),
    };

    ctx.get_mut_control_ctx().pop_sync_position();

    declaration
}

pub fn parse_forward(ctx: &mut ParserContext) {
    let mut entered_at_block: bool = false;

    while !ctx.is_eof() {
        match &ctx.peek().kind {
            TokenType::Type if !entered_at_block => {
                let _ = glcstype::build_custom_type(ctx, true);
            }

            TokenType::Struct if !entered_at_block => {
                let _ = glstructure::build_structure(ctx, true);
            }

            TokenType::Static if !entered_at_block => {
                let _ = glstatic::build_global_static(ctx, true);
            }

            TokenType::Const if !entered_at_block => {
                let _ = glconstant::build_global_const(ctx, true);
            }

            TokenType::Enum if !entered_at_block => {
                let _ = glenum::build_enum(ctx, true);
            }

            TokenType::Intrinsic if !entered_at_block => {
                let _ = intrinsic::build_intrinsic(ctx, true);
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
