pub mod impls;
pub mod traits;

use std::sync::Arc;

use inkwell::{context::Context, targets::TargetData, types::BasicTypeEnum};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    core::{
        console::logging::{self, LoggingType},
        errors::standard::ThrushCompilerIssue,
    },
    frontend::{
        lexer::span::Span,
        parser::symbols::SymbolsTable,
        types::{
            lexer::traits::{
                LLVMTypeExtensions, ThrushTypeMutableExtensions, ThrushTypeNumericExtensions,
                ThrushTypePointerExtensions,
            },
            parser::stmts::{traits::StructExtensions, types::StructFields},
            symbols::types::Struct,
        },
    },
};

#[derive(Debug, Clone, Default)]
pub enum ThrushType {
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
    Str(Arc<ThrushType>),

    // Mutable Type
    Mut(Arc<ThrushType>),

    // Ptr Type
    Ptr(Option<Arc<ThrushType>>),

    // Struct Type
    Struct(String, Vec<Arc<ThrushType>>),

    // Fixed FixedArray
    FixedArray(Arc<ThrushType>, u32),

    // Address
    Addr,

    // Void Type
    #[default]
    Void,
}

impl ThrushTypeMutableExtensions for ThrushType {
    fn is_mut_fixed_array_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            if let ThrushType::FixedArray(..) = &**inner {
                return true;
            }
        }

        false
    }

    fn is_mut_struct_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            if let ThrushType::Struct(..) = &**inner {
                return true;
            }
        }

        false
    }

    fn is_mut_numeric_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
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

    fn is_mut_any_nonumeric_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            if inner.is_struct_type() || inner.is_fixed_array_type() {
                return true;
            }
        }

        false
    }

    fn defer_mut_all(&self) -> ThrushType {
        if let ThrushType::Mut(inner_type) = self {
            return inner_type.defer_mut_all();
        }

        self.clone()
    }
}

impl ThrushTypeNumericExtensions for ThrushType {
    fn is_numeric_type(&self) -> bool {
        if self.is_integer_type()
            || self.is_bool_type()
            || self.is_char_type()
            || self.is_float_type()
        {
            return true;
        }

        false
    }
}

impl ThrushTypePointerExtensions for ThrushType {
    fn is_all_ptr(&self) -> bool {
        if let ThrushType::Ptr(Some(ptr)) = self {
            return ptr.is_all_ptr();
        }

        if let ThrushType::Ptr(None) = self {
            return true;
        }

        false
    }

    fn is_typed_ptr(&self) -> bool {
        if let ThrushType::Ptr(Some(inner)) = self {
            return inner.is_typed_ptr();
        }

        if let ThrushType::Ptr(None) = self {
            return false;
        }

        true
    }

    fn is_ptr_struct_type(&self) -> bool {
        if let ThrushType::Ptr(Some(inner)) = self {
            return inner.is_ptr_struct_type_inner();
        }

        false
    }

    fn is_ptr_struct_type_inner(&self) -> bool {
        if let ThrushType::Ptr(Some(inner)) = self {
            return inner.is_ptr_struct_type_inner();
        }

        if let ThrushType::Ptr(None) = self {
            return false;
        }

        if let ThrushType::Struct(..) = self {
            return true;
        }

        false
    }

    fn is_ptr_fixed_array_type(&self) -> bool {
        if let ThrushType::Ptr(Some(inner)) = self {
            return inner.is_ptr_fixed_array_type_inner();
        }

        false
    }

    fn is_ptr_fixed_array_type_inner(&self) -> bool {
        if let ThrushType::Ptr(Some(inner)) = self {
            return inner.is_ptr_fixed_array_type();
        }

        if let ThrushType::Ptr(None) = self {
            return false;
        }

        if let ThrushType::FixedArray(..) = self {
            return true;
        }

        false
    }
}

impl LLVMTypeExtensions for ThrushType {
    fn is_same_size(&self, context: &LLVMCodeGenContext<'_, '_>, other: &ThrushType) -> bool {
        let llvm_context: &Context = context.get_llvm_context();

        let a_llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, self);
        let b_llvm_type: BasicTypeEnum = typegen::generate_subtype(llvm_context, other);

        let target_data: &TargetData = context.get_target_data();

        target_data.get_bit_size(&a_llvm_type) == target_data.get_bit_size(&b_llvm_type)
    }
}

impl ThrushType {
    pub fn deref(&self) -> ThrushType {
        if let ThrushType::Ptr(Some(any)) = self {
            return (**any).clone();
        }

        if let ThrushType::Mut(any) = self {
            return (**any).clone();
        }

        self.clone()
    }

    pub fn get_array_base_type(&self) -> &ThrushType {
        if let ThrushType::FixedArray(inner, _) = self {
            return inner.get_array_base_type();
        }

        if let ThrushType::Mut(inner) = self {
            return inner.get_array_base_type();
        }

        if let ThrushType::Ptr(Some(inner)) = self {
            return inner.get_array_base_type();
        }

        self
    }

