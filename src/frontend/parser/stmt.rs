use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::tokentype::TokenType,
        parser::{
            checks,
            declarations::{asmfn, glasm, glconstant, structure, union},
            stmts::{
                block, conditional, constant, controlflow, cstype, function, lli, local, loops,
                terminator,
            },
        },
        types::ast::Ast,
    },
};

use super::{ParserContext, contexts::sync::SyncPosition, expr};

pub fn declaration<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Declaration);

    let declaration: Result<Ast<'parser>, ThrushCompilerIssue> = match &parser_context.peek().kind {
        TokenType::Type => Ok(cstype::build_custom_type(parser_context, false)?),
        TokenType::Struct => Ok(structure::build_structure(parser_context, false)?),
        TokenType::Const => Ok(glconstant::build_global_const(parser_context, false)?),
        TokenType::Enum => Ok(union::build_enum(parser_context, false)?),
        TokenType::Fn => Ok(function::build_function(parser_context, false)?),
        TokenType::AsmFn => Ok(asmfn::build_assembler_function(parser_context, false)?),
        TokenType::GlobalAsm => Ok(glasm::build_global_assembler(parser_context)?),

        _ => Ok(self::statement(parser_context)?),
    };

    declaration
}

pub fn statement<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    self::check_statement_state(parser_context)?;

    parser_context
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Statement);

    let statement: Result<Ast<'parser>, ThrushCompilerIssue> = match &parser_context.peek().kind {
        TokenType::LBrace => Ok(block::build_block(parser_context)?),
        TokenType::Return => Ok(terminator::build_return(parser_context)?),

        TokenType::Const => Ok(constant::build_const(parser_context)?),
        TokenType::Local => Ok(local::build_local(parser_context)?),
        TokenType::Instr => Ok(lli::build_lli(parser_context)?),

        TokenType::If => Ok(conditional::build_conditional(parser_context)?),

        TokenType::For => Ok(loops::build_for_loop(parser_context)?),
        TokenType::While => Ok(loops::build_while_loop(parser_context)?),
        TokenType::Loop => Ok(loops::build_loop(parser_context)?),

        TokenType::Continue => Ok(controlflow::build_continue(parser_context)?),
        TokenType::Break => Ok(controlflow::build_break(parser_context)?),

        _ => Ok(expr::build_expression(parser_context)?),
    };

    statement
}

fn check_statement_state(parser_context: &mut ParserContext) -> Result<(), ThrushCompilerIssue> {
    checks::check_unreacheable_state(parser_context)
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
