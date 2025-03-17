use {
    super::{
        super::{
            backend::compiler::misc::ThrushFile, constants::MINIMAL_ERROR_CAPACITY,
            diagnostic::Diagnostic, error::ThrushError, logging::LogType,
        },
        traits::TokenLexeme,
    },
    ahash::{HashMap, HashMapExt},
    inkwell::{FloatPredicate, IntPredicate},
    lazy_static::lazy_static,
    std::{mem, num::ParseFloatError, process::exit},
};

const KEYWORDS_CAPACITY: usize = 36;
const MINIMAL_TOKENS_CAPACITY: usize = 100_000;

pub type Lexeme<'a> = &'a [u8];

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
        keywords.insert(b"public", TokenKind::Public);
        keywords.insert(b"builtin", TokenKind::Builtin);
        keywords.insert(b"@import", TokenKind::Import);
        keywords.insert(b"@extern", TokenKind::Extern);
        keywords.insert(b"new", TokenKind::New);
        keywords.insert(b"nullptr", TokenKind::NullPtr);
        keywords.insert(b"i8", TokenKind::DataType(DataTypes::I8));
        keywords.insert(b"i16", TokenKind::DataType(DataTypes::I16));
        keywords.insert(b"i32", TokenKind::DataType(DataTypes::I32));
        keywords.insert(b"i64", TokenKind::DataType(DataTypes::I64));
        keywords.insert(b"f32", TokenKind::DataType(DataTypes::F32));
        keywords.insert(b"f64", TokenKind::DataType(DataTypes::F64));
        keywords.insert(b"bool", TokenKind::DataType(DataTypes::Bool));
        keywords.insert(b"char", TokenKind::DataType(DataTypes::Char));
        keywords.insert(b"ptr", TokenKind::DataType(DataTypes::Ptr));
        keywords.insert(b"T", TokenKind::DataType(DataTypes::Generic));
        keywords.insert(b"str", TokenKind::DataType(DataTypes::Str));
        keywords.insert(b"void", TokenKind::DataType(DataTypes::Void));

        keywords
    };
}

