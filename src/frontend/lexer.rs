use {
    super::{
        super::{
            backend::compiler::misc::ThrushFile, constants::MINIMAL_ERROR_CAPACITY,
            diagnostic::Diagnostic, error::ThrushCompilerError, logging::LogType,
        },
        traits::TokenLexemeBasics,
        types::TokenLexeme,
    },
    ahash::{HashMap, HashMapExt},
    inkwell::{FloatPredicate, IntPredicate},
    lazy_static::lazy_static,
    std::{mem, process::exit},
};

const KEYWORDS_CAPACITY: usize = 43;
const MINIMAL_TOKENS_CAPACITY: usize = 100_000;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static [u8], TokenKind> = {
        let mut keywords: HashMap<&'static [u8], TokenKind> =
            HashMap::with_capacity(KEYWORDS_CAPACITY);

        keywords.insert(b"local", TokenKind::Local);
        keywords.insert(b"fn", TokenKind::Fn);
        keywords.insert(b"if", TokenKind::If);
        keywords.insert(b"elif", TokenKind::Elif);
        keywords.insert(b"else", TokenKind::Else);
        keywords.insert(b"for", TokenKind::For);
        keywords.insert(b"while", TokenKind::While);
        keywords.insert(b"loop", TokenKind::Loop);
        keywords.insert(b"true", TokenKind::True);
        keywords.insert(b"false", TokenKind::False);
        keywords.insert(b"or", TokenKind::Or);
        keywords.insert(b"and", TokenKind::And);
        keywords.insert(b"const", TokenKind::Const);
        keywords.insert(b"struct", TokenKind::Struct);
        keywords.insert(b"return", TokenKind::Return);
        keywords.insert(b"break", TokenKind::Break);
        keywords.insert(b"continue", TokenKind::Continue);
        keywords.insert(b"this", TokenKind::This);
        keywords.insert(b"builtin", TokenKind::Builtin);
        keywords.insert(b"match", TokenKind::Match);
        keywords.insert(b"pattern", TokenKind::Pattern);
        keywords.insert(b"@import", TokenKind::Import);
        keywords.insert(b"@public", TokenKind::Public);
        keywords.insert(b"@extern", TokenKind::Extern);
        keywords.insert(b"@ignore", TokenKind::Ignore);
        keywords.insert(b"new", TokenKind::New);
        keywords.insert(b"nullptr", TokenKind::NullPtr);
        keywords.insert(b"s8", TokenKind::DataType(Type::S8));
        keywords.insert(b"s16", TokenKind::DataType(Type::S16));
        keywords.insert(b"s32", TokenKind::DataType(Type::S32));
        keywords.insert(b"s64", TokenKind::DataType(Type::S64));
        keywords.insert(b"u8", TokenKind::DataType(Type::U8));
        keywords.insert(b"u16", TokenKind::DataType(Type::U16));
        keywords.insert(b"u32", TokenKind::DataType(Type::U32));
        keywords.insert(b"u64", TokenKind::DataType(Type::U64));
        keywords.insert(b"f32", TokenKind::DataType(Type::F32));
        keywords.insert(b"f64", TokenKind::DataType(Type::F64));
        keywords.insert(b"bool", TokenKind::DataType(Type::Bool));
        keywords.insert(b"char", TokenKind::DataType(Type::Char));
        keywords.insert(b"ptr", TokenKind::DataType(Type::Ptr));
        keywords.insert(b"T", TokenKind::DataType(Type::Generic));
        keywords.insert(b"str", TokenKind::DataType(Type::Str));
        keywords.insert(b"void", TokenKind::DataType(Type::Void));

        keywords
    };
}

pub struct Lexer<'a> {
    tokens: Vec<Token<'a>>,
    errors: Vec<ThrushCompilerError>,
    code: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
    span: (usize, usize),
    diagnostic: Diagnostic,
}

impl<'a> Lexer<'a> {
    pub fn lex(code: &'a [u8], file: &'a ThrushFile) -> Vec<Token<'a>> {
        let mut lexer: Lexer = Self {
            tokens: Vec::with_capacity(MINIMAL_TOKENS_CAPACITY),
            errors: Vec::with_capacity(MINIMAL_ERROR_CAPACITY),
            code,
            start: 0,
            current: 0,
            line: 1,
            span: (0, 0),
            diagnostic: Diagnostic::new(file),
        };

