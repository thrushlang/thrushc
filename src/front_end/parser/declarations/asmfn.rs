use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::{ParserContext, expressions};
use crate::front_end::parser::{attributes, typegen};
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::{AstCodeLocation, AstStandardExtensions};
use crate::front_end::types::lexer::traits::TokenTypeExtensions;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::types::parser::symbols::types::ParametersTypes;
use crate::front_end::typesystem::types::Type;
use crate::middle_end::mir::attributes::ThrushAttributes;
use crate::middle_end::mir::attributes::traits::ThrushAttributesExtensions;

pub fn build_assembler_function<'parser>(
    ctx: &mut ParserContext<'parser>,
    declare_forward: bool,
) -> Result<Ast<'parser>, CompilationIssue> {
    ctx.consume(
        TokenType::AsmFn,
        CompilationIssueCode::E0001,
        "Expected 'asmfn' keyword.".into(),
    )?;

    let asm_function_name_tk: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected identifier.".into(),
    )?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
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
            CompilationIssueCode::E0001,
            "Expected 'identifier'.".into(),
        )?;

        let parameter_name: &str = parameter_name_tk.get_lexeme();
        let parameter_span: Span = parameter_name_tk.get_span();

        let parameter_type: Type = typegen::build_type(ctx, false)?;

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

    let attributes: ThrushAttributes = attributes::build_attributes(ctx, &[TokenType::LBrace])?;
    let is_public: bool = attributes.has_public_attribute();

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let mut assembler: String = String::with_capacity(100);
    let mut assembler_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expressions::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let assembly: String = if let Ast::Str { bytes, .. } = raw_str {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            String::new()
        };

        if assembler_pos != 0 {
            assembler.push('\n');
        }

        assembler.push_str(&assembly);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }

        assembler_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
        "Expected '}'.".into(),
    )?;

    ctx.consume(
        TokenType::LBrace,
        CompilationIssueCode::E0001,
        "Expected '{'.".into(),
    )?;

    let mut constraints: String = String::with_capacity(100);
    let mut constraint_pos: usize = 0;

    loop {
        if ctx.check(TokenType::RBrace) {
            break;
        }

        let raw_str: Ast = expressions::build_expr(ctx)?;
        let raw_str_span: Span = raw_str.get_span();

        if !raw_str.is_str() {
            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                "Expected string literal value.".into(),
                None,
                raw_str_span,
            ));
        }

        let constraint: String = if let Ast::Str { bytes, .. } = raw_str {
            String::from_utf8_lossy(&bytes).to_string()
        } else {
            String::new()
        };

        if constraint_pos != 0 {
            constraints.push('\n');
        }

        constraints.push_str(&constraint);

        if ctx.check(TokenType::RBrace) {
            break;
        } else {
            ctx.consume(
                TokenType::Comma,
                CompilationIssueCode::E0001,
                "Expected ','.".into(),
            )?;
        }

        constraint_pos += 1;
    }

    ctx.consume(
        TokenType::RBrace,
        CompilationIssueCode::E0001,
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
