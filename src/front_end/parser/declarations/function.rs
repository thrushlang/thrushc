use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::core::errors::standard::CompilationIssueCode;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::attributes;
use crate::front_end::parser::statements::block;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::metadata::fnparam::FunctionParameterMetadata;
use crate::front_end::types::lexer::traits::TokenTypeExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::symbols::types::ParametersTypes;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

pub fn build_function<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::Fn,
        CompilationIssueCode::E0001,
        "Expected 'fn' keyword.".into(),
    )?;

    let function_name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    let name: &str = function_name_tk.get_lexeme();

    let ascii_name: &str = function_name_tk.get_ascii_lexeme();

    let span: Span = function_name_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
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
            CompilationIssueCode::E0001,
            "Expected 'identifier'.".into(),
        )?;

        let name: &str = parameter_name_tk.get_lexeme();
        let ascii_name: &str = parameter_name_tk.get_ascii_lexeme();
        let span: Span = parameter_name_tk.get_span();

        ctx.consume(
            TokenType::Colon,
            CompilationIssueCode::E0001,
            "Expected ':'.".into(),
        )?;

        let parameter_type: Type = typegen::build_type(ctx, false)?;

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
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }
    }

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    let return_type: Type = if ctx.check(TokenType::LBrace) || ctx.peek().get_type().is_attribute()
    {
        let peeked: &Token = ctx.peek();
        let peeked_type: TokenType = peeked.get_type();

        let span: Span = if peeked_type.is_attribute() {
            peeked.get_span()
        } else {
            ctx.previous().get_span()
        };

        Type::Void(span)
    } else {
        typegen::build_type(ctx, false)?
    };

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

    ctx.get_mut_symbols().declare_parameters(&parameters)?;

    let function_body: Ast = block::build_block(ctx)?;

    ctx.get_mut_symbols().finish_parameters();

    if let Ast::Function { body, .. } = &mut function {
        *body = Some(function_body.into());
    }

    Ok(function)
}
