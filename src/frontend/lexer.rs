use {
    super::super::{
        backend::compiler::options::ThrushFile, diagnostic::Diagnostic, error::{ThrushError, ThrushErrorKind}, logging::LogType
    }, core::str, inkwell::{FloatPredicate, IntPredicate}, std::{mem, num::ParseFloatError, process::exit}
};

pub struct Lexer<'a> {
    tokens: Vec<Token>,
    errors: Vec<ThrushError>,
    code: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
    diagnostic: Diagnostic
}

impl<'a> Lexer<'a> {
    pub fn lex(code: &'a [u8], file: &ThrushFile) -> Vec<Token> {
        let mut lexer: Lexer = Self {
            tokens: Vec::new(),
            errors: Vec::new(),
            code,
            start: 0,
            current: 0,
            line: 1,
            diagnostic: Diagnostic::new(file)
        };

        lexer._lex()
    }

    fn _lex(&mut self) -> Vec<Token> {
        while !self.end() {
            self.start = self.current;

            match self.scan() {
                Ok(()) => {}
                Err(e) => self.errors.push(e),
            }
        }

        if !self.errors.is_empty() {
            self.errors.iter().for_each(|error| {
                self.diagnostic.report(error, LogType::ERROR, false);
            });
       
            exit(1);
        };

        self.tokens.push(Token {
            lexeme: None,
            kind: TokenKind::Eof,
            line: self.line
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
                    return Err(ThrushError::Lex(
                        ThrushErrorKind::SyntaxError,
        
                        String::from("Syntax Error"),
                        String::from(
                            "Unterminated multiline comment. Did you forget to close the string with a '*/'?",
                        ),
                        self.line,
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
            b'a'..=b'z' | b'A'..=b'Z' | b'_'  | b'@' => self.identifier()?,
            _ => {
                return Err(ThrushError::Lex(
                    ThrushErrorKind::UnknownChar,
                    String::from("Unknown character."),
                    String::from("Did you provide a valid character?"),
                    self.line,
                ));
            }
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), ThrushError> {

        while self.is_alpha(self.peek()) || self.peek().is_ascii_digit() {
            self.advance();
        }

        match str::from_utf8(&self.code[self.start..self.current]).unwrap() {
            "var" => self.make(TokenKind::Var),
            "fn" => self.make(TokenKind::Fn),
            "if" => self.make(TokenKind::If),
            "elif" => self.make(TokenKind::Elif),
            "else" => self.make(TokenKind::Else),
            "for" => self.make(TokenKind::For),
            "while" => self.make(TokenKind::While),
            "true" => self.make(TokenKind::True),
            "false" => self.make(TokenKind::False),
            "or" => self.make(TokenKind::Or),
            "and" => self.make(TokenKind::And),
            "const" => self.make(TokenKind::Const),
            "struct" => self.make(TokenKind::Struct),
            "return" => self.make(TokenKind::Return),
            "break" => self.make(TokenKind::Break),
            "continue" => self.make(TokenKind::Continue),
            "super" => self.make(TokenKind::Super),
            "this" => self.make(TokenKind::This),
            "extends" => self.make(TokenKind::Extends),
            "public" => self.make(TokenKind::Public),
            "builtin" => self.make(TokenKind::Builtin),
            "@import" => self.make(TokenKind::Import),
            "@extern" => self.make(TokenKind::Extern),
            "new" => self.make(TokenKind::New),

            "null" => self.make(TokenKind::Null),

            "i8" => self.make(TokenKind::DataType(DataTypes::I8)),
            "i16" => self.make(TokenKind::DataType(DataTypes::I16)),
            "i32" => self.make(TokenKind::DataType(DataTypes::I32)),
            "i64" => self.make(TokenKind::DataType(DataTypes::I64)),

            "f32" => self.make(TokenKind::DataType(DataTypes::F32)),
            "f64" => self.make(TokenKind::DataType(DataTypes::F64)),

            "bool" => self.make(TokenKind::DataType(DataTypes::Bool)),
            "char" => self.make(TokenKind::DataType(DataTypes::Char)),
            "ptr" => self.make(TokenKind::DataType(DataTypes::Ptr)),
            "str" => self.make(TokenKind::DataType(DataTypes::Str)),

            "void" => self.make(TokenKind::DataType(DataTypes::Void)),

            _ => {
                self.tokens.push(Token {
                    kind: TokenKind::Identifier,
                    lexeme: Some(self.lexeme()),
                    line: self.line,
                });
            }
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

        let kind: (DataTypes, bool) =
            self.eval_integer_or_float_type(self.lexeme())?;

        let num: Result<f64, ParseFloatError> = self.lexeme().parse::<f64>();

        if num.is_err() {
            let mut lexeme: String = self.lexeme();
            lexeme.truncate(18);

            return Err(ThrushError::Parse(
                ThrushErrorKind::ParsedNumber,
                String::from("The number is too big for an integer or float."),
                String::from(
                    "Did you provide a valid number with the correct format and not out of bounds?",
                ),
                self.line,
                format!("{};", lexeme),
            ));
        }

        if kind.0.is_float() {
            self.tokens.push(Token {
                kind: TokenKind::Float(kind.0, *num.as_ref().unwrap(), kind.1),
                lexeme: None,
                line: self.line
            });

            return Ok(());
        }

        self.tokens.push(Token {
            kind: TokenKind::Integer(kind.0, num.unwrap(), kind.1),
            lexeme: None,
            line: self.line
        });

        Ok(())
    }

    fn char(&mut self) -> Result<(), ThrushError> {
        while self.peek() != b'\'' && !self.end() {
            self.advance();
        }

        if self.peek() != b'\'' {
            return Err(ThrushError::Lex(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Unterminated char. Did you forget to close the char with a '\''?",
                ),
                self.line,
            ));
        }

        self.advance();

        if self.code[self.start + 1..self.current - 1].len() > 1 {
            return Err(ThrushError::Lex(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "A char data type only can contain one character.",
                ),
                self.line,
            ))
        }

        self.tokens.push(Token {
            kind: TokenKind::Char,
            lexeme: Some(String::from_utf8_lossy(&self.code[self.start + 1..self.current - 1]).to_string()),
            line: self.line,
        });

        Ok(())

    }

    fn string(&mut self) -> Result<(), ThrushError> {

        while self.peek() != b'"' && !self.end() {
            self.advance();
        }

        if self.peek() != b'"' {
            return Err(ThrushError::Lex(
                ThrushErrorKind::SyntaxError,
                String::from("Syntax Error"),
                String::from(
                    "Unterminated string. Did you forget to close the string with a '\"'?",
                ),
                self.line,
            ));
        }

        self.advance();

        let mut string: String =
            String::from_utf8_lossy(&self.code[self.start + 1..self.current - 1]).to_string();

        string = string.replace("\\n", "\n");
        string = string.replace("\\r", "\r");
        string = string.replace("\\t", "\t");

        self.tokens.push(Token {
            kind: TokenKind::Str,
            lexeme: Some(string),
            line: self.line,
        });

        Ok(())
    }

    pub fn eval_integer_or_float_type(
        &mut self,
        mut lexeme: String,
    ) -> Result<(DataTypes, bool), ThrushError> {

        if lexeme.contains(".") {

            if lexeme.chars().filter(|ch| *ch == '.').count() > 1 {
                return Err(ThrushError::Lex(
                    ThrushErrorKind::SyntaxError, 
                    String::from("Float Violated Syntax"), 
                    String::from("Floats values should be only contain one dot."), 
                    self.line
                ));
            } else if lexeme.parse::<f32>().is_ok() {
                return Ok((DataTypes::F32, false));
            } else if lexeme.parse::<f64>().is_ok() {
                return Ok((DataTypes::F64, false));
            } 

            lexeme.truncate(18);

            return Err(ThrushError::Parse(
                ThrushErrorKind::ParsedNumber,
                String::from("The number is too big for an float."),
                String::from("Did you provide a valid number with the correct format and not out of bounds?"),
                self.line,
                format!("{};", lexeme),
            ));
            
        }

        match lexeme.parse::<isize>() {
            Ok(num) => match num {
                -128isize..=127isize => Ok((DataTypes::I8, false)),
                -32728isize..=32767isize => Ok((DataTypes::I16, false)),
                -2147483648isize..=2147483647isize => Ok((DataTypes::I32, false)),
                -9223372036854775808isize..= 9223372036854775807isize => Ok((DataTypes::I64, false)),
                _ => {
                    lexeme.truncate(18);
                    
                    Err(ThrushError::Parse(
                        ThrushErrorKind::UnreachableNumber,
                        String::from("Unreacheable Number."),
                        String::from("The size is out of bounds of an isize (0 to n)."),
                        self.line,
                        format!("{};", lexeme),
                    ))
                }
            },
            Err(_) => {
                lexeme.truncate(18);

                Err(ThrushError::Parse(
                    ThrushErrorKind::ParsedNumber,
                    String::from("Unreacheable Number"),
                    String::from(
                        "Did you provide a valid number with the correct format and not out of bounds?",
                    ),
                    self.line,
                    format!("{};", lexeme),
                ))
            },
        }
    }

    fn advance(&mut self) -> u8 {
        let ch: u8 = self.code[self.current];
        self.current += 1;

        ch
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

    #[inline]
    fn end(&self) -> bool {
        self.current >= self.code.len()
    }

    #[inline]
    fn is_alpha(&self, ch: u8) -> bool {
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == b'_'
    }

    #[inline]
    fn lexeme(&self) -> String {
        String::from_utf8_lossy(&self.code[self.start..self.current]).to_string()
    }

    fn make(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            lexeme: Some(self.lexeme()),
            line: self.line,
        });
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub lexeme: Option<String>,
    pub kind: TokenKind,
    pub line: usize,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    // --- Operators ---
    LParen,       // ' ( '
    RParen,       // ' ) '
    LBrace,       // ' { '
    RBrace,       // ' } '
    Comma,        // ' , '
    Dot,          // ' . '
    Minus,        // ' - '
    Plus,         // ' + '
    Slash,        // ' / '
    Star,         // ' * '
    Colon,        // ' : '
    SemiColon,    // ' ; '
    RBracket,  // ' ] '
    LBracket, // ' [ '
    Arith,        // ' % ',
    Bang,         // ' ! '
    ColonColon,   // ' :: '
    BangEq,    // ' != '
    Eq,           // ' = '
    EqEq,         // ' == '
    Greater,      // ' > '
    GreaterEq, // ' >= '
    Less,         // ' < '
    LessEq,    // ' <= '
    PlusPlus,     // ' ++ '
    MinusMinus,   // ' -- '
    Pass, // ...

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
    Null,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    Const,
    While,
    Extends,

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
            TokenKind::Null => write!(f, "null"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::This => write!(f, "this"),
            TokenKind::True => write!(f, "true"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Extends => write!(f, "extends"),
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
    #[inline]
    pub fn as_int_predicate(&self, left_signed: bool, right_signed: bool) -> IntPredicate {
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

    #[inline]
    pub fn as_float_predicate(&self) -> FloatPredicate {

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

    #[inline]
    pub fn is_logical_type(&self) -> bool {
        if let TokenKind::BangEq | TokenKind::EqEq | TokenKind::LessEq | TokenKind::Less | TokenKind::Greater | TokenKind::GreaterEq = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_logical_gate(&self) -> bool {
        if let TokenKind::And | TokenKind::Or = self {
            return true;
        }

        false
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
            DataTypes::Str => write!(f, "String"),
            DataTypes::Char => write!(f, "char"),
            DataTypes::Struct => write!(f, "struct"),
            DataTypes::Ptr => write!(f, "ptr"),
            DataTypes::Void => write!(f, "()"),
        }
    }
}

impl DataTypes {
    #[inline]
    pub fn calculate_integer_datatype(self, other: DataTypes) -> DataTypes {
        match (self, other) {
            (DataTypes::I64, _) | (_, DataTypes::I64) => DataTypes::I64,
            (DataTypes::I32, _) | (_, DataTypes::I32) => DataTypes::I32,
            (DataTypes::I16, _) | (_, DataTypes::I16) => DataTypes::I16,
            _ => DataTypes::I8,
        }
    }

    #[inline]
    pub fn calculate_float_datatype(self, other: DataTypes) -> DataTypes {
        match (self, other) {
            (DataTypes::F64, _) | (_, DataTypes::F64) => DataTypes::F64,
            (DataTypes::F32, _) | (_, DataTypes::F32) => DataTypes::F32,
            _ => DataTypes::F64,
        }
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        if let DataTypes::F32 | DataTypes::F64 = self {
            return true;
        }

        false
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        if let DataTypes::I8 | DataTypes::I16 | DataTypes::I32 | DataTypes::I64 | DataTypes::Char = self {
            return true;
        }

        false
    }
}
