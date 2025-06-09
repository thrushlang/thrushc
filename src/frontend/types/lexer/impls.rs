use crate::frontend::{
    lexer::tokentype::TokenType,
    types::lexer::{ThrushStructType, ThrushType, traits::ThrushStructTypeExtensions},
};

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Keywords
            TokenType::And => write!(f, "and"),
            TokenType::Break => write!(f, "break"),
            TokenType::Const => write!(f, "const"),
            TokenType::Continue => write!(f, "continue"),
            TokenType::Elif => write!(f, "elif"),
            TokenType::Else => write!(f, "else"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::False => write!(f, "false"),
            TokenType::Fn => write!(f, "fn"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::Loop => write!(f, "loop"),
            TokenType::Match => write!(f, "match"),
            TokenType::Mut => write!(f, "mut"),
            TokenType::New => write!(f, "new"),
            TokenType::Or => write!(f, "or"),
            TokenType::Pattern => write!(f, "pattern"),
            TokenType::Ref => write!(f, "ref"),
            TokenType::Return => write!(f, "return"),
            TokenType::Struct => write!(f, "struct"),
            TokenType::This => write!(f, "this"),
            TokenType::True => write!(f, "true"),
            TokenType::Type => write!(f, "type"),
            TokenType::While => write!(f, "while"),
            TokenType::Write => write!(f, "write"),
            TokenType::Local => write!(f, "local"),
            TokenType::Asm => write!(f, "asm"),
            // Types
            TokenType::Address => write!(f, "address"),
            TokenType::Bool => write!(f, "bool"),
            TokenType::Char => write!(f, "char"),
            TokenType::F32 => write!(f, "f32"),
            TokenType::F64 => write!(f, "f64"),
            TokenType::Ptr => write!(f, "ptr"),
            TokenType::Array => write!(f, "array"),
            TokenType::S8 => write!(f, "s8"),
            TokenType::S16 => write!(f, "s16"),
            TokenType::S32 => write!(f, "s32"),
            TokenType::S64 => write!(f, "s64"),
            TokenType::Str => write!(f, "str"),
            TokenType::U8 => write!(f, "u8"),
            TokenType::U16 => write!(f, "u16"),
            TokenType::U32 => write!(f, "u32"),
            TokenType::U64 => write!(f, "u64"),
            TokenType::Void => write!(f, "void"),
            // Attributes
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
            TokenType::Import => write!(f, "@import"),
            TokenType::InlineHint => write!(f, "@inlinehint"),
            TokenType::MinSize => write!(f, "@minsize"),
            TokenType::NoInline => write!(f, "@noinline"),
            TokenType::PreciseFloats => write!(f, "@precisefloats"),
            TokenType::Public => write!(f, "@public"),
            TokenType::SafeStack => write!(f, "@safestack"),
            TokenType::StrongStack => write!(f, "@strongstack"),
            TokenType::WeakStack => write!(f, "@weakstack"),
            // Operators, Punctuation, and Special Constructs
            TokenType::Float => write!(f, "integer"),
            TokenType::Integer => write!(f, "float"),
            TokenType::Addr => write!(f, "addr"),
            TokenType::Alloc => write!(f, "alloc"),
            TokenType::Arith => write!(f, "%"),
            TokenType::Arrow => write!(f, "->"),
            TokenType::AsmFn => write!(f, "asmfn"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEq => write!(f, "!="),
            TokenType::Cast => write!(f, "cast"),
            TokenType::CastPtr => write!(f, "castptr"),
            TokenType::CastRaw => write!(f, "castraw"),
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
            TokenType::Instr => write!(f, "instr"),
            TokenType::LBrace => write!(f, "{{"),
            TokenType::LBracket => write!(f, "["),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEq => write!(f, "<="),
            TokenType::Load => write!(f, "load"),
            TokenType::LParen => write!(f, "("),
            TokenType::LShift => write!(f, "<<"),
            TokenType::Methods => write!(f, "methods"),
            TokenType::Minus => write!(f, "-"),
            TokenType::MinusEq => write!(f, "-="),
            TokenType::MinusMinus => write!(f, "--"),
            TokenType::NullPtr => write!(f, "nullptr"),
            TokenType::Pass => write!(f, "..."),
            TokenType::Plus => write!(f, "+"),
            TokenType::PlusEq => write!(f, "+="),
            TokenType::PlusPlus => write!(f, "++"),
            TokenType::Range => write!(f, ".."),
            TokenType::RawPtr => write!(f, "rawptr"),
            TokenType::RBrace => write!(f, "}}"),
            TokenType::RBracket => write!(f, "]"),
            TokenType::RParen => write!(f, ")"),
            TokenType::RShift => write!(f, ">>"),
            TokenType::SemiColon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
        }
    }
}

impl std::fmt::Display for ThrushType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThrushType::S8 => write!(f, "s8"),
            ThrushType::S16 => write!(f, "s16"),
            ThrushType::S32 => write!(f, "s32"),
            ThrushType::S64 => write!(f, "s64"),
            ThrushType::U8 => write!(f, "u8"),
            ThrushType::U16 => write!(f, "u16"),
            ThrushType::U32 => write!(f, "u32"),
            ThrushType::U64 => write!(f, "u64"),
            ThrushType::F32 => write!(f, "f32"),
            ThrushType::F64 => write!(f, "f64"),
            ThrushType::Bool => write!(f, "bool"),
            ThrushType::Str => write!(f, "str"),
            ThrushType::Char => write!(f, "char"),
            ThrushType::Mut(any_type) => write!(f, "mut {}", any_type),
            ThrushType::FixedArray(kind, size) => {
                write!(f, "[{}; {}]", kind, size)
            }
            ThrushType::Struct(name, fields) => {
                write!(f, "struct {} {{ ", name)?;

                fields.iter().for_each(|field| {
                    let _ = write!(f, "{} ", field);
                });

                write!(f, "}}")
            }
            ThrushType::Ptr(nested_type) => {
                if let Some(nested_type) = nested_type {
                    let _ = write!(f, "ptr[");
                    let _ = write!(f, "{}", nested_type);

                    return write!(f, "]");
                }

                write!(f, "ptr")
            }
            ThrushType::Addr => {
                write!(f, "memory address")
            }
            ThrushType::Void => write!(f, "void"),
        }
    }
}

impl ThrushStructTypeExtensions for ThrushStructType {
    fn get_name(&self) -> String {
        self.0.clone()
    }
}
