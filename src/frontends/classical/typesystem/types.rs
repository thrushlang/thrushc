use std::sync::Arc;

use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::span::Span,
        parser::symbols::SymbolsTable,
        types::parser::{
            stmts::{traits::StructExtensions, types::StructFields},
            symbols::types::Struct,
        },
    },
};

#[derive(Debug, Clone, Default)]
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

    // Constant Type
    Const(Arc<Type>),

    // Mutable Type
    Mut(Arc<Type>),

    // Ptr Type
    Ptr(Option<Arc<Type>>),

    // Struct Type
    Struct(String, Vec<Arc<Type>>),

    // Fixed FixedArray
    FixedArray(Arc<Type>, u32),

    // Array Type
    Array(Arc<Type>),

    // Address
    Addr,

    // Void Type
    #[default]
    Void,
}

impl Type {
    pub fn get_fixed_array_base_type(&self) -> &Type {
        if let Type::FixedArray(inner, ..) = self {
            return inner;
        }

        if let Type::Mut(inner) = self {
            return inner.get_fixed_array_base_type();
        }

        if let Type::Ptr(Some(inner)) = self {
            return inner.get_fixed_array_base_type();
        }

        if let Type::Const(inner) = self {
            return inner.get_fixed_array_base_type();
        }

        self
    }

    pub fn get_array_base_type(&self) -> &Type {
        if let Type::Array(inner, ..) = self {
            return inner;
        }

        if let Type::Mut(inner) = self {
            return inner.get_array_base_type();
        }

        if let Type::Ptr(Some(inner)) = self {
            return inner.get_array_base_type();
        }

        if let Type::Const(inner) = self {
            return inner.get_array_base_type();
        }

        self
    }

    pub fn create_structure_type(name: String, fields: &[Type]) -> Type {
        Type::Struct(
            name,
            fields.iter().map(|field| Arc::new(field.clone())).collect(),
        )
    }

    #[inline(always)]
    pub fn is_char_type(&self) -> bool {
        matches!(self, Type::Char)
    }

    #[inline(always)]
    pub fn is_void_type(&self) -> bool {
        matches!(self, Type::Void)
    }

    #[inline(always)]
    pub fn is_bool_type(&self) -> bool {
        matches!(self, Type::Bool)
    }

    #[inline(always)]
    pub fn is_struct_type(&self) -> bool {
        matches!(self, Type::Struct(..))
    }

    #[inline(always)]
    pub fn is_fixed_array_type(&self) -> bool {
        matches!(self, Type::FixedArray(..))
    }

    #[inline(always)]
    pub fn is_array_type(&self) -> bool {
        matches!(self, Type::Array(..))
    }

