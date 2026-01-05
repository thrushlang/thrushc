use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_typesystem::Type;

use crate::{
    Token,
    tokentype::TokenType,
    traits::{TokenTypeBuiltinExtensions, TokenTypeExtensions, TokenTypeTypeTransform},
};

impl TokenTypeTypeTransform for TokenType {
    fn as_type(&self, span: Span) -> Result<Type, CompilationIssue> {
        match self {
            TokenType::Char => Ok(Type::Char(span)),

            TokenType::S8 => Ok(Type::S8(span)),
            TokenType::S16 => Ok(Type::S16(span)),
            TokenType::S32 => Ok(Type::S32(span)),
            TokenType::S64 => Ok(Type::S64(span)),
            TokenType::Ssize => Ok(Type::SSize(span)),

            TokenType::U8 => Ok(Type::U8(span)),
            TokenType::U16 => Ok(Type::U16(span)),
            TokenType::U32 => Ok(Type::U32(span)),
            TokenType::U64 => Ok(Type::U64(span)),
            TokenType::U128 => Ok(Type::U128(span)),
            TokenType::Usize => Ok(Type::USize(span)),

            TokenType::Bool => Ok(Type::Bool(span)),

            TokenType::F32 => Ok(Type::F32(span)),
            TokenType::F64 => Ok(Type::F64(span)),
            TokenType::F128 => Ok(Type::F128(span)),

            TokenType::FX8680 => Ok(Type::FX8680(span)),
            TokenType::FPPC128 => Ok(Type::FPPC128(span)),

            TokenType::Ptr => Ok(Type::Ptr(None, span)),
            TokenType::Addr => Ok(Type::Addr(span)),
            TokenType::Void => Ok(Type::Void(span)),

            any => Err(CompilationIssue::Error(
                CompilationIssueCode::E0001,
                format!("{} isn't a valid type.", any),
                None,
                span,
            )),
        }
    }
}

impl TokenTypeExtensions for TokenType {
    #[inline]
    fn is_logical_operator(&self) -> bool {
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

    #[inline]
    fn is_logical_gate(&self) -> bool {
        matches!(self, TokenType::And | TokenType::Or)
    }

    #[inline]
    fn is_minus_minus_operator(&self) -> bool {
        matches!(self, TokenType::MinusMinus)
    }

    #[inline]
    fn is_plus_plus_operator(&self) -> bool {
        matches!(self, TokenType::PlusPlus)
    }

    #[inline]
    fn is_address(&self) -> bool {
        matches!(self, TokenType::Addr)
    }

    #[inline]
    fn is_void(&self) -> bool {
        matches!(self, TokenType::Void)
    }

    #[inline]
    fn is_bool(&self) -> bool {
        matches!(self, TokenType::Bool)
    }

    #[inline]
    fn is_array(&self) -> bool {
        matches!(self, TokenType::Array)
    }

    #[inline]
    fn is_ptr(&self) -> bool {
        matches!(self, TokenType::Ptr)
    }

    #[inline]
    fn is_float(&self) -> bool {
        matches!(
            self,
            TokenType::F32
                | TokenType::F64
                | TokenType::F128
                | TokenType::FX8680
                | TokenType::FPPC128
        )
    }

    #[inline]
    fn is_const(&self) -> bool {
        matches!(self, TokenType::Const)
    }

    #[inline]
    fn is_fn_ref(&self) -> bool {
        matches!(self, TokenType::FnRef)
    }

    #[inline]
    fn is_integer(&self) -> bool {
        matches!(
            self,
            TokenType::S8
                | TokenType::S16
                | TokenType::S32
                | TokenType::S64
                | TokenType::Ssize
                | TokenType::U8
                | TokenType::U16
                | TokenType::U32
                | TokenType::U64
                | TokenType::U128
                | TokenType::Usize
                | TokenType::Char
        )
    }

    #[inline]
    fn is_type(&self) -> bool {
        self.is_integer()
            || self.is_float()
            || self.is_bool()
            || self.is_array()
            || self.is_ptr()
            || self.is_void()
            || self.is_address()
            || self.is_const()
            || self.is_fn_ref()
    }

    #[inline]
    fn is_attribute(&self) -> bool {
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
                | TokenType::AsmSyntax
                | TokenType::Packed
                | TokenType::NoUnwind
                | TokenType::OptFuzzing
                | TokenType::Constructor
                | TokenType::Destructor
                | TokenType::Public
                | TokenType::Linkage
                | TokenType::Extern
                | TokenType::Convention
        )
    }

