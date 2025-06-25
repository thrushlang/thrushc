use crate::{
    backend::llvm::compiler::builtins::Builtin,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expression},
        types::{ast::Ast, lexer::ThrushType, parser::stmts::traits::TokenExtensions},
    },
};

pub fn build_sqrt<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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

    let value: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::Sqrt {
            value: value.into(),
        },
        kind: ThrushType::F64,
        span,
    })
}

pub fn build_memcpy<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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

    let source: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let destination: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemCpy {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
        },
        kind: ThrushType::Ptr(None),
        span,
    })
}

pub fn build_memmove<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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

    let source: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let destination: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemMove {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
        },
        kind: ThrushType::Ptr(None),
        span,
    })
}

pub fn build_memset<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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

    let destination: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let new_size: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expression::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemSet {
            destination: destination.into(),
            new_size: new_size.into(),
            size: size.into(),
        },
        kind: ThrushType::Ptr(None),
        span,
    })
}
