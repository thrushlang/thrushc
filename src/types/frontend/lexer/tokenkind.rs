use crate::backend::llvm::compiler::attributes::LLVMAttribute;

use super::types::ThrushType;

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

    // --- Keywords ---
    Alloc,
    Address,
    Instr,
    Load,
    Write,
    CastPtr,
    Heap,
    Stack,
    Static,
    New,
    Import,
    Methods,
    Mut,
    Ref,
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
    Match,
    Pattern,
    If,
    Elif,
    Or,
    Return,
    This,
    Me,
    True,
    Local,
    Const,
    While,
    Loop,
    NullPtr,

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

    Eof,
}

impl TokenKind {
    #[inline(always)]
    pub const fn as_compiler_attribute<'ctx>(self) -> Option<LLVMAttribute<'ctx>> {
        match self {
            TokenKind::Ignore => Some(LLVMAttribute::Ignore),
            TokenKind::MinSize => Some(LLVMAttribute::MinSize),
            TokenKind::NoInline => Some(LLVMAttribute::NoInline),
            TokenKind::AlwaysInline => Some(LLVMAttribute::AlwaysInline),
            TokenKind::InlineHint => Some(LLVMAttribute::InlineHint),
            TokenKind::Hot => Some(LLVMAttribute::Hot),
            TokenKind::SafeStack => Some(LLVMAttribute::SafeStack),
            TokenKind::WeakStack => Some(LLVMAttribute::WeakStack),
            TokenKind::StrongStack => Some(LLVMAttribute::StrongStack),
            TokenKind::PreciseFloats => Some(LLVMAttribute::PreciseFloats),
            _ => None,
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

    #[must_use]
    pub const fn is_sync_declaration(&self) -> bool {
        matches!(
            self,
            TokenKind::Import
                | TokenKind::Type
                | TokenKind::Struct
                | TokenKind::Fn
                | TokenKind::Enum
                | TokenKind::Const
                | TokenKind::Methods
        )
    }

    #[must_use]
    pub const fn is_sync_statement(&self) -> bool {
        matches!(
            self,
            TokenKind::LBrace
                | TokenKind::Return
                | TokenKind::Local
                | TokenKind::For
                | TokenKind::New
                | TokenKind::If
                | TokenKind::Match
                | TokenKind::While
                | TokenKind::Continue
                | TokenKind::Break
                | TokenKind::Loop
        )
    }

    #[must_use]
    pub const fn is_sync_expression(&self) -> bool {
        matches!(
            self,
            TokenKind::SemiColon | TokenKind::LBrace | TokenKind::RBrace
        )
    }

    pub const fn is_logical_gate(&self) -> bool {
        matches!(self, TokenKind::And | TokenKind::Or)
    }

    pub const fn is_struct_keyword(&self) -> bool {
        matches!(self, TokenKind::Struct)
    }

    pub const fn is_methods_keyword(&self) -> bool {
        matches!(self, TokenKind::Methods)
    }

    #[inline(always)]
    pub const fn is_type_keyword(&self) -> bool {
        matches!(self, TokenKind::Type)
    }

    #[inline(always)]
    pub const fn is_const_keyword(&self) -> bool {
        matches!(self, TokenKind::Const)
    }

    #[inline(always)]
    pub const fn is_enum_keyword(&self) -> bool {
        matches!(self, TokenKind::Enum)
    }

    #[inline(always)]
    pub const fn is_plusplus_operator(&self) -> bool {
        matches!(self, TokenKind::PlusPlus)
    }

    #[inline(always)]
    pub const fn is_minus_operator(&self) -> bool {
        matches!(self, TokenKind::Minus)
    }

    #[inline(always)]
    pub const fn is_mut(&self) -> bool {
        matches!(self, TokenKind::Mut)
    }

    #[inline(always)]
    pub const fn is_me(&self) -> bool {
        matches!(self, TokenKind::Me)
    }

    #[inline(always)]
    pub const fn is_function_keyword(&self) -> bool {
        matches!(self, TokenKind::Fn)
    }

    #[inline(always)]
    pub const fn is_void(&self) -> bool {
        matches!(self, TokenKind::Void)
    }

    #[inline(always)]
    pub const fn is_bool(&self) -> bool {
        matches!(self, TokenKind::Bool)
    }

    pub const fn is_str(&self) -> bool {
        matches!(self, TokenKind::Str)
    }

    #[inline(always)]
    pub const fn is_ptr(&self) -> bool {
        matches!(self, TokenKind::Ptr)
    }

    #[inline(always)]
    pub const fn is_float(&self) -> bool {
        matches!(self, TokenKind::F32 | TokenKind::F64)
    }

    #[inline(always)]
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            TokenKind::S8
                | TokenKind::S16
                | TokenKind::S32
                | TokenKind::S64
                | TokenKind::U8
                | TokenKind::U16
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::Char
        )
    }

    #[inline(always)]
    pub const fn is_type(&self) -> bool {
        self.is_integer()
            || self.is_float()
            || self.is_bool()
            || self.is_ptr()
            || self.is_str()
            || self.is_void()
            || self.is_mut()
            || self.is_me()
    }

    #[inline(always)]
    pub fn as_type(&self) -> ThrushType {
        match self {
            TokenKind::Char => ThrushType::Char,

            TokenKind::S8 => ThrushType::S8,
            TokenKind::S16 => ThrushType::S16,
            TokenKind::S32 => ThrushType::S32,
            TokenKind::S64 => ThrushType::S64,

            TokenKind::U8 => ThrushType::U8,
            TokenKind::U16 => ThrushType::U16,
            TokenKind::U32 => ThrushType::U32,
            TokenKind::U64 => ThrushType::U64,

            TokenKind::Bool => ThrushType::Bool,

            TokenKind::F32 => ThrushType::F32,
            TokenKind::F64 => ThrushType::F64,

            TokenKind::Str => ThrushType::Str,
            TokenKind::Ptr => ThrushType::Ptr(None),

            _ => ThrushType::Void,
        }
    }
}