    #[inline]
    fn is_identifier(&self) -> bool {
        matches!(self, TokenType::Identifier)
    }
}

impl TokenTypeBuiltinExtensions for TokenType {
    fn is_builtin(&self) -> bool {
        matches!(
            self,
            TokenType::Halloc
                | TokenType::MemCpy
                | TokenType::MemMove
                | TokenType::MemSet
                | TokenType::AlignOf
                | TokenType::SizeOf
                | TokenType::BitSizeOf
                | TokenType::AbiSizeOf
                | TokenType::AbiAlignOf
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TOKEN TYPE '{:?}' literal '{}', ascii '{}' at '{}'.",
            self.kind, self.lexeme, self.ascii, self.span
        )
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Keywords
            TokenType::Break => write!(f, "break"),
            TokenType::Const => write!(f, "const"),
            TokenType::Continue => write!(f, "continue"),
            TokenType::Elif => write!(f, "elif"),
            TokenType::Else => write!(f, "else"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::False => write!(f, "false"),
            TokenType::Intrinsic => write!(f, "intrinsic"),
            TokenType::Fn => write!(f, "fn"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::Loop => write!(f, "loop"),
            TokenType::Mut => write!(f, "mut"),
            TokenType::New => write!(f, "new"),
            TokenType::Return => write!(f, "return"),
            TokenType::Struct => write!(f, "struct"),
            TokenType::True => write!(f, "true"),
            TokenType::Type => write!(f, "type"),
            TokenType::While => write!(f, "while"),
            TokenType::Write => write!(f, "write"),
            TokenType::Local => write!(f, "local"),
            TokenType::Asm => write!(f, "asm"),
            TokenType::GlobalAsm => write!(f, "global_asm"),

            // Direct Reference
            TokenType::DirectRef => write!(f, "ref"),

            // Indirect Call
            TokenType::Indirect => write!(f, "indirect"),

            // Types
            TokenType::Address => write!(f, "address"),
            TokenType::Bool => write!(f, "bool"),
            TokenType::Char => write!(f, "char"),
            TokenType::F32 => write!(f, "f32"),
            TokenType::F64 => write!(f, "f64"),
            TokenType::FX8680 => write!(f, "fx86_80"),
            TokenType::F128 => write!(f, "f128"),
            TokenType::FPPC128 => write!(f, "fppc_128"),
            TokenType::Ptr => write!(f, "ptr"),
            TokenType::Array => write!(f, "array"),
            TokenType::S8 => write!(f, "s8"),
            TokenType::S16 => write!(f, "s16"),
            TokenType::S32 => write!(f, "s32"),
            TokenType::S64 => write!(f, "s64"),
            TokenType::Ssize => write!(f, "ssize"),
            TokenType::Str => write!(f, "const ptr[array[char]]"),
            TokenType::U8 => write!(f, "u8"),
            TokenType::U16 => write!(f, "u16"),
            TokenType::U32 => write!(f, "u32"),
            TokenType::U64 => write!(f, "u64"),
            TokenType::U128 => write!(f, "u128"),
            TokenType::Usize => write!(f, "usize"),
            TokenType::FnRef => write!(f, "Fn"),
            TokenType::Void => write!(f, "void"),

            // Special
            TokenType::Unreachable => write!(f, "unreachable"),

            // Modificators
            TokenType::LazyThread => write!(f, "lazythread"),
            TokenType::Volatile => write!(f, "volatile"),

            TokenType::AtomNone => write!(f, "atomnone"),
            TokenType::AtomFree => write!(f, "atomfree"),
            TokenType::AtomRelax => write!(f, "atomrelax"),
            TokenType::AtomGrab => write!(f, "atomgrab"),
            TokenType::AtomDrop => write!(f, "atomdrop"),
            TokenType::AtomSync => write!(f, "atomsync"),
            TokenType::AtomStrict => write!(f, "atomstrict"),

            TokenType::ThreadDynamic => write!(f, "threaddyn"),
            TokenType::ThreadExec => write!(f, "threadexec"),
            TokenType::ThreadInit => write!(f, "threadinit"),
            TokenType::ThreadLDynamic => write!(f, "threadldyn"),

            // Attributes
            TokenType::Linkage => write!(f, "@linkage"),
            TokenType::OptFuzzing => write!(f, "@optfuzzing"),
            TokenType::NoUnwind => write!(f, "@nounwind"),
            TokenType::Packed => write!(f, "@packed"),
            TokenType::Stack => write!(f, "@stack"),
            TokenType::Static => write!(f, "@static"),
            TokenType::Heap => write!(f, "@heap"),
            TokenType::AlwaysInline => write!(f, "@alwaysinline"),
            TokenType::AsmAlignStack => write!(f, "@asmalignstack"),
            TokenType::AsmSyntax => write!(f, "@asmsyntax"),
            TokenType::AsmSideEffects => write!(f, "@asmeffects"),
            TokenType::AsmThrow => write!(f, "@asmthrow"),
            TokenType::Convention => write!(f, "@convention"),
            TokenType::Extern => write!(f, "@extern"),
            TokenType::Hot => write!(f, "@hot"),
            TokenType::Ignore => write!(f, "@ignore"),
            TokenType::Import => write!(f, "import"),
            TokenType::ImportC => write!(f, "importC"),
            TokenType::InlineHint => write!(f, "@inlinehint"),
            TokenType::MinSize => write!(f, "@minsize"),
            TokenType::NoInline => write!(f, "@noinline"),
            TokenType::PreciseFloats => write!(f, "@precisefloats"),
            TokenType::Public => write!(f, "@public"),
            TokenType::SafeStack => write!(f, "@safestack"),
            TokenType::StrongStack => write!(f, "@strongstack"),
            TokenType::WeakStack => write!(f, "@weakstack"),
            TokenType::Destructor => write!(f, "@destructor"),
            TokenType::Constructor => write!(f, "@constructor"),

            // Operators, Punctuation, and Special Constructs
            TokenType::Or => write!(f, "||"),
            TokenType::And => write!(f, "&&"),
            TokenType::Float => write!(f, "integer"),
            TokenType::Integer => write!(f, "float"),
            TokenType::Addr => write!(f, "addr"),
            TokenType::Alloc => write!(f, "alloc"),
            TokenType::Arith => write!(f, "%"),
            TokenType::Arrow => write!(f, "->"),
            TokenType::AsmFn => write!(f, "asmfn"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEq => write!(f, "!="),
            TokenType::As => write!(f, "as"),
            TokenType::Colon => write!(f, ":"),
            TokenType::ColonColon => write!(f, "::"),
            TokenType::Comma => write!(f, ","),
            TokenType::Deref => write!(f, "deref"),
            TokenType::Dot => write!(f, "."),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Eq => write!(f, "="),
            TokenType::EqEq => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEq => write!(f, ">="),
            TokenType::Identifier => write!(f, "identifier"),
            TokenType::Fixed => write!(f, "fixed"),
            TokenType::Instr => write!(f, "instr"),
            TokenType::LBrace => write!(f, "{{"),
            TokenType::LBracket => write!(f, "["),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEq => write!(f, "<="),
            TokenType::Load => write!(f, "load"),
            TokenType::LParen => write!(f, "("),
            TokenType::LShift => write!(f, "<<"),
            TokenType::Minus => write!(f, "-"),
            TokenType::MinusEq => write!(f, "-="),
            TokenType::MinusMinus => write!(f, "--"),
            TokenType::NullPtr => write!(f, "nullptr"),
            TokenType::Pass => write!(f, "..."),
            TokenType::Plus => write!(f, "+"),
            TokenType::PlusEq => write!(f, "+="),
            TokenType::PlusPlus => write!(f, "++"),
            TokenType::Range => write!(f, ".."),
            TokenType::RBrace => write!(f, "}}"),
            TokenType::RBracket => write!(f, "]"),
            TokenType::RParen => write!(f, ")"),
            TokenType::RShift => write!(f, ">>"),
            TokenType::SemiColon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Xor => write!(f, "^"),
            TokenType::Not => write!(f, "~"),
            TokenType::Bor => write!(f, "|"),
            TokenType::BAnd => write!(f, "&"),
            TokenType::Star => write!(f, "*"),

            // Builtins
            TokenType::Halloc => write!(f, "halloc"),
            TokenType::AlignOf => write!(f, "align_of"),
            TokenType::MemSet => write!(f, "memset"),
            TokenType::MemMove => write!(f, "memmove"),
            TokenType::MemCpy => write!(f, "memcpy"),
            TokenType::SizeOf => write!(f, "size_of"),
            TokenType::AbiSizeOf => write!(f, "abi_size_of"),
            TokenType::BitSizeOf => write!(f, "bit_size_of"),
            TokenType::AbiAlignOf => write!(f, "abi_align_of"),
        }
    }
}
