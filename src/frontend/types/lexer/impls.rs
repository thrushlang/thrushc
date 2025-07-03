use crate::frontend::{lexer::tokentype::TokenType, types::lexer::Type};

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
            TokenType::Mut => write!(f, "mut"),
            TokenType::New => write!(f, "new"),
            TokenType::Or => write!(f, "or"),
            TokenType::Return => write!(f, "return"),
            TokenType::Struct => write!(f, "struct"),
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
            TokenType::Star => write!(f, "*"),
            TokenType::SizeOf => write!(f, "sizeof"),

            // Builtins
            TokenType::AlignOf => write!(f, "alignof"),
            TokenType::MemSet => write!(f, "memset"),
            TokenType::MemMove => write!(f, "memmove"),
            TokenType::MemCpy => write!(f, "memcpy"),
            TokenType::Halloc => write!(f, "halloc"),
        }
    }
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
            Type::Mut(any_type) => write!(f, "mut {}", any_type),
            Type::FixedArray(kind, size) => {
                write!(f, "[{}; {}]", kind, size)
            }
            Type::Array(kind) => {
                write!(f, "[{}]", kind)
            }
            Type::Struct(name, fields) => {
                write!(f, "struct {} {{ ", name)?;

                fields.iter().for_each(|field| {
                    let _ = write!(f, "{} ", field);
                });

                write!(f, "}}")
            }
            Type::Ptr(nested_type) => {
                if let Some(nested_type) = nested_type {
                    let _ = write!(f, "ptr[");
                    let _ = write!(f, "{}", nested_type);

                    return write!(f, "]");
                }

                write!(f, "ptr")
            }
            Type::Addr => {
                write!(f, "memory address")
            }
            Type::Void => write!(f, "void"),
        }
    }
}
