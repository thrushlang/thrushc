use super::super::common::error::ThrushCompilerError;

use super::types::CodeLocation;

use super::super::backend::compiler::instruction::Instruction;

use super::{
    lexer::{TokenKind, Type},
    objects::{FoundObjectId, Struct},
    traits::{FoundObjectEither, FoundObjectExtensions, StructureExtensions},
};

impl<'a> StructureExtensions<'a> for Struct<'a> {
    fn contains_field(&self, name: &str) -> bool {
        self.iter().any(|field| field.0 == name)
    }

    fn get_field_type(&self, name: &str) -> Option<Instruction<'a>> {
        if let Some(field) = self.iter().find(|field| field.0 == name) {
            let field_type: Instruction = field.1.clone();

            return Some(field_type);
        }

        None
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
            TokenKind::Or => write!(f, "or"),
            TokenKind::Take => write!(f, "take"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::This => write!(f, "this"),
            TokenKind::True => write!(f, "true"),
            TokenKind::Local => write!(f, "local"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Integer(tp, _, _) => write!(f, "{}", tp),
            TokenKind::Float(tp, _, _) => write!(f, "{}", tp),
            TokenKind::Enum => write!(f, "enum"),
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
            Type::Ptr => write!(f, "ptr"),
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

    fn is_local(&self) -> bool {
        self.2.is_some()
    }
}

impl<'instr> FoundObjectEither<'instr> for FoundObjectId<'instr> {
    fn expected_local(
        &self,
        location: CodeLocation,
    ) -> Result<(&'instr str, usize), ThrushCompilerError> {
        if let Some((name, scope_idx)) = self.2 {
            return Ok((name, scope_idx));
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected local reference"),
            String::from("Expected local but found something else."),
            location.0,
            Some(location.1),
        ))
    }

    fn expected_function(
        &self,
        location: CodeLocation,
    ) -> Result<&'instr str, ThrushCompilerError> {
        if let Some(name) = self.1 {
            return Ok(name);
        }

        Err(ThrushCompilerError::Error(
            String::from("Expected function reference"),
            String::from("Expected function but found something else."),
            location.0,
            Some(location.1),
        ))
    }
}
