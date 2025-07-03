pub mod impls;
pub mod traits;

use std::sync::Arc;

use inkwell::{context::Context, targets::TargetData, types::BasicTypeEnum};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::span::Span,
        parser::symbols::SymbolsTable,
        types::{
            lexer::traits::{
                LLVMTypeExtensions, TypeMutableExtensions, TypeNumericExtensions,
                TypePointerExtensions, TypeStructExtensions,
            },
            parser::{
                stmts::{traits::StructExtensions, types::StructFields},
                symbols::types::Struct,
            },
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

impl TypeStructExtensions for Type {
    fn get_struct_fields(&self) -> &[Arc<Type>] {
        if let Type::Struct(_, fields) = self {
            return fields;
        }

        &[]
    }
}

impl TypeMutableExtensions for Type {
    fn is_mut_fixed_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_fixed_array_type();
        }

        false
    }

    fn is_mut_array_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_array_type();
        }

        false
    }

    fn is_mut_struct_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            return inner.is_struct_type();
        }

        false
    }

    fn is_mut_numeric_type(&self) -> bool {
        if let Type::Mut(inner) = self {
            if inner.is_integer_type()
                || inner.is_bool_type()
                || inner.is_char_type()
                || inner.is_float_type()
            {
                return true;
            }
        }

        false
    }

    fn defer_mut_all(&self) -> Type {
        if let Type::Mut(inner_type) = self {
            return inner_type.defer_mut_all();
        }

        self.clone()
    }
}

impl TypeNumericExtensions for Type {
    fn is_numeric_type(&self) -> bool {
        if self.is_integer_type()
            || self.is_float_type()
            || self.is_bool_type()
            || self.is_char_type()
        {
            return true;
        }

        false
    }
}

impl TypePointerExtensions for Type {
    fn is_all_ptr(&self) -> bool {
        if let Type::Ptr(Some(ptr)) = self {
            return ptr.is_all_ptr();
        }

        if let Type::Ptr(None) = self {
            return true;
        }

        false
    }

    fn is_typed_ptr(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_typed_ptr();
        }

        if let Type::Ptr(None) = self {
            return false;
        }

        true
    }

    fn is_ptr_struct_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_struct_type();
        }

        false
    }

    fn is_ptr_fixed_array_type(&self) -> bool {
        if let Type::Ptr(Some(inner)) = self {
            return inner.is_fixed_array_type();
        }

        false
    }
}

impl LLVMTypeExtensions for Type {
    fn is_same_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &Type) -> bool {
        let llvm_context: &Context = context.get_llvm_context();

        let a_llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, self);
        let b_llvm_type: BasicTypeEnum = typegen::generate_type(llvm_context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&a_llvm_type) == target_data.get_bit_size(&b_llvm_type)
    }
}

impl Type {
    pub fn deref(&self) -> Type {
        if let Type::Ptr(Some(any)) = self {
            return (**any).clone();
        }

        if let Type::Mut(any) = self {
            return (**any).clone();
        }

        self.clone()
    }

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

        self
    }

    pub fn get_type_with_depth(&self, depth: usize) -> &Type {
        if depth == 0 {
            return self;
        }

        match self {
            Type::FixedArray(element_type, _) => element_type.get_type_with_depth(depth),
            Type::Array(element_type) => element_type.get_type_with_depth(depth),
            Type::Mut(inner_type) => inner_type.get_type_with_depth(depth),
            Type::Ptr(Some(inner_type)) => inner_type.get_type_with_depth(depth - 1),
            Type::Struct(_, _) => self,
            Type::S8
            | Type::S16
            | Type::S32
            | Type::S64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Char
            | Type::Str
            | Type::Addr
            | Type::Void
            | Type::Ptr(None) => self,
        }
    }

    #[must_use]
    pub fn precompute_numeric_type(&self, other: &Type) -> &Type {
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

            (Type::Mut(a), Type::Mut(b)) => a.precompute_numeric_type(b),

            _ => self,
        }
    }

    pub fn narrowing_cast(&self) -> Type {
        match self {
            Type::U8 => Type::S8,
            Type::U16 => Type::S16,
            Type::U32 => Type::S32,
            Type::U64 => Type::S64,

            Type::S8 => Type::U8,
            Type::S16 => Type::U16,
            Type::S32 => Type::U32,
            Type::S64 => Type::U64,

            _ => self.clone(),
        }
    }

    pub fn create_structure_type(name: String, fields: &[Type]) -> Type {
        Type::Struct(
            name,
            fields.iter().map(|field| Arc::new(field.clone())).collect(),
        )
    }

    pub fn is_probably_heap_allocated(
        &self,
        llvm_context: &Context,
        target_data: &TargetData,
    ) -> bool {
        target_data.get_abi_size(&typegen::generate_type(llvm_context, self)) >= 128
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
    pub fn is_numeric(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[must_use]
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

pub fn decompose_struct_property(
    mut position: usize,
    property_names: Vec<&'_ str>,
    base_type: Type,
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

        _ => &base_type,
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

            let (result_type, mut nested_indices) = decompose_struct_property(
                position,
                property_names,
                field_type.clone(),
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
