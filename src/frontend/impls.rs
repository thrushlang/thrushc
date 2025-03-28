use super::{
    lexer::{TokenKind, Type},
    objects::Struct,
    traits::StructureBasics,
};

impl StructureBasics for Struct<'_> {
    fn contains_field(&self, field_name: &str) -> bool {
        self.iter().any(|field| field.0 == field_name)
    }

    fn get_field_type(&self, field_name: &str) -> Type {
        self.iter()
            .find(|field| field.0 == field_name)
            .map(|field| field.1)
            .unwrap()
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
            TokenKind::NullT => write!(f, "nullT"),
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
            TokenKind::DataType(datatype) => write!(f, "{}", datatype),
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
            Type::Struct => write!(f, "struct"),
            Type::T => write!(f, "T"),
            Type::Void => write!(f, "void"),
        }
    }
}