        lexer._lex()
    }

    fn _lex(&mut self) -> Vec<Token<'a>> {
        while !self.end() {
            self.start = self.current;
            self.start_span();

            if let Err(error) = self.scan() {
                self.errors.push(error)
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostic.report(error, LogType::Error);
            });

            exit(1);
        };

        self.tokens.push(Token {
            lexeme: b"",
            kind: TokenKind::Eof,
            line: self.line,
            span: self.span,
        });

        mem::take(&mut self.tokens)
    }

    fn scan(&mut self) -> Result<(), ThrushCompilerError> {
        match self.advance() {
            b'[' => self.make(TokenKind::LBracket),
            b']' => self.make(TokenKind::RBracket),
            b'(' => self.make(TokenKind::LParen),
            b')' => self.make(TokenKind::RParen),
            b'{' => self.make(TokenKind::LBrace),
            b'}' => self.make(TokenKind::RBrace),
            b',' => self.make(TokenKind::Comma),
            b'.' if self.char_match(b'.') => self.make(TokenKind::Range),
            b'.' => self.make(TokenKind::Dot),
            b'%' => self.make(TokenKind::Arith),
            b'*' => self.make(TokenKind::Star),
            b'/' if self.char_match(b'/') => loop {
                if self.peek() == b'\n' || self.end() {
                    break;
                }

                self.advance();
            },
            b'/' if self.char_match(b'*') => loop {
                if self.char_match(b'*') && self.char_match(b'/') {
                    break;
                } else if self.end() {
                    self.end_span();

                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax Error"),
                        String::from(
                            "Unterminated multiline comment. Did you forget to close the comment with a '*/'?",
                        ),
                        self.line,
                        Some(self.span),
                    ));
                }

                self.advance();
            },
            b'/' => self.make(TokenKind::Slash),
            b';' => self.make(TokenKind::SemiColon),
            b'-' if self.char_match(b'-') => self.make(TokenKind::MinusMinus),
            b'-' => self.make(TokenKind::Minus),
            b'+' if self.char_match(b'+') => self.make(TokenKind::PlusPlus),
            b'+' => self.make(TokenKind::Plus),
            b':' if self.char_match(b':') => self.make(TokenKind::ColonColon),
            b':' => self.make(TokenKind::Colon),
            b'!' if self.char_match(b'=') => self.make(TokenKind::BangEq),
            b'!' => self.make(TokenKind::Bang),
            b'=' if self.char_match(b'=') => self.make(TokenKind::EqEq),
            b'=' => self.make(TokenKind::Eq),
            b'<' if self.char_match(b'=') => self.make(TokenKind::LessEq),
            b'<' if self.char_match(b'<') => self.make(TokenKind::LShift),
            b'<' => self.make(TokenKind::Less),
            b'>' if self.char_match(b'=') => self.make(TokenKind::GreaterEq),
            b'>' if self.char_match(b'>') => self.make(TokenKind::RShift),
            b'>' => self.make(TokenKind::Greater),
            b'|' if self.char_match(b'|') => self.make(TokenKind::Or),
            b'&' if self.char_match(b'&') => self.make(TokenKind::And),
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'\'' => self.char()?,
            b'"' => self.string()?,
            b'0'..=b'9' => self.integer_or_float()?,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'@' => self.identifier()?,
            _ => {
                self.end_span();

                return Err(ThrushCompilerError::Error(
                    String::from("Unknown character."),
                    String::from("Did you provide a valid character?"),
                    self.line,
                    Some(self.span),
                ));
            }
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ThrushCompilerError> {
        while self.is_alpha(self.peek())
            || self.peek().is_ascii_digit()
            || self.peek() == b'!' && self.peek() != b':'
        {
            self.advance();
        }

        let code: &[u8] = &self.code[self.start..self.current];

        if let Some(token_type) = KEYWORDS.get(code) {
            self.make(*token_type);
        } else {
            self.make(TokenKind::Identifier);
        }

        Ok(())
    }

    fn integer_or_float(&mut self) -> Result<(), ThrushCompilerError> {
        let mut is_hexadecimal: bool = false;
        let mut is_binary: bool = false;

        while self.peek().is_ascii_digit()
            || self.peek() == b'_' && self.peek_next().is_ascii_digit()
            || self.peek() == b'.' && self.peek_next().is_ascii_digit()
            || self.peek() == b'x' && self.peek_next().is_ascii_digit()
            || self.peek() == b'b' && self.peek_next().is_ascii_digit()
        {
            if is_hexadecimal && self.previous() == b'0' && self.peek() == b'x' {
                self.end_span();

                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("The hexadecimal identifier '0x' cannot be repeated."),
                    self.line,
                    Some(self.span),
                ));
            }

            if is_binary && self.previous() == b'0' && self.peek() == b'b' {
                self.end_span();

                return Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("The binary identifier '0b' cannot be repeated."),
                    self.line,
                    Some(self.span),
                ));
            }

            if self.peek() == b'x' && self.peek_next().is_ascii_digit() {
                is_hexadecimal = true;
            }

            if self.peek() == b'b' && self.peek_next().is_ascii_digit() {
                is_binary = true;
            }

            self.advance();
        }

        self.end_span();

        let parsed_number: (Type, f64) = self.parse_float_or_integer(self.lexeme().to_str())?;

        if parsed_number.0.is_float_type() {
            self.tokens.push(Token {
                kind: TokenKind::Float(parsed_number.0, parsed_number.1, false),
                lexeme: b"",
                line: self.line,
                span: self.span,
            });

            return Ok(());
        }

        self.tokens.push(Token {
            kind: TokenKind::Integer(parsed_number.0, parsed_number.1, false),
            lexeme: b"",
            line: self.line,
            span: self.span,
        });

        Ok(())
    }

    #[inline]
    fn parse_float_or_integer(&mut self, lexeme: &str) -> Result<(Type, f64), ThrushCompilerError> {
        if lexeme.contains('.') {
            return self.parse_float(lexeme);
        }

        self.parse_integer(lexeme)
    }

    #[inline]
    fn parse_float(&self, lexeme: &str) -> Result<(Type, f64), ThrushCompilerError> {
        let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

        if dot_count > 1 {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Float values should only contain one dot."),
                self.line,
                Some(self.span),
            ));
        }

        if let Ok(float) = lexeme.parse::<f32>() {
            return Ok((Type::F32, float as f64));
        }

        if let Ok(float) = lexeme.parse::<f64>() {
            return Ok((Type::F64, float));
        }

        Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            String::from("Out of bounds."),
            self.line,
            Some(self.span),
        ))
    }

    #[inline]
    fn parse_integer(&self, lexeme: &str) -> Result<(Type, f64), ThrushCompilerError> {
        const I8_MIN: isize = -128;
        const I8_MAX: isize = 127;
        const I16_MIN: isize = -32768;
        const I16_MAX: isize = 32767;
        const I32_MIN: isize = -2147483648;
        const I32_MAX: isize = 2147483647;

        const U8_MIN: usize = 0;
        const U8_MAX: usize = 255;
        const U16_MIN: usize = 0;
        const U16_MAX: usize = 65535;
        const U32_MIN: usize = 0;
        const U32_MAX: usize = 4294967295;

        if lexeme.starts_with("0x") {
            let cleaned_lexeme: String = lexeme
                .strip_prefix("0x")
                .unwrap_or(&lexeme.replace("0x", ""))
                .replace("_", "");

            return match isize::from_str_radix(&cleaned_lexeme, 16) {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num) {
                        return Ok((Type::S8, num as f64));
                    } else if (I16_MIN..=I16_MAX).contains(&num) {
                        return Ok((Type::S16, num as f64));
                    } else if (I32_MIN..=I32_MAX).contains(&num) {
                        return Ok((Type::S32, num as f64));
                    } else if (isize::MIN..=isize::MAX).contains(&num) {
                        return Ok((Type::S64, num as f64));
                    } else {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            String::from("Invalid hexadecimal format."),
                            self.line,
                            Some(self.span),
                        ));
                    }
                }

                Err(_) => Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Invalid hexadecimal format."),
                    self.line,
                    Some(self.span),
                )),
            };
        }

        if lexeme.starts_with("0b") {
            let cleaned_lexeme: String = lexeme
                .strip_prefix("0b")
                .unwrap_or(&lexeme.replace("0b", ""))
                .replace("_", "");

            return match isize::from_str_radix(&cleaned_lexeme, 2) {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num) {
                        return Ok((Type::S8, num as f64));
                    } else if (I16_MIN..=I16_MAX).contains(&num) {
                        return Ok((Type::S16, num as f64));
                    } else if (I32_MIN..=I32_MAX).contains(&num) {
                        return Ok((Type::S32, num as f64));
                    } else if (isize::MIN..=isize::MAX).contains(&num) {
                        return Ok((Type::S64, num as f64));
                    } else {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            String::from("Invalid binary format."),
                            self.line,
                            Some(self.span),
                        ));
                    }
                }

                Err(_) => Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Invalid binary format."),
                    self.line,
                    Some(self.span),
                )),
            };
        }

        match lexeme.parse::<usize>() {
            Ok(num) => {
                if (U8_MIN..=U8_MAX).contains(&num) {
                    Ok((Type::U8, num as f64))
                } else if (U16_MIN..=U16_MAX).contains(&num) {
                    return Ok((Type::U16, num as f64));
                } else if (U32_MIN..=U32_MAX).contains(&num) {
                    return Ok((Type::U32, num as f64));
                } else if (usize::MIN..=usize::MAX).contains(&num) {
                    return Ok((Type::U64, num as f64));
                } else {
                    return Err(ThrushCompilerError::Error(
                        String::from("Syntax error"),
                        String::from("Out of bounds."),
                        self.line,
                        Some(self.span),
                    ));
                }
            }

            Err(_) => match lexeme.parse::<isize>() {
                Ok(num) => {
                    if (I8_MIN..=I8_MAX).contains(&num) {
                        Ok((Type::S8, num as f64))
                    } else if (I16_MIN..=I16_MAX).contains(&num) {
                        Ok((Type::S16, num as f64))
                    } else if (I32_MIN..=I32_MAX).contains(&num) {
                        Ok((Type::S32, num as f64))
                    } else if (isize::MIN..=isize::MAX).contains(&num) {
                        Ok((Type::S64, num as f64))
                    } else {
                        Err(ThrushCompilerError::Error(
                            String::from("Syntax error"),
                            String::from("Out of bounds."),
                            self.line,
                            Some(self.span),
                        ))
                    }
                }

                Err(_) => Err(ThrushCompilerError::Error(
                    String::from("Syntax error"),
                    String::from("Out of bounds."),
                    self.line,
                    Some(self.span),
                )),
            },
        }
    }

    fn char(&mut self) -> Result<(), ThrushCompilerError> {
        while self.peek() != b'\'' && !self.end() {
            self.advance();
        }

        self.end_span();

        if self.peek() != b'\'' {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("Unclosed char. Did you forget to close the char with a \'?"),
                self.line,
                Some(self.span),
            ));
        }

        self.advance();

        if self.code[self.start + 1..self.current - 1].len() > 1 {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from("A char data type only can contain one character."),
                self.line,
                Some(self.span),
            ));
        }

        self.tokens.push(Token {
            kind: TokenKind::Char,
            lexeme: &self.code[self.start + 1..self.current - 1],
            line: self.line,
            span: self.span,
        });

        Ok(())
    }

    fn string(&mut self) -> Result<(), ThrushCompilerError> {
        while self.is_string_boundary() {
            self.advance();
        }

        self.end_span();

        if self.peek() != b'"' {
            return Err(ThrushCompilerError::Error(
                String::from("Syntax error"),
                String::from(
                    "Unclosed literal string. Did you forget to close the literal string with a '\"'?",
                ),
                self.line,
                Some(self.span),
            ));
        }

        self.advance();

        self.tokens.push(Token {
            kind: TokenKind::Str,
            lexeme: &self.code[self.start + 1..self.current - 1],
            line: self.line,
            span: self.span,
        });

        Ok(())
    }

    fn make(&mut self, kind: TokenKind) {
        self.end_span();

        self.tokens.push(Token {
            kind,
            lexeme: self.lexeme(),
            line: self.line,
            span: self.span,
        });
    }

    fn char_match(&mut self, ch: u8) -> bool {
        if !self.end() && self.code[self.current] == ch {
            self.current += 1;
            return true;
        }

        false
    }

    fn advance(&mut self) -> u8 {
        let char: u8 = self.code[self.current];
        self.current += 1;

        char
    }

    #[inline]
    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.code.len() {
            return b'\0';
        }

        self.code[self.current + 1]
    }

    #[inline]
    fn previous(&self) -> u8 {
        self.code[self.current - 1]
    }

    #[inline]
    fn peek(&self) -> u8 {
        if self.end() {
            return b'\0';
        }

        self.code[self.current]
    }

    #[inline(always)]
    fn lexeme(&self) -> TokenLexeme<'a> {
        &self.code[self.start..self.current]
    }

    #[inline(always)]
    fn start_span(&mut self) {
        self.span.0 = self.start;
    }

    #[inline(always)]
    fn end_span(&mut self) {
        self.span.1 = self.current;
    }

    #[inline(always)]
    fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[inline(always)]
    fn is_string_boundary(&self) -> bool {
        self.peek() != b'"' && !self.end()
    }

    #[inline(always)]
    const fn is_alpha(&self, char: u8) -> bool {
        char.is_ascii_lowercase() || char.is_ascii_uppercase() || char == b'_'
    }
}

