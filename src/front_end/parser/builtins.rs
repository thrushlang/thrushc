use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::CompilationIssue;

use crate::middle_end::mir::builtins::ThrushBuiltin;

use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;
use crate::front_end::parser::ParserContext;
use crate::front_end::parser::expr;
use crate::front_end::parser::typegen;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::parser::stmts::traits::TokenExtensions;
use crate::front_end::typesystem::types::Type;

pub fn build_builtin<'parser>(
    ctx: &mut ParserContext<'parser>,
    tk_type: TokenType,
) -> Result<Ast<'parser>, CompilationIssue> {
    match tk_type {
        TokenType::SizeOf => self::build_sizeof(ctx),
        TokenType::AlignOf => self::build_alignof(ctx),
        TokenType::Halloc => self::build_halloc(ctx),
        TokenType::MemSet => self::build_memset(ctx),
        TokenType::MemMove => self::build_memmove(ctx),
        TokenType::MemCpy => self::build_memcpy(ctx),
        TokenType::AbiSizeOf => self::build_abi_size_of(ctx),
        TokenType::BitSizeOf => self::build_bit_size_of(ctx),
        TokenType::AbiAlignOf => self::build_abi_align_of(ctx),

        _ => {
            let span: Span = ctx.peek().get_span();

            Err(CompilationIssue::Error(
                "Syntax error".into(),
                format!("Unknown '{}' compiler builtin.", span),
                None,
                span,
            ))
        }
    }
}

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
        builtin: ThrushBuiltin::Halloc {
            of: alloc.clone(),
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
        builtin: ThrushBuiltin::MemCpy {
            src: source.into(),
            dst: destination.into(),
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
        builtin: ThrushBuiltin::MemMove {
            src: source.into(),
            dst: destination.into(),
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
        builtin: ThrushBuiltin::MemSet {
            dst: destination.into(),
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

    let of: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::AlignOf { of, span },
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
        builtin: ThrushBuiltin::SizeOf {
            of: sizeof_type,
            span,
        },
        kind: Type::USize,
        span,
    })
}

pub fn build_bit_size_of<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::BitSizeOf,
        String::from("Syntax error"),
        String::from("Expected 'bit_size_of' keyword."),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::BitSizeOf { of, span },
        kind: Type::U64,
        span,
    })
}

pub fn build_abi_size_of<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AbiSizeOf,
        "Syntax error".into(),
        "Expected 'abi_size_of' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::AbiSizeOf { of, span },
        kind: Type::U64,
        span,
    })
}

pub fn build_abi_align_of<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AbiAlignOf,
        "Syntax error".into(),
        "Expected 'abi_align_of' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx)?;

    ctx.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::AbiAlignOf { of, span },
        kind: Type::U32,
        span,
    })
}
