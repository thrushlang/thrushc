use std::rc::Rc;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, checks, statements::block, typegen},
        types::{
            ast::{Ast, metadata::fnparam::FunctionParameterMetadata},
            parser::stmts::traits::TokenExtensions,
        },
        typesystem::types::Type,
    },
};

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
            String::from("Syntax error"),
            String::from("Expected identifier."),
        )?;

        let name: &str = parameter_tk.get_lexeme();
        let ascii_name: &str = parameter_tk.get_ascii_lexeme();

        let span: Span = parameter_tk.get_span();

        ctx.consume(
            TokenType::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
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
                String::from("Syntax error"),
                String::from("Expected ','."),
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

    if let Err(error) = ctx.get_mut_symbols().start_parameters(&parameters) {
        ctx.add_silent_error(error);
    }

    let body: Rc<Ast> = block::build_block(ctx)?.into();

    ctx.get_mut_symbols().end_parameters();
    ctx.get_mut_control_ctx().set_inside_function(false);

    Ok(Ast::EntryPoint {
        body,
        parameters,
        span,
    })
}
