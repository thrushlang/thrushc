use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{lexer::span::Span, types::lexer::Type},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
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
    Pass,       // ' ... '
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
    MinusEq,    // -=
    PlusEq,     // +=
    LShift,     // ' << '
    RShift,     // ' >> '
    Arrow,      // ->

    // --- Literals ---
    Identifier,
    Integer,
    Float,

    // --- Attributes ---
    Extern,
    Ignore,
    Public,
    MinSize,
    NoInline,
    AlwaysInline,
    InlineHint,
    Hot,
    SafeStack,
    WeakStack,
    StrongStack,
    PreciseFloats,
    Convention,
    AsmAlignStack,
    AsmSyntax,
    AsmThrow,
    AsmSideEffects,

    // --- Keywords ---
    Alloc,
    Address,
    Instr,
    Load,
    Write,
    AsmFn,
    Asm,
    Glasm,
    Deref,
    As,
    Heap,
    Stack,
    Static,
    New,
    Fixed,
    Import,
    SizeOf,
    Mut,
    Type,
    Enum,
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
    Or,
    Return,
    True,
    Local,
    Const,
    While,
    Loop,
    NullPtr,

    // -- Builtins --
    AlignOf,
    MemCpy,
    MemMove,
    MemSet,

    // --- Types ---
    S8,
    S16,
    S32,
    S64,

    U8,
    U16,
    U32,
    U64,

    F32,
    F64,

    Bool,
    Char,
    Str,
    Ptr,
    Void,
    Addr,
    Array,

    Eof,
}

impl TokenType {
    #[must_use]
    pub const fn is_logical_operator(&self) -> bool {
        matches!(
            self,
            TokenType::BangEq
                | TokenType::EqEq
                | TokenType::LessEq
                | TokenType::Less
                | TokenType::Greater
                | TokenType::GreaterEq
        )
    }

    #[must_use]
    pub const fn is_sync_declaration(&self) -> bool {
        matches!(
            self,
            TokenType::Import
                | TokenType::Type
                | TokenType::Struct
                | TokenType::Fn
                | TokenType::Enum
                | TokenType::Const
        )
    }

    #[must_use]
    pub const fn is_sync_statement(&self) -> bool {
        matches!(
            self,
            TokenType::LBrace
                | TokenType::Return
                | TokenType::Local
                | TokenType::For
                | TokenType::New
                | TokenType::If
                | TokenType::While
                | TokenType::Continue
                | TokenType::Break
                | TokenType::Loop
        )
    }

    #[must_use]
    pub const fn is_sync_expression(&self) -> bool {
        matches!(
            self,
            TokenType::SemiColon | TokenType::LBrace | TokenType::RBrace
        )
    }

    pub const fn is_logical_gate(&self) -> bool {
        matches!(self, TokenType::And | TokenType::Or)
    }

    #[must_use]
    pub const fn is_minus_minus_operator(&self) -> bool {
        matches!(self, TokenType::MinusMinus)
    }

    #[must_use]
    pub const fn is_plus_plus_operator(&self) -> bool {
        matches!(self, TokenType::PlusPlus)
    }

    #[must_use]
    pub const fn is_address(&self) -> bool {
        matches!(self, TokenType::Addr)
    }

    #[must_use]
    pub const fn is_mut(&self) -> bool {
        matches!(self, TokenType::Mut)
    }

    #[must_use]
    pub const fn is_void(&self) -> bool {
        matches!(self, TokenType::Void)
    }

    #[must_use]
    pub const fn is_bool(&self) -> bool {
        matches!(self, TokenType::Bool)
    }

    pub const fn is_str(&self) -> bool {
        matches!(self, TokenType::Str)
    }

    #[must_use]
    pub const fn is_array(&self) -> bool {
        matches!(self, TokenType::Array)
    }

    #[must_use]
    pub const fn is_ptr(&self) -> bool {
        matches!(self, TokenType::Ptr)
    }

    #[must_use]
    pub const fn is_float(&self) -> bool {
        matches!(self, TokenType::F32 | TokenType::F64)
    }

    #[must_use]
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            TokenType::S8
                | TokenType::S16
                | TokenType::S32
                | TokenType::S64
                | TokenType::U8
                | TokenType::U16
                | TokenType::U32
                | TokenType::U64
                | TokenType::Char
        )
    }

    #[must_use]
    pub fn is_type(&self) -> bool {
        self.is_integer()
            || self.is_float()
            || self.is_bool()
            || self.is_array()
            || self.is_ptr()
            || self.is_str()
            || self.is_void()
            || self.is_mut()
            || self.is_address()
    }

    #[must_use]
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenType::Identifier)
    }
}

impl TokenType {
    #[must_use]
    pub fn as_compiler_attribute<'ctx>(self, span: Span) -> Option<LLVMAttribute<'ctx>> {
        match self {
            TokenType::Ignore => Some(LLVMAttribute::Ignore(span)),
            TokenType::MinSize => Some(LLVMAttribute::MinSize(span)),
            TokenType::NoInline => Some(LLVMAttribute::NoInline(span)),
            TokenType::AlwaysInline => Some(LLVMAttribute::AlwaysInline(span)),
            TokenType::InlineHint => Some(LLVMAttribute::InlineHint(span)),
            TokenType::Hot => Some(LLVMAttribute::Hot(span)),
            TokenType::SafeStack => Some(LLVMAttribute::SafeStack(span)),
            TokenType::WeakStack => Some(LLVMAttribute::WeakStack(span)),
            TokenType::StrongStack => Some(LLVMAttribute::StrongStack(span)),
            TokenType::PreciseFloats => Some(LLVMAttribute::PreciseFloats(span)),
            TokenType::Stack => Some(LLVMAttribute::Stack(span)),
            TokenType::Heap => Some(LLVMAttribute::Heap(span)),
            TokenType::AsmThrow => Some(LLVMAttribute::AsmThrow(span)),
            TokenType::AsmSideEffects => Some(LLVMAttribute::AsmSideEffects(span)),
            TokenType::AsmAlignStack => Some(LLVMAttribute::AsmAlignStack(span)),
            _ => None,
        }
    }

    pub fn as_type(&self, span: Span) -> Result<Type, ThrushCompilerIssue> {
        match self {
            TokenType::Char => Ok(Type::Char),

            TokenType::S8 => Ok(Type::S8),
            TokenType::S16 => Ok(Type::S16),
            TokenType::S32 => Ok(Type::S32),
            TokenType::S64 => Ok(Type::S64),

            TokenType::U8 => Ok(Type::U8),
            TokenType::U16 => Ok(Type::U16),
            TokenType::U32 => Ok(Type::U32),
            TokenType::U64 => Ok(Type::U64),

            TokenType::Bool => Ok(Type::Bool),

            TokenType::F32 => Ok(Type::F32),
            TokenType::F64 => Ok(Type::F64),

            TokenType::Str => Ok(Type::Str),

            TokenType::Ptr => Ok(Type::Ptr(None)),
            TokenType::Addr => Ok(Type::Addr),
            TokenType::Void => Ok(Type::Void),

            any => Err(ThrushCompilerIssue::Error(
                "Syntax error".into(),
                format!("{} isn't a valid type.", any),
                None,
                span,
            )),
        }
    }
}
