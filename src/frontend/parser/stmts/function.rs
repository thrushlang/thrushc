use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{
            ParserContext, attributes,
            stmts::{block, entrypoint},
            typegen,
        },
        types::{
            ast::Ast,
            lexer::ThrushType,
            parser::{
                stmts::{
                    traits::{ThrushAttributesExtensions, TokenExtensions},
                    types::ThrushAttributes,
                },
                symbols::types::ParametersTypes,
            },
        },
    },
};

pub fn build_function<'parser>(
    parser_ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    parser_ctx.consume(
        TokenType::Fn,
        String::from("Syntax error"),
        String::from("Expected 'fn' keyword."),
    )?;

    let function_name_tk: &Token = parser_ctx.consume(
        TokenType::Identifier,
        String::from("Syntax error"),
        String::from("Expected name to the function."),
    )?;

    let function_name: &str = function_name_tk.get_lexeme();
    let function_ascii_name: &str = function_name_tk.get_ascii_lexeme();

    let span: Span = function_name_tk.get_span();

    if !parser_ctx.is_main_scope() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Functions can only be defined globally."),
            None,
            span,
        ));
    }

    if function_name == "main" {
        if declare_forward {
            return Ok(Ast::Null { span });
        }

        parser_ctx.get_mut_control_ctx().set_inside_function(true);

        let entrypoint: Result<Ast, ThrushCompilerIssue> = entrypoint::build_main(parser_ctx);

        parser_ctx.get_mut_control_ctx().set_inside_function(false);

        return entrypoint;
    }

    parser_ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameters_types: Vec<ThrushType> = Vec::with_capacity(10);

    let mut parameter_position: u32 = 0;

    loop {
        if parser_ctx.check(TokenType::RParen) {
            break;
        }

        let is_mutable: bool = parser_ctx.match_token(TokenType::Mut)?;

        let parameter_tk: &Token = parser_ctx.consume(
            TokenType::Identifier,
            String::from("Syntax error"),
            String::from("Expected parameter name."),
        )?;

        let name: &str = parameter_tk.get_lexeme();
        let ascii_name: &str = parameter_tk.get_ascii_lexeme();

        let span: Span = parameter_tk.get_span();

        parser_ctx.consume(
            TokenType::Colon,
            String::from("Syntax error"),
            String::from("Expected ':'."),
        )?;

        let parameter_type: ThrushType = typegen::build_type(parser_ctx)?;

        parameters_types.push(parameter_type.clone());

        parameters.push(Ast::FunctionParameter {
            name,
            ascii_name,
            kind: parameter_type,
            position: parameter_position,
            is_mutable,
            span,
        });

        parameter_position += 1;

        if parser_ctx.check(TokenType::RParen) {
            break;
        } else {
            parser_ctx.consume(
                TokenType::Comma,
                String::from("Syntax error"),
                String::from("Expected ','."),
            )?;
        }
    }

    parser_ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    let return_type: ThrushType = typegen::build_type(parser_ctx)?;

    let function_attributes: ThrushAttributes =
        attributes::build_attributes(parser_ctx, &[TokenType::SemiColon, TokenType::LBrace])?;

    let function_has_ignore: bool = function_attributes.has_ignore_attribute();

    let mut function: Ast = Ast::Function {
        name: function_name,
        ascii_name: function_ascii_name,
        parameters: parameters.clone(),
        parameter_types: parameters_types.clone(),
        body: Ast::Null { span }.into(),
        return_type: return_type.clone(),
        attributes: function_attributes,
        span,
    };

    if declare_forward {
        if let Err(error) = parser_ctx.get_mut_symbols().new_function(
            function_name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                function_has_ignore,
            ),
            span,
        ) {
            parser_ctx.add_error(error);
        }

        if parser_ctx.match_token(TokenType::SemiColon)? {
            return Ok(function);
        }

        return Ok(Ast::Null { span });
    }

    if parser_ctx.match_token(TokenType::SemiColon)? {
        return Ok(function);
    }

    parser_ctx.get_mut_control_ctx().set_inside_function(true);

    parser_ctx
        .get_mut_type_ctx()
        .set_function_type(return_type.clone());

    parser_ctx.get_mut_symbols().start_parameters(&parameters)?;

    let function_body: Ast = block::build_block(parser_ctx)?;

    parser_ctx.get_mut_symbols().end_parameters();
    parser_ctx.get_mut_control_ctx().set_inside_function(false);

    if let Ast::Function { body, .. } = &mut function {
        *body = function_body.into();
    }

    Ok(function)
}
