use crate::{
    backends::classical::llvm::compiler::builtins::Builtin,
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::{span::Span, token::Token, tokentype::TokenType},
        parser::{ParserContext, expr, expressions::reference, typegen},
        types::{ast::Ast, parser::stmts::traits::TokenExtensions},
        typesystem::types::Type,
    },
};

pub fn build_halloc<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let memcpy_tk: &Token = parser_context.consume(
        TokenType::Halloc,
        "Syntax error".into(),
        "Expected 'halloc' keyword.".into(),
    )?;

    parser_context.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let span: Span = memcpy_tk.get_span();

    let alloc: Type = typegen::build_type(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::Halloc {
            alloc: alloc.clone(),
        },
        kind: Type::Ptr(Some(alloc.into())),
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
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let span: Span = memcpy_tk.get_span();

    let source: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let destination: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        "Syntax error".into(),
        "Expected ','.".into(),
    )?;

    let size: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::MemCpy {
            source: source.into(),
            destination: destination.into(),
            size: size.into(),
        },
        kind: Type::Ptr(None),
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

    let source: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let destination: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expr::build_expr(parser_context)?;

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
        kind: Type::Ptr(None),
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

    let destination: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let new_size: Ast = expr::build_expr(parser_context)?;

    parser_context.consume(
        TokenType::Comma,
        String::from("Syntax error"),
        String::from("Expected ','."),
    )?;

    let size: Ast = expr::build_expr(parser_context)?;

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
        kind: Type::Ptr(None),
        span,
    })
}

pub fn build_alignof<'parser>(
    parser_context: &mut ParserContext<'parser>,
) -> Result<Ast<'parser>, ThrushCompilerIssue> {
    let sizeof_tk: &Token = parser_context.consume(
        TokenType::AlignOf,
        "Syntax error".into(),
        "Expected 'alignof' keyword.".into(),
    )?;

    parser_context.consume(
        TokenType::LParen,
        "Syntax error".into(),
        "Expected '('.".into(),
    )?;

    let span: Span = sizeof_tk.get_span();

    if parser_context.match_token(TokenType::Identifier)? {
        let identifier_tk: &Token = parser_context.previous();

        let name: &str = identifier_tk.get_lexeme();
        let span: Span = identifier_tk.get_span();

        let reference: Ast = reference::build_reference(parser_context, name, span)?;

        let reference_type: &Type = reference.get_value_type()?;

        parser_context.consume(
            TokenType::RParen,
            String::from("Syntax error"),
            String::from("Expected ')'."),
        )?;

        return Ok(Ast::Builtin {
            builtin: Builtin::AlignOf {
                align_of: reference_type.clone(),
            },
            kind: Type::Ptr(None),
            span,
        });
    }

    let alignof_type: Type = typegen::build_type(parser_context)?;

    parser_context.consume(
        TokenType::RParen,
        "Syntax error".into(),
        "Expected ')'.".into(),
    )?;

    Ok(Ast::Builtin {
        builtin: Builtin::AlignOf {
            align_of: alignof_type,
        },
        kind: Type::Ptr(None),
        span,
    })
}