    #[inline(always)]
    pub fn is_float_type(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    #[inline(always)]
    pub fn is_ptr_type(&self) -> bool {
        matches!(self, Type::Ptr(_))
    }

    #[inline(always)]
    pub fn is_address_type(&self) -> bool {
        matches!(self, Type::Addr)
    }

    #[inline(always)]
    pub fn is_str_type(&self) -> bool {
        matches!(self, Type::Str)
    }

    #[inline(always)]
    pub fn is_mut_type(&self) -> bool {
        matches!(self, Type::Mut(_))
    }

    #[inline(always)]
    pub fn is_const_type(&self) -> bool {
        matches!(self, Type::Const(_))
    }

    #[inline(always)]
    pub fn is_numeric(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[inline(always)]
    pub fn is_signed_integer_type(&self) -> bool {
        matches!(self, Type::S8 | Type::S16 | Type::S32 | Type::S64)
    }

    #[inline(always)]
    pub fn is_integer_type(&self) -> bool {
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

    #[inline]
    pub fn get_array_type_herarchy(&self) -> u8 {
        match self {
            Type::Void => 0,

            Type::Bool => 1,
            Type::Char => 2,
            Type::Str => 3,

            Type::S8 => 4,
            Type::S16 => 5,
            Type::S32 => 6,
            Type::S64 => 7,

            Type::U8 => 8,
            Type::U16 => 9,
            Type::U32 => 10,
            Type::U64 => 11,

            Type::F32 => 12,
            Type::F64 => 13,

            Type::Const(subtype) => subtype.get_array_type_herarchy(),
            Type::Mut(subtype) => subtype.get_array_type_herarchy(),

            Type::Addr => 14,
            Type::Ptr(Some(subtype)) => subtype.get_array_type_herarchy(),
            Type::Ptr(None) => 15,

            Type::FixedArray(..) => 16,
            Type::Array(..) => 17,
            Type::Struct(..) => 18,
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Struct(a, fields1), Type::Struct(b, fields2)) => {
                fields1.len() == fields2.len()
                    && a == b
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .all(|(f1, f2)| f1.as_ref() == f2.as_ref())
            }

            (Type::FixedArray(type_a, size_a), Type::FixedArray(type_b, size_b)) => {
                type_a == type_b && size_a == size_b
            }

            (Type::Mut(target), Type::Mut(from)) => target == from,
            (Type::Array(target), Type::Array(from)) => target == from,
            (Type::Const(target), Type::Const(from)) => target == from,

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

pub fn decompose_property(
    mut position: usize,
    property_names: Vec<&'_ str>,
    base_type: &Type,
    symbols_table: &SymbolsTable<'_>,
    span: Span,
) -> Result<(Type, Vec<(Type, u32)>), ThrushCompilerIssue> {
    let mut gep_indices: Vec<(Type, u32)> = Vec::with_capacity(10);

    let mut is_parent_mut: bool = false;
    let mut is_parent_ptr: bool = false;

    if position >= property_names.len() {
        return Ok((base_type.clone(), gep_indices));
    }

    let current_type: &Type = match &base_type {
        Type::Mut(inner_type) => {
            is_parent_mut = true;
            inner_type
        }

        Type::Ptr(inner_ptr) => {
            is_parent_ptr = true;

            if let Some(inner_type) = inner_ptr {
                inner_type
            } else {
                return Err(ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Properties of an non-typed pointer 'ptr' cannot be accessed.".into(),
                    None,
                    span,
                ));
            }
        }

        _ => base_type,
    };

    if let Type::Struct(name, _) = current_type {
        let structure: Struct = symbols_table.get_struct(name, span)?;
        let fields: StructFields = structure.get_fields();

        let field_name: &str = property_names[position];

        let field_with_index = fields
            .1
            .iter()
            .enumerate()
            .find(|field| field.1.0 == field_name);

        if let Some((index, (_, field_type, ..))) = field_with_index {
            let mut adjusted_field_type: Type = field_type.clone();

            if is_parent_mut {
                adjusted_field_type = Type::Mut(adjusted_field_type.into());
            } else if is_parent_ptr {
                adjusted_field_type = Type::Ptr(Some(adjusted_field_type.into()));
            }

            gep_indices.push((adjusted_field_type.clone(), index as u32));

            position += 1;

            let (result_type, mut nested_indices) = self::decompose_property(
                position,
                property_names,
                field_type,
                symbols_table,
                span,
            )?;

            for (ty, _) in &mut nested_indices {
                let mut adjusted_ty: Type = ty.clone();

                if is_parent_mut {
                    adjusted_ty = Type::Mut(adjusted_ty.into());
                } else if is_parent_ptr {
                    adjusted_ty = Type::Ptr(Some(adjusted_ty.into()));
                }

                *ty = adjusted_ty;
            }

            gep_indices.append(&mut nested_indices);

            let final_result_type = if is_parent_mut {
                Type::Mut(result_type.into())
            } else if is_parent_ptr {
                Type::Ptr(Some(result_type.into()))
            } else {
                result_type
            };

            return Ok((final_result_type, gep_indices));
        }

        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!("Expected existing property, not '{}'.", field_name,),
            None,
            span,
        ));
    }

    if position < property_names.len() {
        return Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            format!(
                "Existing property '{}' is not a structure.",
                property_names[position]
            ),
            None,
            span,
        ));
    }

    Ok((base_type.clone(), gep_indices))
}