pub struct Lexer<'a> {
    tokens: Vec<Token<'a>>,
    errors: Vec<ThrushError>,
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
                self.diagnostic.report(error, LogType::ERROR);
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

    fn scan(&mut self) -> Result<(), ThrushError> {
        match self.advance() {
            b'[' => self.make(TokenKind::LBracket),
            b']' => self.make(TokenKind::RBracket),
            b'(' => self.make(TokenKind::LParen),
            b')' => self.make(TokenKind::RParen),
            b'{' => self.make(TokenKind::LBrace),
            b'}' => self.make(TokenKind::RBrace),
            b',' => self.make(TokenKind::Comma),
            b'.' if self.char_match(b'.') && self.char_match(b'.') => self.make(TokenKind::Pass),
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

                    return Err(ThrushError::Error(
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
            b'<' => self.make(TokenKind::Less),
            b'>' if self.char_match(b'=') => self.make(TokenKind::GreaterEq),
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

                return Err(ThrushError::Error(
                    String::from("Unknown character."),
                    String::from("Did you provide a valid character?"),
                    self.line,
                    Some(self.span),
                ));
            }
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ThrushError> {
        while self.is_alpha(self.peek()) || self.peek().is_ascii_digit() && self.peek() != b':' {
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

    fn integer_or_float(&mut self) -> Result<(), ThrushError> {
        while self.peek().is_ascii_digit()
            || self.peek() == b'_' && self.peek_next().is_ascii_digit()
            || self.peek() == b'.' && self.peek_next().is_ascii_digit()
        {
            self.advance();
        }

        self.end_span();

        let types: (DataTypes, bool) = self.parse_float_or_integer(self.lexeme().to_str())?;

        let raw_parsed_number: Result<f64, ParseFloatError> = self.lexeme().to_str().parse::<f64>();

        if raw_parsed_number.is_err() {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Out of bounds number."),
                self.line,
                Some(self.span),
            ));
        }

        let parsed_number: f64 = raw_parsed_number.unwrap();

        if types.0.is_float_type() {
            self.tokens.push(Token {
                kind: TokenKind::Float(types.0, parsed_number, types.1),
                lexeme: b"",
                line: self.line,
                span: self.span,
            });

            return Ok(());
        }

        self.tokens.push(Token {
            kind: TokenKind::Integer(types.0, parsed_number, types.1),
            lexeme: b"",
            line: self.line,
            span: self.span,
        });

        Ok(())
    }

    #[inline(always)]
    fn parse_float_or_integer(&mut self, lexeme: &str) -> Result<(DataTypes, bool), ThrushError> {
        if lexeme.contains('.') {
            return self.parse_float(lexeme);
        }

        self.parse_integer(lexeme)
    }

    #[inline(always)]
    fn parse_float(&self, lexeme: &str) -> Result<(DataTypes, bool), ThrushError> {
        let dot_count: usize = lexeme.bytes().filter(|&b| b == b'.').count();

        if dot_count > 1 {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Float values should only contain one dot."),
                self.line,
                Some(self.span),
            ));
        }

        if lexeme.parse::<f32>().is_ok() {
            return Ok((DataTypes::F32, false));
        }

        if lexeme.parse::<f64>().is_ok() {
            return Ok((DataTypes::F64, false));
        }

        Err(ThrushError::Error(
            String::from("Syntax error"),
            String::from("Out of bounds."),
            self.line,
            Some(self.span),
        ))
    }

    #[inline(always)]
    fn parse_integer(&self, lexeme: &str) -> Result<(DataTypes, bool), ThrushError> {
        const I8_MIN: isize = -128;
        const I8_MAX: isize = 127;
        const I16_MIN: isize = -32768;
        const I16_MAX: isize = 32767;
        const I32_MIN: isize = -2147483648;
        const I32_MAX: isize = 2147483647;

        match lexeme.parse::<isize>() {
            Ok(num) => {
                if (I8_MIN..=I8_MAX).contains(&num) {
                    Ok((DataTypes::I8, false))
                } else if (I16_MIN..=I16_MAX).contains(&num) {
                    Ok((DataTypes::I16, false))
                } else if (I32_MIN..=I32_MAX).contains(&num) {
                    Ok((DataTypes::I32, false))
                } else if (isize::MIN..=isize::MAX).contains(&num) {
                    Ok((DataTypes::I64, false))
                } else {
                    Err(ThrushError::Error(
                        String::from("Syntax error."),
                        String::from("Out of bounds."),
                        self.line,
                        Some(self.span),
                    ))
                }
            }

            Err(_) => Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Out of bounds."),
                self.line,
                Some(self.span),
            )),
        }
    }

    fn char(&mut self) -> Result<(), ThrushError> {
        while self.peek() != b'\'' && !self.end() {
            self.advance();
        }

        self.end_span();

        if self.peek() != b'\'' {
            return Err(ThrushError::Error(
                String::from("Syntax error"),
                String::from("Unclosed char. Did you forget to close the char with a \'?"),
                self.line,
                Some(self.span),
            ));
        }

        self.advance();

        if self.code[self.start + 1..self.current - 1].len() > 1 {
            return Err(ThrushError::Error(
                String::from("Syntax Error"),
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

    fn string(&mut self) -> Result<(), ThrushError> {
        while self.is_string_boundary() {
            self.advance();
        }

        self.end_span();

        if self.peek() != b'"' {
            return Err(ThrushError::Error(
                String::from("Syntax Error"),
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

    fn advance(&mut self) -> u8 {
        let char: u8 = self.code[self.current];
        self.current += 1;

        char
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.code.len() {
            return b'\0';
        }

        self.code[self.current + 1]
    }

    fn peek(&self) -> u8 {
        if self.end() {
            return b'\0';
        }

        self.code[self.current]
    }

    fn char_match(&mut self, ch: u8) -> bool {
        if !self.end() && self.code[self.current] == ch {
            self.current += 1;
            return true;
        }

        false
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

    #[inline(always)]
    fn lexeme(&self) -> Lexeme<'a> {
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

impl TokenLexeme for Lexeme<'_> {
    #[inline(always)]
    fn to_str(&self) -> &str {
        core::str::from_utf8(self).unwrap_or("invalid utf-8")
    }

    #[inline(always)]
    fn to_string(&self) -> String {
        self.to_str().to_string()
    }

    fn parse_scapes(&self, line: usize, span: (usize, usize)) -> Result<String, ThrushError> {
        let mut parsed_string: String = String::with_capacity(self.len());

        let mut i: usize = 0;

        while i < self.len() {
            if self[i] == b'\\' {
                i += 1;

                match self[i] {
                    b'n' => parsed_string.push('\n'),
                    b't' => parsed_string.push('\t'),
                    b'r' => parsed_string.push('\r'),
                    b'\\' => parsed_string.push('\\'),
                    b'0' => parsed_string.push('\0'),
                    b'\'' => parsed_string.push('\''),
                    b'"' => parsed_string.push('"'),
                    _ => {
                        return Err(ThrushError::Error(
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

            let char: Option<char> = char::from_u32(self[i] as u32);

            if char.is_none() {
                return Err(ThrushError::Error(
                    String::from("Syntax Error"),
                    String::from("Invalid char boundary in string."),
                    line,
                    Some(span),
                ));
            }

            parsed_string.push(char.unwrap());

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
    Pass,       // ...

    // --- Literals ---
    Identifier,
    Integer(DataTypes, f64, bool),
    Float(DataTypes, f64, bool),
    DataType(DataTypes),
    Str,
    Char,

    // --- Keywords ---
    New,
    Import,
    Extern,
    Builtin,
    Public,
    And,
    Struct,
    Else,
    False,
    Fn,
    For,
    Continue,
    Break,
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
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::For => write!(f, "for"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Elif => write!(f, "elif"),
            TokenKind::Public => write!(f, "public"),
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
            TokenKind::Extern => write!(f, "@extern"),
            TokenKind::Import => write!(f, "@import"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Pass => write!(f, "..."),
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
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataTypes {
    // Integer DataTypes
    I8,
    I16,
    I32,
    I64,

    // Floating Point DataTypes
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

impl std::fmt::Display for DataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataTypes::I8 => write!(f, "i8"),
            DataTypes::I16 => write!(f, "i16"),
            DataTypes::I32 => write!(f, "i32"),
            DataTypes::I64 => write!(f, "i64"),
            DataTypes::F32 => write!(f, "f32"),
            DataTypes::F64 => write!(f, "f64"),
            DataTypes::Bool => write!(f, "bool"),
            DataTypes::Str => write!(f, "str"),
            DataTypes::Char => write!(f, "char"),
            DataTypes::Struct => write!(f, "struct"),
            DataTypes::Ptr => write!(f, "ptr"),
            DataTypes::Generic => write!(f, "generic"),
            DataTypes::Void => write!(f, "void"),
        }
    }
}

impl DataTypes {
    #[inline(always)]
    pub const fn calculate_integer_datatype(self, other: DataTypes) -> DataTypes {
        match (self, other) {
            (DataTypes::I64, _) | (_, DataTypes::I64) => DataTypes::I64,
            (DataTypes::I32, _) | (_, DataTypes::I32) => DataTypes::I32,
            (DataTypes::I16, _) | (_, DataTypes::I16) => DataTypes::I16,
            _ => DataTypes::I8,
        }
    }

    #[inline(always)]
    pub const fn calculate_float_datatype(self, other: DataTypes) -> DataTypes {
        match (self, other) {
            (DataTypes::F64, _) | (_, DataTypes::F64) => DataTypes::F64,
            (DataTypes::F32, _) | (_, DataTypes::F32) => DataTypes::F32,
            _ => DataTypes::F64,
        }
    }

    #[inline(always)]
    pub const fn is_void_type(&self) -> bool {
        matches!(self, DataTypes::Void)
    }

    #[inline(always)]
    pub const fn is_bool_type(&self) -> bool {
        matches!(self, DataTypes::Bool)
    }

    #[inline(always)]
    pub const fn is_struct_type(&self) -> bool {
        matches!(self, DataTypes::Struct)
    }

    #[inline(always)]
    pub const fn is_float_type(&self) -> bool {
        matches!(self, DataTypes::F32 | DataTypes::F64)
    }

    #[inline(always)]
    pub const fn is_ptr_type(&self) -> bool {
        matches!(self, DataTypes::Struct | DataTypes::Str | DataTypes::Ptr)
    }

    #[inline(always)]
    pub const fn is_heaped_ptr(&self) -> bool {
        matches!(self, DataTypes::Struct | DataTypes::Ptr)
    }

    #[inline(always)]
    pub const fn is_integer_type(&self) -> bool {
        matches!(
            self,
            DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64 | DataTypes::Char
        )
    }
}
