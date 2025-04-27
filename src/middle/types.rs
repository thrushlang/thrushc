use std::sync::Arc;

use ahash::{HashSet, HashSetExt};
use inkwell::{context::Context, targets::TargetData};

use crate::backend::llvm::compiler::{attributes::LLVMAttribute, typegen};

pub struct TypeContext {
    pub function_type: Type,
}

impl TypeContext {
    pub fn new(function_type: Type) -> Self {
        Self { function_type }
    }
}

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
    Address,
    Carry,
    Write,
    New,
    Import,
    Builtin,
    Raw,
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
    True,
    Local,
    Const,
    While,
    Loop,

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
    #[inline(always)]
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::New
                | TokenKind::Import
                | TokenKind::Builtin
                | TokenKind::Struct
                | TokenKind::Else
                | TokenKind::False
                | TokenKind::Fn
                | TokenKind::For
                | TokenKind::Continue
                | TokenKind::Break
                | TokenKind::Match
                | TokenKind::Pattern
                | TokenKind::If
                | TokenKind::Elif
                | TokenKind::Or
                | TokenKind::Return
                | TokenKind::This
                | TokenKind::Local
                | TokenKind::Const
                | TokenKind::While
                | TokenKind::Loop
        )
    }

    #[inline(always)]
    pub const fn is_logical_gate(&self) -> bool {
        matches!(self, TokenKind::And | TokenKind::Or)
    }

    #[inline(always)]
    pub const fn is_struct_keyword(&self) -> bool {
        matches!(self, TokenKind::Struct)
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
    }

    #[inline(always)]
    pub fn as_type(&self) -> Type {
        match self {
            TokenKind::Char => Type::Char,

            TokenKind::S8 => Type::S8,
            TokenKind::S16 => Type::S16,
            TokenKind::S32 => Type::S32,
            TokenKind::S64 => Type::S64,

            TokenKind::U8 => Type::U8,
            TokenKind::U16 => Type::U16,
            TokenKind::U32 => Type::U32,
            TokenKind::U64 => Type::U64,

            TokenKind::Bool => Type::Bool,

            TokenKind::F32 => Type::F32,
            TokenKind::F64 => Type::F64,

            TokenKind::Str => Type::Str,
            TokenKind::Ptr => Type::Ptr(None),

            _ => Type::Void,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    // Signed Integer Type
    S8,
    S16,
    S32,
    S64,

    // Unsigned Integer Type
    U8,
    U16,
    U32,
    U64,

    // Floating Point Type
    F32,
    F64,

    // Boolean Type
    Bool,

    // Char Type
    Char,

    // Str Type
    Str,

    // Ptr Type
    Ptr(Option<Arc<Type>>),

    // Struct Type
    Struct(Vec<Arc<Type>>),

    // Address
    Address,

    // Void Type
    Void,
}

impl Type {
    #[inline(always)]
    pub fn precompute_type(&self, other: &Type) -> &Type {
        match (self, other) {
            (Type::S64, _) | (_, Type::S64) => &Type::S64,
            (Type::S32, _) | (_, Type::S32) => &Type::S32,
            (Type::S16, _) | (_, Type::S16) => &Type::S16,
            (Type::S8, _) | (_, Type::S8) => &Type::S8,

            (Type::U64, _) | (_, Type::U64) => &Type::U64,
            (Type::U32, _) | (_, Type::U32) => &Type::U32,
            (Type::U16, _) | (_, Type::U16) => &Type::U16,
            (Type::U8, _) | (_, Type::U8) => &Type::U8,

            (Type::F64, _) | (_, Type::F64) => &Type::F64,
            (Type::F32, _) | (_, Type::F32) => &Type::F32,

            _ => self,
        }
    }

    #[inline(always)]
    pub fn is_stack_allocated(&self) -> bool {
        self.is_bool_type()
            || self.is_float_type()
            || self.is_integer_type()
            || self.is_char_type()
            || self.is_str_type()
            || !self.is_recursive_type()
            || self.is_stack_allocated_pointer()
    }

    pub fn is_stack_allocated_pointer(&self) -> bool {
        if let Type::Ptr(Some(subtype)) = self {
            return subtype.is_stack_allocated();
        }

        false
    }

    #[inline(always)]
    pub const fn is_char_type(&self) -> bool {
        matches!(self, Type::Char)
    }

    #[inline(always)]
    pub const fn is_void_type(&self) -> bool {
        matches!(self, Type::Void)
    }

    #[inline(always)]
    pub const fn is_bool_type(&self) -> bool {
        matches!(self, Type::Bool)
    }

    #[inline(always)]
    pub const fn is_struct_type(&self) -> bool {
        matches!(self, Type::Struct(_))
    }

    #[inline(always)]
    pub const fn is_float_type(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    #[inline(always)]
    pub const fn is_ptr_type(&self) -> bool {
        matches!(self, Type::Ptr(_))
    }

    #[inline(always)]
    pub const fn is_address_type(&self) -> bool {
        matches!(self, Type::Address)
    }

    #[inline(always)]
    pub const fn is_str_type(&self) -> bool {
        matches!(self, Type::Str)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_signed_integer_type(&self) -> bool {
        matches!(self, Type::S8 | Type::S16 | Type::S32 | Type::S64)
    }

    #[inline(always)]
    pub const fn is_integer_type(&self) -> bool {
        matches!(
            self,
            Type::S8
                | Type::S16
                | Type::S32
                | Type::S64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Char
        )
    }

    pub fn narrowing_cast(&self) -> Type {
        match self {
            Type::U8 => Type::S8,
            Type::U16 => Type::S16,
            Type::U32 => Type::S32,
            Type::U64 => Type::S64,
            _ => self.clone(),
        }
    }

    pub fn llvm_exceeds_stack(&self, context: &Context, target_data: &TargetData) -> u64 {
        target_data.get_abi_size(&typegen::generate_type(context, self))
    }

    pub fn is_recursive_type(&self) -> bool {
        if let Type::Struct(fields) = self {
            let mut visited: HashSet<*const Type> = HashSet::with_capacity(100);

            fields
                .iter()
                .any(|field| field.is_recursive_with_original(fields, &mut visited))
        } else {
            false
        }
    }

    fn is_recursive_with_original(
        &self,
        original_fields: &[Arc<Type>],
        visited: &mut HashSet<*const Type>,
    ) -> bool {
        let ptr: *const Type = self as *const Type;

        if visited.contains(&ptr) {
            return false;
        }

        visited.insert(ptr);

        let result: bool = if let Type::Struct(fields) = self {
            if fields.iter().map(|f| f.as_ref()).collect::<Vec<&Type>>()
                == original_fields
                    .iter()
                    .map(|f| f.as_ref())
                    .collect::<Vec<&Type>>()
            {
                true
            } else {
                fields
                    .iter()
                    .any(|field| field.is_recursive_with_original(original_fields, visited))
            }
        } else {
            false
        };

        visited.remove(&ptr);

        result
    }

    pub fn create_structure_type(fields: &[Type]) -> Type {
        Type::Struct(fields.iter().map(|field| Arc::new(field.clone())).collect())
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Struct(fields1), Type::Struct(fields2)) => {
                fields1.len() == fields2.len()
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .all(|(f1, f2)| f1.as_ref() == f2.as_ref())
            }

            (Type::Char, Type::Char) => true,
            (Type::S8, Type::S8) => true,
            (Type::S16, Type::S16) => true,
            (Type::S32, Type::S32) => true,
            (Type::S64, Type::S64) => true,
            (Type::U8, Type::U8) => true,
            (Type::U16, Type::U16) => true,
            (Type::U32, Type::U32) => true,
            (Type::U64, Type::U64) => true,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Ptr(None), Type::Ptr(None)) => true,
            (Type::Ptr(Some(target)), Type::Ptr(Some(from))) => target == from,
            (Type::Void, Type::Void) => true,
            (Type::Str, Type::Str) => true,
            (Type::Bool, Type::Bool) => true,

            _ => false,
        }
    }
}
