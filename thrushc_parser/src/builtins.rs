use thrushc_ast::{Ast, builitins::ThrushBuiltin};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::Type;

use crate::{ParserContext, expressions, typegen};

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
            let span: Span = ctx.advance()?.get_span();

            ctx.add_error(CompilationIssue::Error(
                CompilationIssueCode::E0003,
                format!("Unknown '{}' compiler builtin.", span),
                None,
                span,
            ));

            Ok(Ast::invalid_ast(span))
        }
    }
}

pub fn build_halloc<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let halloc_tk: &Token = ctx.consume(
        TokenType::Halloc,
        CompilationIssueCode::E0001,
        "Expected 'halloc' keyword.".into(),
    )?;

    let span: Span = halloc_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx, true)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::Halloc {
            of: of.clone(),
            span,
        },
        kind: Type::Ptr(Some(of.into()), span),
        span,
    })
}

pub fn build_memcpy<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let memcpy_tk: &Token = ctx.consume(
        TokenType::MemCpy,
        CompilationIssueCode::E0001,
        String::from("Expected 'memcpy' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        "Expected ','.".into(),
    )?;

    let destination: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        "Expected ','.".into(),
    )?;

    let size: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::MemCpy {
            src: source.into(),
            dst: destination.into(),
            size: size.into(),
            span,
        },
        kind: Type::Ptr(None, span),
        span,
    })
}

pub fn build_memmove<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let memcpy_tk: &Token = ctx.consume(
        TokenType::MemMove,
        CompilationIssueCode::E0001,
        String::from("Expected 'memmove' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        String::from("Expected ','."),
    )?;

    let destination: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        String::from("Expected ','."),
    )?;

    let size: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::MemMove {
            src: source.into(),
            dst: destination.into(),
            size: size.into(),
            span,
        },
        kind: Type::Ptr(None, span),
        span,
    })
}

pub fn build_memset<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let memcpy_tk: &Token = ctx.consume(
        TokenType::MemSet,
        CompilationIssueCode::E0001,
        String::from("Expected 'memset' keyword."),
    )?;

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        String::from("Expected '('."),
    )?;

    let span: Span = memcpy_tk.get_span();

    let destination: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        String::from("Expected ','."),
    )?;

    let new_size: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::Comma,
        CompilationIssueCode::E0001,
        String::from("Expected ','."),
    )?;

    let size: Ast = expressions::build_expr(ctx)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        String::from("Expected ')'."),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::MemSet {
            dst: destination.into(),
            new_size: new_size.into(),
            size: size.into(),
            span,
        },
        kind: Type::Ptr(None, span),
        span,
    })
}

pub fn build_alignof<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AlignOf,
        CompilationIssueCode::E0001,
        "Expected 'alignof' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx, true)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::AlignOf { of, span },
        kind: Type::U32(span),
        span,
    })
}

pub fn build_sizeof<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::SizeOf,
        CompilationIssueCode::E0001,
        String::from("Expected 'sizeof' keyword."),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx, true)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::SizeOf { of, span },
        kind: Type::USize(span),
        span,
    })
}

pub fn build_bit_size_of<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::BitSizeOf,
        CompilationIssueCode::E0001,
        String::from("Expected 'bit_size_of' keyword."),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx, true)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::BitSizeOf { of, span },
        kind: Type::U64(span),
        span,
    })
}

pub fn build_abi_size_of<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AbiSizeOf,
        CompilationIssueCode::E0001,
        "Expected 'abi_size_of' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx, true)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::AbiSizeOf { of, span },
        kind: Type::U64(span),
        span,
    })
}

pub fn build_abi_align_of<'parser>(
    ctx: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let sizeof_tk: &Token = ctx.consume(
        TokenType::AbiAlignOf,
        CompilationIssueCode::E0001,
        "Expected 'abi_align_of' keyword.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    ctx.consume(
        TokenType::LParen,
        CompilationIssueCode::E0001,
        "Expected '('.".into(),
    )?;

    let of: Type = typegen::build_type(ctx, true)?;

    ctx.consume(
        TokenType::RParen,
        CompilationIssueCode::E0001,
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: ThrushBuiltin::AbiAlignOf { of, span },
        kind: Type::U32(span),
        span,
    })
}
