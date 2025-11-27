use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::{attributes, checks, expr, typegen};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::attributes::traits::ThrushAttributesExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::stmts::types::ThrushAttributes;
use crate::front_end::types::parser::symbols::types::ParametersTypes;
use crate::front_end::typesystem::types::Type;

pub fn build_assembler_function<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    checks::check_main_scope_state(ctx)?;

    ctx.consume(
        TokenType::AsmFn,
        "Syntax error".into(),
        "Expected 'asmfn' keyword.".into(),
    )?;

    let asm_function_name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        "Syntax error".into(),
        "Expected identifier.".into(),
    )?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let asm_function_name: &str = asm_function_name_tk.get_lexeme();
    let asm_function_ascii_name: &str = asm_function_name_tk.get_ascii_lexeme();

    let span: Span = asm_function_name_tk.get_span();

    let mut parameters: Vec<Ast> = Vec::with_capacity(10);
    let mut parameters_types: Vec<Type> = Vec::with_capacity(10);

    let mut parameter_position: u32 = 0;

    loop {
        if ctx.check(TokenType::RParen) {
            break;
        }

        let parameter_name_tk: &Token = ctx.consume(
            TokenType::Identifier,
            "Syntax error".into(),
            "Expected 'identifier'.".into(),
        )?;

        let parameter_name: &str = parameter_name_tk.get_lexeme();
        let parameter_span: Span = parameter_name_tk.get_span();

        let parameter_type: Type = typegen::build_type(ctx)?;

        parameters_types.push(parameter_type.clone());

        parameters.push(Ast::AssemblerFunctionParameter {
            name: parameter_name,
            kind: parameter_type,
            position: parameter_position,
            span: parameter_span,
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

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;

    let is_public: bool = attributes.has_public_attribute();

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut assembler: String = String::with_capacity(100);
    let mut assembler_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expr::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        let assembly: &str = raw_str.get_str_literal_content(raw_str_span)?;

        if assembler_pos != 0 {
            assembler.push('\n');
        }

        assembler.push_str(assembly);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }

        assembler_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    ctx.consume(
        TokenType::LBrace,
        "Syntax error".into(),
        "Expected '{'.".into(),
    )?;

    let mut constraints: String = String::with_capacity(100);
    let mut constraint_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expr::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        let constraint: &str = raw_str.get_str_literal_content(raw_str_span)?;

        if constraint_pos != 0 {
            constraints.push('\n');
        }

        constraints.push_str(constraint);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                "Syntax error".into(),
                "Expected ','.".into(),
            )?;
        }

        constraint_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        "Syntax error".into(),
        "Expected '}'.".into(),
    )?;

    if declare_forward {
        ctx.get_mut_symbols().new_asm_function(
            asm_function_name,
            (
                return_type,
                ParametersTypes::new(parameters_types),
                is_public,
            ),
            span,
        )?;

        return Ok(Ast::new_nullptr(span));
    }

    Ok(Ast::AssemblerFunction {
        name: asm_function_name,
        ascii_name: asm_function_ascii_name,
        parameters,
        parameters_types,
        assembler,
        constraints,
        return_type,
        attributes,
        span,
    })
}
