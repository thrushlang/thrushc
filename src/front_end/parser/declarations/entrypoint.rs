use std::rc::Rc;

use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::checks;
use crate::front_end::parser::statements::block;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

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
        TokenType::S32,
        "Syntax error".into(),
        "Expected 's32' type.".into(),
    )?;

    ctx.get_mut_control_ctx().set_inside_function(true);
    ctx.get_mut_control_ctx().set_has_entrypoint();
    ctx.get_mut_type_ctx().set_function_type(Type::S32);

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
