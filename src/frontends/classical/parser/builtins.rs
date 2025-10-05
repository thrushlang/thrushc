use crate::{
    backends::classical::llvm::compiler::builtins::Builtin,
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_halloc<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let halloc_tk: &Token = ctx.consume(
        TokenType::Halloc,
        "Syntax error".into(),
        "Expected 'halloc' keyword.".into(),
    )?;

    let span: Span = halloc_tk.get_span();

    let alloc: Type = typegen::build_type(ctx)?;

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
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
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
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AlignOf,
        "Syntax error".into(),
        "Expected 'alignof' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    let alignof_type: Type = typegen::build_type(ctx)?;

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
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::SizeOf,
        String::from("Syntax error"),
        String::from("Expected 'sizeof' keyword."),
    )?;

    let span: Span = sizeof_tk.get_span();

    let sizeof_type: Type = typegen::build_type(ctx)?;

    Ok(Ast::Builtin {
        builtin: Builtin::SizeOf {
            size_of: sizeof_type,
            span,
        },
        kind: Type::U64,
        span,
    })
}
