use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::tokentype::TokenType,
        parser::{
            declarations::{asmfn, glasm, glconstant, glstatic, structure, union},
            stmt,
            stmts::{cstype, function},
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

        _ => Ok(stmt::statement(parser_context)?),
    };

    declaration
}
