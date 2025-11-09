use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::checks;
use crate::front_end::parser::declarations::entrypoint;
use crate::front_end::parser::statements::block;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::front_end::types::parser::stmts::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;
use crate::front_end::types::parser::symbols::types::ParametersTypes;
use crate::front_end::typesystem::types::Type;

pub fn build_function<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::Fn,
        "Syntax error".into(),
        "Expected 'fn' keyword.".into(),
    )?;

    let function_name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected identifier.".into(),
    )?;

    let name: &str = function_name_tk.get_lexeme();
    let ascii_name: &str = function_name_tk.get_ascii_lexeme();
    let span: Span = function_name_tk.get_span();

    if name == "main" {
        if declare_forward {
            return Ok(Ast::new_nullptr(span));
        }

        return entrypoint::build_entrypoint(ctx, span);
    }

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameters_types: Vec<Type> = Vec::with_capacity(10);

    let mut parameter_position: u32 = 0;

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let is_mutable: bool = ctx.match_token(TokenType::Mut)?;

        let parameter_name_tk: &Token = ctx.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected 'identifier'.".into(),
        )?;

        let name: &str = parameter_name_tk.get_lexeme();
        let ascii_name: &str = parameter_name_tk.get_ascii_lexeme();
        let span: Span = parameter_name_tk.get_span();

        ctx.consume(
            TokenType::Colon,
            "Syntax error".into(),
            "Expected ':'.".into(),
        )?;

        let parameter_type: Type = typegen::build_type(ctx)?;

        parameters_types.push(parameter_type.clone());

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

    let return_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::SemiColon, TokenType::LBrace])?;

    let function_has_ignore: bool = attributes.has_ignore_attribute();

    let mut function: Ast = Ast::Function {
        name,
        ascii_name,
        parameters: parameters.clone(),
        parameter_types: parameters_types.clone(),
        body: None,
        return_type: return_type.clone(),
        attributes,
        span,
    };

    if declare_forward {
        ctx.get_mut_symbols().new_function(
            name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                function_has_ignore,
            ),
            span,
        )?;

        if ctx.match_token(TokenType::SemiColon)? {
            return Ok(function);
        }

        return Ok(Ast::new_nullptr(span));
    }

    if ctx.match_token(TokenType::SemiColon)? {
        return Ok(function);
    }

    ctx.get_mut_control_ctx().set_inside_function(true);

    ctx.get_mut_type_ctx()
        .set_function_type(return_type.clone());

    ctx.get_mut_symbols().declare_parameters(&parameters)?;

    let function_body: Ast = block::build_block(ctx)?;

    ctx.get_mut_symbols().finish_parameters();
    ctx.get_mut_control_ctx().set_inside_function(false);

    if let Ast::Function { body, .. } = &mut function {
        *body = Some(function_body.into());
    }

    Ok(function)
}