    pub fn get_type_with_depth(&self, depth: usize) -> &ThrushType {
        if depth == 0 {
            return self;
        }

        match self {
            ThrushType::Mut(inner_type) => inner_type.get_type_with_depth(depth - 1),
            ThrushType::Ptr(Some(inner_type)) => inner_type.get_type_with_depth(depth - 1),
            ThrushType::FixedArray(element_type, _) => element_type.get_type_with_depth(depth - 1),
            ThrushType::Struct(_, _) => self,
            ThrushType::S8
            | ThrushType::S16
            | ThrushType::S32
            | ThrushType::S64
            | ThrushType::U8
            | ThrushType::U16
            | ThrushType::U32
            | ThrushType::U64
            | ThrushType::F32
            | ThrushType::F64
            | ThrushType::Bool
            | ThrushType::Char
            | ThrushType::Str(..)
            | ThrushType::Addr
            | ThrushType::Void
            | ThrushType::Ptr(None) => self,
        }
    }

    #[must_use]
    pub fn precompute_numeric_type(&self, other: &ThrushType) -> &ThrushType {
        match (self, other) {
            (ThrushType::S64, _) | (_, ThrushType::S64) => &ThrushType::S64,
            (ThrushType::S32, _) | (_, ThrushType::S32) => &ThrushType::S32,
            (ThrushType::S16, _) | (_, ThrushType::S16) => &ThrushType::S16,
            (ThrushType::S8, _) | (_, ThrushType::S8) => &ThrushType::S8,

            (ThrushType::U64, _) | (_, ThrushType::U64) => &ThrushType::U64,
            (ThrushType::U32, _) | (_, ThrushType::U32) => &ThrushType::U32,
            (ThrushType::U16, _) | (_, ThrushType::U16) => &ThrushType::U16,
            (ThrushType::U8, _) | (_, ThrushType::U8) => &ThrushType::U8,

            (ThrushType::F64, _) | (_, ThrushType::F64) => &ThrushType::F64,
            (ThrushType::F32, _) | (_, ThrushType::F32) => &ThrushType::F32,

            (ThrushType::Mut(a_inner), ThrushType::Mut(b_inner)) => {
                a_inner.precompute_numeric_type(b_inner)
            }

            _ => self,
        }
    }

    pub fn narrowing_cast(&self) -> ThrushType {
        match self {
            ThrushType::U8 => ThrushType::S8,
            ThrushType::U16 => ThrushType::S16,
            ThrushType::U32 => ThrushType::S32,
            ThrushType::U64 => ThrushType::S64,
            _ => self.clone(),
        }
    }

    pub fn create_structure_type(name: String, fields: &[ThrushType]) -> ThrushType {
        ThrushType::Struct(
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
        matches!(self, ThrushType::Char)
    }

    #[inline(always)]
    pub fn is_void_type(&self) -> bool {
        matches!(self, ThrushType::Void)
    }

    #[inline(always)]
    pub fn is_bool_type(&self) -> bool {
        matches!(self, ThrushType::Bool)
    }

    #[inline(always)]
    pub fn is_struct_type(&self) -> bool {
        matches!(self, ThrushType::Struct(..))
    }

    #[inline(always)]
    pub fn is_fixed_array_type(&self) -> bool {
        matches!(self, ThrushType::FixedArray(..))
    }

    #[inline(always)]
    pub fn is_float_type(&self) -> bool {
        matches!(self, ThrushType::F32 | ThrushType::F64)
    }

    #[inline(always)]
    pub fn is_ptr_type(&self) -> bool {
        matches!(self, ThrushType::Ptr(_))
    }

    #[inline(always)]
    pub fn is_address_type(&self) -> bool {
        matches!(self, ThrushType::Addr)
    }

    #[inline(always)]
    pub fn is_str_type(&self) -> bool {
        matches!(self, ThrushType::Str(..))
    }

    #[inline(always)]
    pub fn is_mut_type(&self) -> bool {
        matches!(self, ThrushType::Mut(_))
    }

    #[inline(always)]
    pub fn is_numeric(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[must_use]
    #[inline(always)]
    pub fn is_signed_integer_type(&self) -> bool {
        matches!(
            self,
            ThrushType::S8 | ThrushType::S16 | ThrushType::S32 | ThrushType::S64
        )
    }

    #[inline(always)]
    pub fn is_integer_type(&self) -> bool {
        matches!(
            self,
            ThrushType::S8
                | ThrushType::S16
                | ThrushType::S32
                | ThrushType::S64
                | ThrushType::U8
                | ThrushType::U16
                | ThrushType::U32
                | ThrushType::U64
                | ThrushType::Char
        )
    }

    #[inline(always)]
    pub fn is_mut_integer_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            return inner.is_integer_type();
        }

        false
    }

