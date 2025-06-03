use crate::types::frontend::parser::{
    stmts::{
        traits::{EnumExtensions, EnumFieldsExtensions},
        types::{EnumField, EnumFields},
    },
    symbols::types::EnumSymbol,
};

use super::{
    tokenkind::TokenKind,
    traits::ThrushStructTypeExtensions,
    types::{ThrushStructType, ThrushType},
};

impl<'a> EnumFieldsExtensions<'a> for EnumFields<'a> {
    fn contain_field(&self, name: &'a str) -> bool {
        self.iter().any(|enum_field| enum_field.0 == name)
    }

    fn get_field(&self, name: &'a str) -> EnumField<'a> {
        self.iter()
            .find(|enum_field| enum_field.0 == name)
            .cloned()
            .unwrap()
    }
}
impl<'a> EnumExtensions<'a> for EnumSymbol<'a> {
    fn get_fields(&self) -> EnumFields<'a> {
        self.0.clone()
    }
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
            TokenKind::Arrow => write!(f, "->"),
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
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "+="),
            TokenKind::Identifier => write!(f, "identifier"),
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
            TokenKind::Or => write!(f, "or"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Write => write!(f, "write"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::This => write!(f, "this"),
            TokenKind::True => write!(f, "true"),
            TokenKind::Local => write!(f, "local"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Ref => write!(f, "ref"),
            TokenKind::Pass => write!(f, "..."),
            TokenKind::NullPtr => write!(f, "nullptr"),
            TokenKind::Methods => write!(f, "methods"),
            TokenKind::Instr => write!(f, "instr"),
            TokenKind::Integer | TokenKind::Float => write!(f, "number"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Public => write!(f, "@public"),
            TokenKind::Ignore => write!(f, "@ignore"),
            TokenKind::MinSize => write!(f, "@minsize"),
            TokenKind::NoInline => write!(f, "@noinline"),
            TokenKind::AlwaysInline => write!(f, "@alwaysinline"),
            TokenKind::InlineHint => write!(f, "@inlinehint"),
            TokenKind::Hot => write!(f, "@hot"),
            TokenKind::SafeStack => write!(f, "@safestack"),
            TokenKind::WeakStack => write!(f, "@weakstack"),
            TokenKind::StrongStack => write!(f, "@strongstack"),
            TokenKind::PreciseFloats => write!(f, "@precisefloats"),
            TokenKind::Convention => write!(f, "@convention"),
            TokenKind::Extern => write!(f, "@extern"),
            TokenKind::Import => write!(f, "@import"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::S8 => write!(f, "s8"),
            TokenKind::S16 => write!(f, "s16"),
            TokenKind::S32 => write!(f, "s32"),
            TokenKind::S64 => write!(f, "s64"),
            TokenKind::U8 => write!(f, "u8"),
            TokenKind::U16 => write!(f, "u16"),
            TokenKind::U32 => write!(f, "u32"),
            TokenKind::U64 => write!(f, "u64"),
            TokenKind::F32 => write!(f, "f32"),
            TokenKind::F64 => write!(f, "f64"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Str => write!(f, "str"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::Ptr => write!(f, "ptr"),
            TokenKind::Static => write!(f, "static!"),
            TokenKind::Heap => write!(f, "heap!"),
            TokenKind::Stack => write!(f, "stack!"),
            TokenKind::Address => write!(f, "address"),
            TokenKind::Load => write!(f, "load"),
            TokenKind::Alloc => write!(f, "alloc"),
            TokenKind::Addr => write!(f, "addr"),
            TokenKind::CastRaw => write!(f, "castraw"),
            TokenKind::Cast => write!(f, "cast"),
            TokenKind::Void => write!(f, "void"),
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
            ThrushType::Struct(name, fields) => {
                let _ = write!(f, "struct {} {{ ", name);

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
