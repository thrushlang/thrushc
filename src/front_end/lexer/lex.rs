use crate::core::errors::standard::ThrushCompilerIssue;

use crate::front_end::lexer::Lexer;
use crate::front_end::lexer::character;
use crate::front_end::lexer::identifier;
use crate::front_end::lexer::number;
use crate::front_end::lexer::span::Span;
use crate::front_end::lexer::string;
use crate::front_end::lexer::tokentype::TokenType;

pub fn analyze(lexer: &mut Lexer) -> Result<(), ThrushCompilerIssue> {
    match lexer.advance() {
        '[' => lexer.make(TokenType::LBracket),
        ']' => lexer.make(TokenType::RBracket),
        '(' => lexer.make(TokenType::LParen),
        ')' => lexer.make(TokenType::RParen),
        '{' => lexer.make(TokenType::LBrace),
        '}' => lexer.make(TokenType::RBrace),
        ',' => lexer.make(TokenType::Comma),
        '.' if lexer.char_match('.') && lexer.char_match('.') => lexer.make(TokenType::Pass),
        '.' if lexer.char_match('.') => lexer.make(TokenType::Range),
        '.' => lexer.make(TokenType::Dot),
        '%' => lexer.make(TokenType::Arith),
        '*' => lexer.make(TokenType::Star),
        '^' => lexer.make(TokenType::Xor),
        '~' => lexer.make(TokenType::Not),
        '/' if lexer.char_match('/') => loop {
            if lexer.peek() == '\n' || lexer.is_eof() {
                break;
            }

            lexer.advance_only();
        },
        '/' if lexer.char_match('*') => loop {
            if lexer.char_match('*') && lexer.char_match('/') {
                break;
            } else if lexer.is_eof() {
                lexer.end_span();

                let span: Span = Span::new(lexer.line, lexer.span);

                return Err(ThrushCompilerIssue::Error(
                    "Syntax error".into(),
                    "Expected '*/'.".into(),
                    None,
                    span,
                ));
            }

            if lexer.peek() == '\n' {
                lexer.line += 1;
            }

            lexer.advance_only();
        },
        '/' => lexer.make(TokenType::Slash),
        ';' => lexer.make(TokenType::SemiColon),
        '-' if lexer.char_match('-') => lexer.make(TokenType::MinusMinus),
        '-' if lexer.char_match('=') => lexer.make(TokenType::MinusEq),
        '-' if lexer.char_match('>') => lexer.make(TokenType::Arrow),
        '-' => lexer.make(TokenType::Minus),
        '+' if lexer.char_match('+') => lexer.make(TokenType::PlusPlus),
        '+' if lexer.char_match('=') => lexer.make(TokenType::PlusEq),
        '+' => lexer.make(TokenType::Plus),
        ':' if lexer.char_match(':') => lexer.make(TokenType::ColonColon),
        ':' => lexer.make(TokenType::Colon),
        '!' if lexer.char_match('=') => lexer.make(TokenType::BangEq),
        '!' => lexer.make(TokenType::Bang),
        '=' if lexer.char_match('=') => lexer.make(TokenType::EqEq),
        '=' => lexer.make(TokenType::Eq),
        '<' if lexer.char_match('=') => lexer.make(TokenType::LessEq),
        '<' if lexer.char_match('<') => lexer.make(TokenType::LShift),
        '<' => lexer.make(TokenType::Less),
        '>' if lexer.char_match('=') => lexer.make(TokenType::GreaterEq),
        '>' if lexer.char_match('>') => lexer.make(TokenType::RShift),
        '>' => lexer.make(TokenType::Greater),

        '|' if lexer.char_match('|') => lexer.make(TokenType::Or),
        '|' => lexer.make(TokenType::Bor),
        '&' if lexer.char_match('&') => lexer.make(TokenType::And),
        '&' => lexer.make(TokenType::BAnd),
        '\r' | '\t' => {}

        ' ' => {
            lexer.start_span();
        }
        '\n' => lexer.line += 1,

        '\'' => character::lex(lexer)?,
        '"' => string::lex(lexer)?,
        '0'..='9' => number::lex(lexer)?,

        identifier if lexer.is_identifier_boundary(identifier) => identifier::lex(lexer)?,

        _ => {
            lexer.end_span();

            let span: Span = Span::new(lexer.line, lexer.span);

            return Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                "This character isn't recognized.".into(),
                None,
                span,
            ));
        }
    }

    Ok(())
}
