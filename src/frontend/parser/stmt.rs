use ahash::AHashMap as HashMap;

use crate::{
    backend::llvm::compiler::conventions::CallConvention,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::tokentype::TokenType,
        parser::stmts::{
            asmfunction, block, conditional, constant, controlflow, cstype, function, lli, local,
            loops, structure, terminator, union,
        },
        types::ast::Ast,
    },
    lazy_static,
};

use super::{ParserContext, contexts::SyncPosition, expression};

const CALL_CONVENTIONS_CAPACITY: usize = 10;

lazy_static! {
    pub static ref CALL_CONVENTIONS: HashMap<&'static [u8], CallConvention> = {
        let mut call_conventions: HashMap<&'static [u8], CallConvention> =
            HashMap::with_capacity(CALL_CONVENTIONS_CAPACITY);

        call_conventions.insert(b"C", CallConvention::Standard);
        call_conventions.insert(b"fast", CallConvention::Fast);
        call_conventions.insert(b"tail", CallConvention::Tail);
        call_conventions.insert(b"cold", CallConvention::Cold);
        call_conventions.insert(b"weakReg", CallConvention::PreserveMost);
        call_conventions.insert(b"strongReg", CallConvention::PreserveAll);
        call_conventions.insert(b"swift", CallConvention::Swift);
        call_conventions.insert(b"haskell", CallConvention::GHC);
        call_conventions.insert(b"erlang", CallConvention::HiPE);

        call_conventions
    };
}

pub fn parse<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Declaration);

    let statement: Result<Ast<'parser>, ThrushCompilerIssue> = match &parser_ctx.peek().kind {
        TokenType::Type => Ok(cstype::build_custom_type(parser_ctx, false)?),
        TokenType::Struct => Ok(structure::build_structure(parser_ctx, false)?),
        TokenType::Enum => Ok(union::build_enum(parser_ctx, false)?),
        TokenType::Fn => Ok(function::build_function(parser_ctx, false)?),
        TokenType::AsmFn => Ok(asmfunction::build_assembler_function(parser_ctx, false)?),

        _ => Ok(self::statement(parser_ctx)?),
    };

    statement
}

pub fn statement<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_ctx
        .get_mut_control_ctx()
        .set_sync_position(SyncPosition::Statement);

    let statement: Result<Ast<'parser>, ThrushCompilerIssue> = match &parser_ctx.peek().kind {
        TokenType::LBrace => Ok(block::build_block(parser_ctx)?),
        TokenType::Return => Ok(terminator::build_return(parser_ctx)?),

        TokenType::Const => Ok(constant::build_const(parser_ctx, false)?),
        TokenType::Local => Ok(local::build_local(parser_ctx)?),
        TokenType::Instr => Ok(lli::build_lli(parser_ctx)?),

        TokenType::If => Ok(conditional::build_conditional(parser_ctx)?),

        TokenType::For => Ok(loops::build_for_loop(parser_ctx)?),
        TokenType::While => Ok(loops::build_while_loop(parser_ctx)?),
        TokenType::Loop => Ok(loops::build_loop(parser_ctx)?),

        TokenType::Continue => Ok(controlflow::build_continue(parser_ctx)?),
        TokenType::Break => Ok(controlflow::build_break(parser_ctx)?),

        _ => Ok(expression::build_expression(parser_ctx)?),
    };

    statement
}
