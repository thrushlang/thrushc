use crate::{
    backend::llvm::compiler::attributes::LLVMAttribute,
    core::errors::standard::ThrushCompilerIssue,
    frontend::{lexer::span::Span, typesystem::types::Type},
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
    Heap,
    Stack,
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

    // --- Special ---
    Unreachable,

    // --- Modificators ---
    Volatile,
    LazyThread,

    // --- LLI ---
    Alloc,
    Address,
    Instr,
    Load,
    Write,

    // --- Keywords ---
    AsmFn,
    Asm,
    GlobalAsm,
    Deref,
    As,
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
    Fn,
    For,
    Continue,
    Break,
    If,
    Elif,
    Or,
    Return,
    Local,
    Const,
    While,
    Loop,

    // --- Literals ---
    True,
    False,
    NullPtr,

    // -- Builtins --
    AlignOf,
    Halloc,
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
    pub fn is_logical_operator(&self) -> bool {
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
    pub fn is_logical_gate(&self) -> bool {
        matches!(self, TokenType::And | TokenType::Or)
    }

    #[must_use]
    pub fn is_minus_minus_operator(&self) -> bool {
        matches!(self, TokenType::MinusMinus)
    }

    #[must_use]
    pub fn is_plus_plus_operator(&self) -> bool {
        matches!(self, TokenType::PlusPlus)
    }

    #[must_use]
    pub fn is_address(&self) -> bool {
        matches!(self, TokenType::Addr)
    }

    #[must_use]
    pub fn is_mut(&self) -> bool {
        matches!(self, TokenType::Mut)
    }

    #[must_use]
    pub fn is_void(&self) -> bool {
        matches!(self, TokenType::Void)
    }

    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, TokenType::Bool)
    }

    pub fn is_str(&self) -> bool {
        matches!(self, TokenType::Str)
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, TokenType::Array)
    }

    #[must_use]
    pub fn is_ptr(&self) -> bool {
        matches!(self, TokenType::Ptr)
    }

    #[must_use]
    pub fn is_float(&self) -> bool {
        matches!(self, TokenType::F32 | TokenType::F64)
    }

    #[must_use]
    pub fn is_const(&self) -> bool {
        matches!(self, TokenType::Const)
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
            || self.is_const()
    }

    #[must_use]
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenType::Identifier)
    }
}

impl TokenType {
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

impl TokenType {
    #[must_use]
    pub fn as_attribute<'ctx>(self, span: Span) -> Option<LLVMAttribute<'ctx>> {
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

    #[must_use]
    pub fn is_attribute(self) -> bool {
        matches!(
            self,
            TokenType::Ignore
                | TokenType::MinSize
                | TokenType::NoInline
                | TokenType::AlwaysInline
                | TokenType::InlineHint
                | TokenType::Hot
                | TokenType::SafeStack
                | TokenType::WeakStack
                | TokenType::StrongStack
                | TokenType::PreciseFloats
                | TokenType::Stack
                | TokenType::Heap
                | TokenType::AsmThrow
                | TokenType::AsmSideEffects
                | TokenType::AsmAlignStack
        )
    }
}
