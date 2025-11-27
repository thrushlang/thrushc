use crate::core::errors::standard::CompilationIssue;

use crate::front_end::lexer::Lexer;
use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::token::Token;
use crate::front_end::lexer::tokentype::TokenType;

pub mod float;
pub mod integer;

pub fn lex(lexer: &mut Lexer) -> Result<(), CompilationIssue> {
    let mut is_hexadecimal: bool = false;
    let mut is_binary: bool = false;

    while lexer.is_number_boundary(is_hexadecimal, is_binary) {
        if is_hexadecimal && lexer.previous() == '0' && lexer.peek() == 'x' {
            lexer.end_span();

            return Err(CompilationIssue::Error(
                String::from("Syntax error"),
                String::from("Hexadecimal identifier '0x' cannot be repeated."),
                None,
                Span::new(lexer.line, lexer.span),
            ));
        }

        if is_binary && lexer.previous() == '0' && lexer.peek() == 'b' {
            lexer.end_span();

            return Err(CompilationIssue::Error(
                String::from("Syntax error"),
                String::from("Binary identifier '0b' cannot be repeated."),
                None,
                Span::new(lexer.line, lexer.span),
            ));
        }

        if is_hexadecimal && !lexer.peek().is_ascii_alphanumeric() {
            lexer.end_span();
            break;
        }

        if is_binary && !lexer.peek().is_ascii_digit() {
            lexer.end_span();
            break;
        }

        if lexer.peek() == 'x' && lexer.peek_next().is_ascii_alphanumeric() {
            is_hexadecimal = true;
        }

        if lexer.peek() == 'b' && lexer.peek_next().is_ascii_digit() {
            is_binary = true;
        }

        let _ = lexer.advance();
    }

    lexer.end_span();

    let span: Span = Span::new(lexer.line, lexer.span);

    let lexeme: String = lexer.lexeme();

    if lexeme.contains(".") {
        float::check_float_format(lexer, &lexeme)?;

        lexer.tokens.push(Token {
            lexeme,
            ascii: String::default(),
            bytes: Vec::default(),
            kind: TokenType::Float,
            span,
        });

        return Ok(());
    }

    integer::check_integer_format(lexer, &lexeme)?;

    lexer.tokens.push(Token {
        lexeme,
        ascii: String::default(),
        bytes: Vec::default(),
        kind: TokenType::Integer,
        span,
    });

    Ok(())
}