#[derive(Debug, Clone)]
pub struct Token<'token> {
    pub lexeme: &'token [u8],
    pub kind: TokenKind,
    pub line: usize,
    pub span: (usize, usize),
}

impl TokenLexemeBasics for TokenLexeme<'_> {
    #[inline(always)]
    fn to_str(&self) -> &str {
        core::str::from_utf8(self).unwrap_or("invalid utf-8")
    }

    #[inline(always)]
    fn to_string(&self) -> String {
        self.to_str().to_string()
    }

    fn parse_scapes(
        &self,
        line: usize,
        span: (usize, usize),
    ) -> Result<Vec<u8>, ThrushCompilerError> {
        let mut parsed_string: Vec<u8> = Vec::with_capacity(self.len());

        let mut i: usize = 0;

        while i < self.len() {
            if self[i] == b'\\' {
                i += 1;

                match self.get(i) {
                    Some(b'n') => parsed_string.push(b'\n'),
                    Some(b't') => parsed_string.push(b'\t'),
                    Some(b'r') => parsed_string.push(b'\r'),
                    Some(b'\\') => parsed_string.push(b'\\'),
                    Some(b'0') => parsed_string.push(b'\0'),
                    Some(b'\'') => parsed_string.push(b'\''),
                    Some(b'"') => parsed_string.push(b'"'),
                    _ => {
                        return Err(ThrushCompilerError::Error(
                            String::from("Syntax Error"),
                            String::from("Invalid escape sequence."),
                            line,
                            Some(span),
                        ));
                    }
                }

                i += 1;

                continue;
            }

            parsed_string.push(self[i]);

            i += 1;
        }

        Ok(parsed_string)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    // --- Operators ---
    LParen,     // ' ( '
    RParen,     // ' ) '
    LBrace,     // ' { '
    RBrace,     // ' } '
    Comma,      // ' , '
    Dot,        // ' . '
    Minus,      // ' - '
    Plus,       // ' + '
    Slash,      // ' / '
    Star,       // ' * '
    Colon,      // ' : '
    SemiColon,  // ' ; '
    RBracket,   // ' ] '
    LBracket,   // ' [ '
    Arith,      // ' % ',
    Bang,       // ' ! '
    Range,      // ' .. '
    ColonColon, // ' :: '
    BangEq,     // ' != '
    Eq,         // ' = '
    EqEq,       // ' == '
    Greater,    // ' > '
    GreaterEq,  // ' >= '
    Less,       // ' < '
    LessEq,     // ' <= '
    PlusPlus,   // ' ++ '
    MinusMinus, // ' -- '
    LShift,     // ' << '
    RShift,     // ' >> '

    // --- Literals ---
    Identifier,
    Integer(Type, f64, bool),
    Float(Type, f64, bool),
    DataType(Type),
    Str,
    Char,

    // --- Attributes ---
    Extern,
    Ignore,
    Public,

    // --- Keywords ---
    New,
    Import,
    Builtin,
    And,
    Struct,
    Else,
    False,
    Fn,
    For,
    Continue,
    Break,
    Match,
    Pattern,
    If,
    Elif,
    NullPtr,
    Or,
    Return,
    This,
    True,
    Local,
    Const,
    While,
    Loop,

    Eof,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::SemiColon => write!(f, ";"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Arith => write!(f, "%"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Range => write!(f, ".."),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::BangEq => write!(f, "!="),
            TokenKind::Eq => write!(f, "="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::GreaterEq => write!(f, ">="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::LessEq => write!(f, "<="),
            TokenKind::PlusPlus => write!(f, "++"),
            TokenKind::MinusMinus => write!(f, "--"),
            TokenKind::LShift => write!(f, "<<"),
            TokenKind::RShift => write!(f, ">>"),
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Pattern => write!(f, "pattern"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::NullPtr => write!(f, "null"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::This => write!(f, "this"),
            TokenKind::True => write!(f, "true"),
            TokenKind::Local => write!(f, "local"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Integer(datatype, _, _) => write!(f, "{}", datatype),
            TokenKind::Float(datatype, _, _) => write!(f, "{}", datatype),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::Builtin => write!(f, "built-in"),
            TokenKind::Public => write!(f, "@public"),
            TokenKind::Ignore => write!(f, "@ignore"),
            TokenKind::Extern => write!(f, "@extern"),
            TokenKind::Import => write!(f, "@import"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::DataType(datatype) => write!(f, "{}", datatype),
        }
    }
}

impl TokenKind {
    #[inline(always)]
    pub const fn as_int_predicate(&self, left_signed: bool, right_signed: bool) -> IntPredicate {
        match self {
            TokenKind::EqEq => IntPredicate::EQ,
            TokenKind::BangEq => IntPredicate::NE,
            TokenKind::Greater if !left_signed && !right_signed => IntPredicate::UGT,
            TokenKind::Greater if left_signed | !right_signed => IntPredicate::SGT,
            TokenKind::Greater if !left_signed && right_signed => IntPredicate::SGT,
            TokenKind::Greater if left_signed && right_signed => IntPredicate::SGT,
            TokenKind::GreaterEq if !left_signed && !right_signed => IntPredicate::UGE,
            TokenKind::GreaterEq if left_signed && !right_signed => IntPredicate::SGE,
            TokenKind::GreaterEq if !left_signed && right_signed => IntPredicate::SGE,
            TokenKind::GreaterEq if left_signed && right_signed => IntPredicate::SGE,
            TokenKind::Less if !left_signed && !right_signed => IntPredicate::ULT,
            TokenKind::Less if left_signed && !right_signed => IntPredicate::SLT,
            TokenKind::Less if !left_signed && right_signed => IntPredicate::SLT,
            TokenKind::Less if left_signed && right_signed => IntPredicate::SLT,
            TokenKind::LessEq if !left_signed && !right_signed => IntPredicate::ULE,
            TokenKind::LessEq if left_signed && !right_signed => IntPredicate::SLE,
            TokenKind::LessEq if !left_signed && right_signed => IntPredicate::SLE,
            TokenKind::LessEq if left_signed && right_signed => IntPredicate::SLE,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub const fn as_float_predicate(&self) -> FloatPredicate {
        // ESTABILIZAR ESTA COSA EN EL FUTURO IGUAL QUE LOS INTEGER PREDICATE (DETERMINAR SI TIENE SIGNO Y CAMBIAR EL PREDICATE A CONVENIR)
        match self {
            TokenKind::EqEq => FloatPredicate::OEQ,
            TokenKind::BangEq => FloatPredicate::ONE,
            TokenKind::Greater => FloatPredicate::OGT,
            TokenKind::GreaterEq => FloatPredicate::OGE,
            TokenKind::Less => FloatPredicate::OLT,
            TokenKind::LessEq => FloatPredicate::OLE,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub const fn is_logical_type(&self) -> bool {
        matches!(
            self,
            TokenKind::BangEq
                | TokenKind::EqEq
                | TokenKind::LessEq
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::GreaterEq
        )
    }

    #[inline(always)]
    pub const fn is_logical_gate(&self) -> bool {
        matches!(self, TokenKind::And | TokenKind::Or)
    }

    #[inline(always)]
    pub const fn is_struct_keyword(&self) -> bool {
        matches!(self, TokenKind::Struct)
    }

    #[inline(always)]
    pub const fn is_function_keyword(&self) -> bool {
        matches!(self, TokenKind::Fn)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Type {
    // Signed Integer Type
    S8,
    S16,
    S32,
    S64,

    // Unsigned Integer Type
    U8,
    U16,
    U32,
    U64,

    // Floating Point Type
    F32,
    F64,

    // Boolean DataType
    Bool,

    // Char DataType
    Char,

    // Str DataType
    Str,

    // Pointer DataType
    Ptr,

    // Generic DataType
    Generic,

    // Struct DataType
    Struct,

    // Void DataType
    Void,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::S8 => write!(f, "s8"),
            Type::S16 => write!(f, "s16"),
            Type::S32 => write!(f, "s32"),
            Type::S64 => write!(f, "s64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "str"),
            Type::Char => write!(f, "char"),
            Type::Struct => write!(f, "struct"),
            Type::Ptr => write!(f, "ptr"),
            Type::Generic => write!(f, "generic"),
            Type::Void => write!(f, "void"),
        }
    }
}

impl Type {
    #[inline(always)]
    pub const fn precompute_numeric_type(self, other: Type, default: Type) -> Type {
        match (self, other) {
            (Type::S64, _) | (_, Type::S64) => Type::S64,
            (Type::S32, _) | (_, Type::S32) => Type::S32,
            (Type::S16, _) | (_, Type::S16) => Type::S16,
            (Type::S8, _) | (_, Type::S8) => Type::S8,

            (Type::U64, _) | (_, Type::U64) => Type::U64,
            (Type::U32, _) | (_, Type::U32) => Type::U32,
            (Type::U16, _) | (_, Type::U16) => Type::U16,
            (Type::U8, _) | (_, Type::U8) => Type::U8,

            (Type::F64, _) | (_, Type::F64) => Type::F64,
            (Type::F32, _) | (_, Type::F32) => Type::F32,

            _ => default,
        }
    }

    #[inline(always)]
    pub const fn reverse_signed_integer_type(self) -> Type {
        if self.is_signed_integer_type() {
            return self;
        }

        match self {
            Type::U64 => Type::S64,
            Type::U32 => Type::S32,
            Type::U16 => Type::S16,
            Type::U8 => Type::S8,
            _ => self,
        }
    }

    #[inline(always)]
    pub const fn is_void_type(&self) -> bool {
        matches!(self, Type::Void)
    }

    #[inline(always)]
    pub const fn is_bool_type(&self) -> bool {
        matches!(self, Type::Bool)
    }

    #[inline(always)]
    pub const fn is_struct_type(&self) -> bool {
        matches!(self, Type::Struct)
    }

    #[inline(always)]
    pub const fn is_float_type(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    #[inline(always)]
    pub const fn is_ptr_type(&self) -> bool {
        matches!(self, Type::Struct | Type::Str | Type::Ptr)
    }

    #[inline(always)]
    pub const fn is_heaped_ptr(&self) -> bool {
        matches!(self, Type::Struct | Type::Ptr)
    }

    #[inline(always)]
    pub const fn is_signed_integer_type(&self) -> bool {
        matches!(self, Type::S8 | Type::S16 | Type::S32 | Type::S64)
    }

    #[inline(always)]
    pub const fn is_integer_type(&self) -> bool {
        matches!(
            self,
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Char
        )
    }
}