    #[inline(always)]
    pub fn is_mut_float_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            return inner.is_float_type();
        }

        false
    }

    #[inline]
    pub fn get_fixed_array_type_herarchy(&self) -> u8 {
        match self {
            ThrushType::Void => 0,

            ThrushType::Bool => 1,
            ThrushType::Char => 2,
            ThrushType::Str(..) => 3,

            ThrushType::S8 => 4,
            ThrushType::S16 => 5,
            ThrushType::S32 => 6,
            ThrushType::S64 => 7,

            ThrushType::U8 => 8,
            ThrushType::U16 => 9,
            ThrushType::U32 => 10,
            ThrushType::U64 => 11,

            ThrushType::F32 => 12,
            ThrushType::F64 => 13,

            ThrushType::Mut(subtype) => subtype.get_fixed_array_type_herarchy(),

            ThrushType::Addr => 14,
            ThrushType::Ptr(Some(subtype)) => subtype.get_fixed_array_type_herarchy(),
            ThrushType::Ptr(None) => 15,

            ThrushType::FixedArray(..) => 16,
            ThrushType::Struct(..) => 17,
        }
    }
}

impl PartialEq for ThrushType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ThrushType::Struct(a, fields1), ThrushType::Struct(b, fields2)) => {
                fields1.len() == fields2.len()
                    && a == b
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .all(|(f1, f2)| f1.as_ref() == f2.as_ref())
            }

            (ThrushType::FixedArray(type_a, size_a), ThrushType::FixedArray(type_b, size_b)) => {
                type_a == type_b && size_a == size_b
            }

            (ThrushType::Mut(target), ThrushType::Mut(from)) => target == from,
            (ThrushType::Char, ThrushType::Char) => true,
            (ThrushType::S8, ThrushType::S8) => true,
            (ThrushType::S16, ThrushType::S16) => true,
            (ThrushType::S32, ThrushType::S32) => true,
            (ThrushType::S64, ThrushType::S64) => true,
            (ThrushType::U8, ThrushType::U8) => true,
            (ThrushType::U16, ThrushType::U16) => true,
            (ThrushType::U32, ThrushType::U32) => true,
            (ThrushType::U64, ThrushType::U64) => true,
            (ThrushType::F32, ThrushType::F32) => true,
            (ThrushType::F64, ThrushType::F64) => true,
            (ThrushType::Ptr(None), ThrushType::Ptr(None)) => true,
            (ThrushType::Ptr(Some(target)), ThrushType::Ptr(Some(from))) => target == from,
            (ThrushType::Void, ThrushType::Void) => true,
            (ThrushType::Str(..), ThrushType::Str(..)) => true,
            (ThrushType::Bool, ThrushType::Bool) => true,

            _ => false,
        }
    }
}

pub fn decompose_struct_property(
    mut position: usize,
    property_names: Vec<&'_ str>,
    base_type: ThrushType,
    symbols_table: &SymbolsTable<'_>,
    span: Span,
) -> Result<(ThrushType, Vec<(ThrushType, u32)>), ThrushCompilerIssue> {
    let mut gep_indices: Vec<(ThrushType, u32)> = Vec::with_capacity(10);

    let mut is_parent_mut: bool = false;
    let mut is_parent_ptr: bool = false;

    if position >= property_names.len() {
        return Ok((base_type.clone(), gep_indices));
    }

    let current_type: &ThrushType = match &base_type {
        ThrushType::Mut(inner_type) => {
            is_parent_mut = true;
            inner_type
        }

        ThrushType::Ptr(inner_ptr) => {
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

    if let ThrushType::Struct(name, _) = current_type {
        let structure: Struct = symbols_table.get_struct(name, span)?;
        let fields: StructFields = structure.get_fields();

        let field_name: &str = property_names[position];

        let field_with_index = fields
            .1
            .iter()
            .enumerate()
            .find(|field| field.1.0 == field_name);

        if let Some((index, (_, field_type, ..))) = field_with_index {
            let mut adjusted_field_type: ThrushType = field_type.clone();

            if is_parent_mut {
                adjusted_field_type = ThrushType::Mut(adjusted_field_type.into());
            }
            if is_parent_ptr {
                adjusted_field_type = ThrushType::Ptr(Some(adjusted_field_type.into()));
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
                let mut adjusted_ty: ThrushType = ty.clone();

                if is_parent_mut {
                    adjusted_ty = ThrushType::Mut(adjusted_ty.into());
                }

                if is_parent_ptr {
                    adjusted_ty = ThrushType::Ptr(Some(adjusted_ty.into()));
                }

                *ty = adjusted_ty;
            }

            gep_indices.append(&mut nested_indices);

            let final_result_type = if is_parent_mut {
                ThrushType::Mut(result_type.into())
            } else if is_parent_ptr {
                ThrushType::Ptr(Some(result_type.into()))
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
