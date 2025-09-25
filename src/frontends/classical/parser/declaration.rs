use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::tokentype::TokenType,
        parser::{
            declarations::{asmfn, glasm, glconstant, glstatic, structure, union},
            statement,
            statements::{cstype, function},
        },
        types::ast::Ast,
    },
};

use super::{ParserContext, contexts::sync::ParserSyncPosition};

pub fn decl<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_context
        .get_mut_control_ctx()
        .set_sync_position(ParserSyncPosition::Declaration);

    let declaration: Result<Ast<'parser>, ThrushCompilerIssue> = match &parser_context.peek().kind {
        TokenType::Type => Ok(cstype::build_custom_type(parser_context, false)?),
        TokenType::Struct => Ok(structure::build_structure(parser_context, false)?),
        TokenType::Const => Ok(glconstant::build_global_const(parser_context, false)?),
        TokenType::Static => Ok(glstatic::build_global_static(parser_context, false)?),
        TokenType::Enum => Ok(union::build_enum(parser_context, false)?),
        TokenType::Fn => Ok(function::build_function(parser_context, false)?),
        TokenType::AsmFn => Ok(asmfn::build_assembler_function(parser_context, false)?),
        TokenType::GlobalAsm => Ok(glasm::build_global_assembler(parser_context)?),

        _ => Ok(statement::parse(parser_context)?),
    };

    declaration
}

pub fn parse_forward(parser_context: &mut ParserContext) {
    let mut entered_at_block: bool = false;

    while !parser_context.is_eof() {
        match &parser_context.peek().kind {
            TokenType::Type if !entered_at_block => {
                let _ = cstype::build_custom_type(parser_context, true);
            }

            TokenType::Struct if !entered_at_block => {
                let _ = structure::build_structure(parser_context, true);
            }

            TokenType::Static if !entered_at_block => {
                let _ = glstatic::build_global_static(parser_context, true);
            }

            TokenType::Const if !entered_at_block => {
                let _ = glconstant::build_global_const(parser_context, true);
            }

            TokenType::Enum if !entered_at_block => {
                let _ = union::build_enum(parser_context, true);
            }

            TokenType::Fn if !entered_at_block => {
                let _ = function::build_function(parser_context, true);
            }

            TokenType::AsmFn if !entered_at_block => {
                let _ = asmfn::build_assembler_function(parser_context, true);
            }

            TokenType::LBrace => {
                entered_at_block = true;
                let _ = parser_context.only_advance();
            }

            TokenType::RBrace => {
                entered_at_block = false;
                let _ = parser_context.only_advance();
            }

            _ => {
                let _ = parser_context.only_advance();
            }
        }
    }

    parser_context.current = 0;
}
