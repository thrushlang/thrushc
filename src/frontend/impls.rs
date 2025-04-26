use super::super::common::error::ThrushCompilerError;

use super::traits::{CustomTypeFieldsExtensions, EnumExtensions, EnumFieldsExtensions};

use super::super::backend::compiler::types::{
    CustomTypeFields, Enum, EnumField, EnumFields, StructFields, ThrushAttributes,
};

use super::{
    lexer::{Span, TokenKind, Type},
    objects::{FoundObjectId, Struct},
    traits::{FoundObjectEither, FoundObjectExtensions, StructureExtensions},
};

impl<'a> StructureExtensions<'a> for Struct<'a> {
    fn contains_field(&self, name: &str) -> bool {
        self.0.iter().any(|field| field.0 == name)
    }

    fn get_field_type(&self, name: &str) -> Option<Type> {
        if let Some(field) = self.0.iter().find(|field| field.0 == name) {
            let field_type: Type = field.1.clone();
            return Some(field_type);
        }

        None
    }

    fn get_fields(&self) -> StructFields<'a> {
        self.0.clone()
    }
}

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
impl<'a> EnumExtensions<'a> for Enum<'a> {
    fn get_fields(&self) -> EnumFields<'a> {
        self.0.clone()
    }

    fn get_attributes(&self) -> ThrushAttributes<'a> {
        self.1.clone()
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
            TokenKind::Take => write!(f, "take"),
            TokenKind::Write => write!(f, "write"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::This => write!(f, "this"),
            TokenKind::True => write!(f, "true"),
            TokenKind::Local => write!(f, "local"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Integer | TokenKind::Float => write!(f, "number"),
            TokenKind::Enum => write!(f, "enum"),
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
            TokenKind::Address => write!(f, "address"),
            TokenKind::Carry => write!(f, "carry"),
            TokenKind::Void => write!(f, "void"),
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
            Type::Struct(fields) => {
                let _ = write!(f, "struct {{ ");

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
            Type::Address => {
                write!(f, "memory address")
            }
            Type::Void => write!(f, "void"),
        }
    }
}

impl FoundObjectExtensions for FoundObjectId<'_> {
    fn is_structure(&self) -> bool {
        self.0.is_some()
    }

    fn is_function(&self) -> bool {
        self.1.is_some()
    }

    fn is_enum(&self) -> bool {
        self.2.is_some()
    }

    fn is_constant(&self) -> bool {
        self.3.is_some()
    }

    fn is_custom_type(&self) -> bool {
        self.4.is_some()
    }
}

impl<'instr> FoundObjectEither<'instr> for FoundObjectId<'instr> {
    fn expected_custom_type(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(type_id) = self.4 {
            return Ok(type_id);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected custom type reference"),
            String::from("Expected custom type but found something else."),
            span,
        ))
    }

    fn expected_constant(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(const_id) = self.3 {
            return Ok(const_id);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected constant reference"),
            String::from("Expected constant but found something else."),
            span,
        ))
    }

    fn expected_enum(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.2 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected enum reference"),
            String::from("Expected enum but found something else."),
            span,
        ))
    }

    fn expected_struct(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.0 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected struct reference"),
            String::from("Expected struct but found something else."),
            span,
        ))
    }

    fn expected_function(&self, span: Span) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            span,
        ))
    }

    fn expected_local(&self, span: Span) -> Result<(&'instr str, usize), ThrushCompilerError> {
        if let Some((name, scope_idx)) = self.5 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            span,
        ))
    }
}

impl CustomTypeFieldsExtensions for CustomTypeFields<'_> {
    fn get_type(&self) -> Type {
        Type::create_structure_type(self)
    }
}
