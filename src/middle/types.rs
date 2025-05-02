use std::sync::Arc;

use ahash::{HashSet, HashSetExt};
use inkwell::{context::Context, targets::TargetData};

use crate::{
    backend::llvm::compiler::{attributes::LLVMAttribute, typegen},
    common::error::ThrushCompilerError,
    frontend::{lexer::Span, symbols::SymbolsTable},
};

use super::{
    instruction::Instruction,
    statement::{StructFields, traits::StructExtensions},
    symbols::types::{Bindings, Struct},
};

pub type ThrushStructType = (String, Vec<Arc<Type>>);

#[derive(Debug, Clone, Copy)]
pub enum BindingsApplicant {
    Struct,
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
    Address,
    Carry,
    Write,
    New,
    Import,
    Bindings,
    Bind,
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
                | TokenKind::Bindings
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
                | TokenKind::Bind
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

    pub const fn is_bindings_keyword(&self) -> bool {
        matches!(self, TokenKind::Bindings)
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

    // Mutable Type
    Mut(Arc<Type>),

    // Ptr Type
    Ptr(Option<Arc<Type>>),

    // Struct Type
    Struct(String, Vec<Arc<Type>>),

    // Address
    Address,

    // Void Type
    Void,
}

impl Type {
    #[must_use]
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

            (Type::Mut(a_subtype), Type::Mut(b_subtype)) => a_subtype.precompute_type(b_subtype),

            _ => self,
        }
    }

    pub fn is_heap_allocated(&self, context: &Context, target_data: &TargetData) -> bool {
        target_data.get_abi_size(&typegen::generate_type(context, self)) >= 100
            || self.is_recursive_type()
    }

    pub fn is_mut_ptr_type(&self) -> bool {
        if let Type::Mut(subtype) = self {
            if let Type::Ptr(_) = &**subtype {
                return true;
            }
        }

        false
    }

    pub fn is_mut_numeric_type(&self) -> bool {
        if let Type::Mut(subtype) = self {
            return subtype.is_integer_type() || subtype.is_float_type();
        }

        false
    }

    pub fn into_structure_type(self) -> ThrushStructType {
        if let Type::Struct(name, types) = self {
            return (name, types);
        }

        unreachable!()
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
        matches!(self, Type::Struct(..))
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
    pub const fn is_mut_type(&self) -> bool {
        matches!(self, Type::Mut(_))
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

    pub fn is_recursive_type(&self) -> bool {
        if let Type::Struct(_, fields) = self {
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

        let result: bool = if let Type::Struct(_, fields) = self {
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

    pub fn create_structure_type(name: String, fields: &[Type]) -> Type {
        Type::Struct(
            name,
            fields.iter().map(|field| Arc::new(field.clone())).collect(),
        )
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Struct(_, fields1), Type::Struct(_, fields2)) => {
                fields1.len() == fields2.len()
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .all(|(f1, f2)| f1.as_ref() == f2.as_ref())
            }

            (Type::Mut(target), Type::Mut(from)) => target == from,
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

pub fn generate_bindings(original_bindings: Vec<Instruction>) -> Bindings {
    let mut bindings: Bindings = Vec::with_capacity(original_bindings.len());

    for binding in original_bindings {
        bindings.push((
            binding.get_binding_name(),
            binding.get_binding_type(),
            binding.get_binding_parameters(),
        ));
    }

    bindings
}

pub fn decompose_struct_property(
    mut position: usize,
    property_names: Vec<&'_ str>,
    struct_type: Type,
    symbols_table: &SymbolsTable<'_>,
    span: Span,
) -> Result<(Type, Vec<(Type, u32)>), ThrushCompilerError> {
    let mut gep_indices: Vec<(Type, u32)> = Vec::with_capacity(10);

    if position >= property_names.len() {
        return Ok((struct_type.clone(), gep_indices));
    }

    if let Type::Struct(name, _) = &struct_type {
        let structure: Struct = symbols_table.get_struct(name, span)?;
        let fields: StructFields = structure.get_fields();

        let field_name: &str = property_names[position];

        let field_with_index: Option<(usize, &(&str, Type, u32))> = fields
            .1
            .iter()
            .enumerate()
            .find(|field| field.1.0 == field_name);

        if let Some((index, (_, field_type, _))) = field_with_index {
            gep_indices.push((field_type.clone(), index as u32));

            position += 1;

            let (result_type, mut nested_indices) = decompose_struct_property(
                position,
                property_names,
                field_type.clone(),
                symbols_table,
                span,
            )?;

            gep_indices.append(&mut nested_indices);

            return Ok((result_type, gep_indices));
        }

        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            format!("Expected existing property, not '{}'.", field_name,),
            String::default(),
            span,
        ));
    }

    if position < property_names.len() {
        return Err(ThrushCompilerError::Error(
            String::from("Syntax error"),
            format!(
                "Existing property '{}' is not a structure.",
                property_names[position]
            ),
            String::default(),
            span,
        ));
    }

    Ok((struct_type.clone(), gep_indices))
}
