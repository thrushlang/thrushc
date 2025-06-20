use crate::{
    backend::llvm::compiler::builtins::Builtin,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{
            lexer::ThrushType,
            parser::stmts::{stmt::ThrushStatement, traits::TokenExtensions},
        },
    },
};

pub fn build_sqrt<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let sqrt_tk: &Token = parser_context.consume(
        TokenType::Sqrt,
        String::from("Syntax error"),
        String::from("Expected 'sqrt' keyword."),
    )?;

    parser_context.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let span: Span = sqrt_tk.get_span();

    let value: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ThrushStatement::Builtin {
        builtin: Builtin::Sqrt {
            value: value.into(),
        },
        kind: ThrushType::F64,
        span,
    })
}

pub fn build_memcpy<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let memcpy_tk: &Token = parser_context.consume(
        TokenType::MemCpy,
        String::from("Syntax error"),
        String::from("Expected 'memcpy' keyword."),
    )?;

    parser_context.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let destination: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ThrushStatement::Builtin {
        builtin: Builtin::MemCpy {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
        },
        kind: ThrushType::Ptr(None),
        span,
    })
}

pub fn build_memmove<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let memcpy_tk: &Token = parser_context.consume(
        TokenType::MemMove,
        String::from("Syntax error"),
        String::from("Expected 'memmove' keyword."),
    )?;

    parser_context.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let destination: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ThrushStatement::Builtin {
        builtin: Builtin::MemMove {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
        },
        kind: ThrushType::Ptr(None),
        span,
    })
}

pub fn build_memset<'instr>(
    parser_context: &mut ParserContext<'instr>,
) -> Result<ThrushStatement<'instr>, ThrushCompilerIssue> {
    let memcpy_tk: &Token = parser_context.consume(
        TokenType::MemSet,
        String::from("Syntax error"),
        String::from("Expected 'memset' keyword."),
    )?;

    parser_context.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let destination: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let new_size: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: ThrushStatement = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(ThrushStatement::Builtin {
        builtin: Builtin::MemSet {
            destination: destination.into(),
            new_size: new_size.into(),
            size: size.into(),
        },
        kind: ThrushType::Ptr(None),
        span,
    })
}
