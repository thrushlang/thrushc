use std::rc::Rc;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::frontend::lexer::span::Span;
use crate::frontend::lexer::token::Token;
use crate::frontend::lexer::tokentype::TokenType;
use crate::frontend::parser::ParserContext;
use crate::frontend::parser::checks;
use crate::frontend::parser::statements::block;
use crate::frontend::parser::typegen;
use crate::frontend::types::ast::Ast;
use crate::frontend::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::frontend::types::parser::stmts::traits::TokenExtensions;
use crate::frontend::typesystem::types::Type;

pub fn build_entrypoint<'parser>(
    ctx: &mut ParserContext<'parser>,
    span: Span,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_double_entrypoint_state(ctx)?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameter_position: u32 = 0;

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

        let parameter_tk: &Token = ctx.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected identifier.".into(),
        )?;

        let name: &str = parameter_tk.get_lexeme();
        let ascii_name: &str = parameter_tk.get_ascii_lexeme();

        let span: Span = parameter_tk.get_span();

        ctx.consume(
            TokenType::Colon,
            "Syntax error".into(),
            "Expected ':'.".into(),
        )?;

        let parameter_type: Type = typegen::build_type(ctx)?;

        parameters.push(Ast::FunctionParameter {
            name,
            ascii_name,
            kind: parameter_type,
            position: parameter_position,
            metadata: FunctionParameterMetadata::new(is_mutable),
            span,
        });

        parameter_position += 1;

        if ctx.check(TokenType::RParen) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    ctx.consume(
        TokenType::U32,
        "Syntax error".into(),
        "Expected 'u32'.".into(),
    )?;

    ctx.get_mut_control_ctx().set_inside_function(true);
    ctx.get_mut_control_ctx().set_has_entrypoint();
    ctx.get_mut_type_ctx().set_function_type(Type::U32);

    ctx.get_mut_symbols().declare_parameters(&parameters)?;

    let body: Rc<Ast> = block::build_block(ctx)?.into();

    ctx.get_mut_symbols().finish_parameters();
    ctx.get_mut_control_ctx().set_inside_function(false);

    Ok(Ast::EntryPoint {
        body,
        parameters,
        span,
    })
}
