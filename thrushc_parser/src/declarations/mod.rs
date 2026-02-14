use thrushc_ast::Ast;
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;

use crate::{ParserContext, control::ParserSyncPosition};

pub mod asmfn;
pub mod embedded;
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
        TokenType::Intrinsic => Ok(intrinsic::build_compiler_intrinsic(ctx, false)?),
        TokenType::GlobalAsm => Ok(glasm::build_global_assembler(ctx)?),
        TokenType::Import => Ok(import::build_import(ctx)?),
        TokenType::Embedded => Ok(embedded::build_embedded(ctx)?),

        _ => {
            let what: &Token = ctx.advance()?;

            Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected a declaration or defination, not a statement and never an expression."
                    .into(),
                None,
                what.get_span(),
            ))
        }
    };

    ctx.get_mut_control_ctx().pop_sync_position();

    declaration
}

pub fn parse_forward(ctx: &mut ParserContext) {
    let mut entered_at_block: bool = false;

    while !ctx.is_eof() {
        match ctx.peek().get_type() {
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
                let _ = intrinsic::build_compiler_intrinsic(ctx, true);
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

    while !ctx.is_eof() {
        if ctx.peek().get_type() == TokenType::Import {
            let _ = import::build_import(ctx);
        }

        let _ = ctx.only_advance();
    }

    ctx.current = 0;
}
