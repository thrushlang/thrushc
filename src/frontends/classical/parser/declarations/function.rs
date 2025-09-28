use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, attributes, checks, declarations::entrypoint, statements::block, typegen,
        },
        types::{
            ast::{Ast, metadata::fnparam::FunctionParameterMetadata},
            parser::{
                stmts::{
                    traits::{ThrushAttributesExtensions, TokenExtensions},
                    types::ThrushAttributes,
                },
                symbols::types::ParametersTypes,
            },
        },
        typesystem::types::Type,
    },
};

pub fn build_function<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::Fn,
        String::from("Syntax error"),
        String::from("Expected 'fn' keyword."),
    )?;

    let function_name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected identifier."),
    )?;

    let function_name: &str = function_name_tk.get_lexeme();
    let function_ascii_name: &str = function_name_tk.get_ascii_lexeme();

    let span: Span = function_name_tk.get_span();

    if function_name == "main" {
        if declare_forward {
            return Ok(Ast::Null { span });
        }

        ctx.get_mut_control_ctx().set_inside_function(true);

        let entrypoint: Result<Ast, ThrushCompilerIssue> = entrypoint::build_entrypoint(ctx, span);

        ctx.get_mut_control_ctx().set_inside_function(false);

        return entrypoint;
    }

    ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameters_types: Vec<Type> = Vec::with_capacity(10);

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
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    let return_type: Type = typegen::build_type(ctx)?;

    let attributes: ThrushAttributes =
        attributes::build_attributes(ctx, &[TokenType::SemiColon, TokenType::LBrace])?;

    let function_has_ignore: bool = attributes.has_ignore_attribute();

    let mut function: Ast = Ast::Function {
        name: function_name,
        ascii_name: function_ascii_name,
        parameters: parameters.clone(),
        parameter_types: parameters_types.clone(),
        body: Ast::Null { span }.into(),
        return_type: return_type.clone(),
        attributes,
        span,
    };

    if declare_forward {
        if let Err(error) = ctx.get_mut_symbols().new_function(
            function_name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                function_has_ignore,
            ),
            span,
        ) {
            ctx.add_silent_error(error);
        }

        if ctx.match_token(TokenType::SemiColon)? {
            return Ok(function);
        }

        return Ok(Ast::Null { span });
    }

    if ctx.match_token(TokenType::SemiColon)? {
        return Ok(function);
    }

    ctx.get_mut_control_ctx().set_inside_function(true);

    ctx.get_mut_type_ctx()
        .set_function_type(return_type.clone());

    if let Err(error) = ctx.get_mut_symbols().start_parameters(&parameters) {
        ctx.add_silent_error(error);
    }

    let function_body: Ast = block::build_block(ctx)?;

    ctx.get_mut_symbols().end_parameters();
    ctx.get_mut_control_ctx().set_inside_function(false);

    if let Ast::Function { body, .. } = &mut function {
        *body = function_body.into();
    }

    Ok(function)
}
