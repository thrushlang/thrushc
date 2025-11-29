use crate::back_end::llvm::compiler::builtins::Builtin;

use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_halloc<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let halloc_tk: &Token = ctx.consume(
        TokenType::Halloc,
        "Syntax error".into(),
        "Expected 'halloc' keyword.".into(),
    )?;

    let span: Span = halloc_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let alloc: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::Halloc {
            alloc: alloc.clone(),
            span,
        },
        kind: Type::Ptr(Some(alloc.into())),
        span,
    })
}

pub fn build_memcpy<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let memcpy_tk: &Token = ctx.consume(
        TokenType::MemCpy,
        String::from("Syntax error"),
        String::from("Expected 'memcpy' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let destination: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let size: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemCpy {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
            span,
        },
        kind: Type::Ptr(None),
        span,
    })
}

pub fn build_memmove<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let memcpy_tk: &Token = ctx.consume(
        TokenType::MemMove,
        String::from("Syntax error"),
        String::from("Expected 'memmove' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let destination: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemMove {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
            span,
        },
        kind: Type::Ptr(None),
        span,
    })
}

pub fn build_memset<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let memcpy_tk: &Token = ctx.consume(
        TokenType::MemSet,
        String::from("Syntax error"),
        String::from("Expected 'memset' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        String::from("Syntax error"),
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let destination: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let new_size: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expr::build_expr(ctx)?;

    ctx.consume(
        TokenType::RParen,
        String::from("Syntax error"),
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemSet {
            destination: destination.into(),
            new_size: new_size.into(),
            size: size.into(),
            span,
        },
        kind: Type::Ptr(None),
        span,
    })
}

pub fn build_alignof<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AlignOf,
        "Syntax error".into(),
        "Expected 'alignof' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let alignof_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::AlignOf {
            align_of: alignof_type,
            span,
        },
        kind: Type::U32,
        span,
    })
}

pub fn build_sizeof<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::SizeOf,
        String::from("Syntax error"),
        String::from("Expected 'sizeof' keyword."),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let sizeof_type: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::SizeOf {
            size_of: sizeof_type,
            span,
        },
        kind: Type::U64,
        span,
    })
}
