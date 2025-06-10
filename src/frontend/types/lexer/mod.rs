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
                ThrushTypePointerExtensions, ThrushTypeStructTypeExtensions,
            },
            parser::stmts::{stmt::ThrushStatement, traits::StructExtensions, types::StructFields},
            symbols::types::{Methods, Struct},
        },
    },
};

#[derive(Debug, Clone, Copy)]
pub enum MethodsApplicant {
    Struct,
}

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
            return inner.is_mut_fixed_array_type();
        }

        if let ThrushType::FixedArray(..) = self {
            return true;
        }

        false
    }

    fn is_mut_struct_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            return inner.is_mut_struct_type();
        }

        if let ThrushType::Struct(..) = self {
            return true;
        }

        false
    }

    fn is_mut_numeric_type(&self) -> bool {
        if let ThrushType::Mut(inner) = self {
            return inner.is_mut_numeric_type();
        }

        if self.is_integer_type()
            || self.is_bool_type()
            || self.is_char_type()
            || self.is_float_type()
        {
            return true;
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

impl ThrushTypeStructTypeExtensions for ThrushType {
    fn parser_get_struct_name(&self, span: Span) -> Result<String, ThrushCompilerIssue> {
        if let ThrushType::Struct(name, ..) = self {
            return Ok(name.clone());
        }

        Err(ThrushCompilerIssue::Error(
            String::from("Syntax error"),
            String::from("Expected struct type."),
            None,
            span,
        ))
    }
}

impl ThrushTypePointerExtensions for ThrushType {
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
            return inner.is_ptr_struct_type();
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
    pub fn defer_all(&self) -> ThrushType {
        if let ThrushType::Mut(inner_type) = self {
            return inner_type.defer_all();
        }

        if let ThrushType::Ptr(Some(inner_type)) = self {
            return inner_type.defer_all();
        }

        if let ThrushType::FixedArray(inner_type, ..) = self {
            return inner_type.defer_all();
        }

        self.clone()
    }

    pub fn deref(&self) -> ThrushType {
        if let ThrushType::Ptr(Some(any)) = self {
            return (**any).clone();
        }

        if let ThrushType::Mut(any) = self {
            return (**any).clone();
        }

        self.clone()
    }

    pub fn is_nested_ptr(&self) -> bool {
        if let ThrushType::Ptr(Some(ptr)) = self {
            if let ThrushType::Ptr(..) = &**ptr {
                return true;
            }
        }

        false
    }

    pub fn get_array_type(&self) -> &ThrushType {
        if let ThrushType::FixedArray(array_type, _) = self {
            return array_type.as_ref();
        }

        if let ThrushType::Mut(array_type) = self {
            return array_type.get_array_type();
        }

        logging::log(
            LoggingType::Bug,
            "The array type could not be obtained from an array.",
        );

        unreachable!()
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
    pub const fn is_char_type(&self) -> bool {
        matches!(self, ThrushType::Char)
    }

    #[inline(always)]
    pub const fn is_void_type(&self) -> bool {
        matches!(self, ThrushType::Void)
    }

    #[inline(always)]
    pub const fn is_bool_type(&self) -> bool {
        matches!(self, ThrushType::Bool)
    }

    #[inline(always)]
    pub const fn is_struct_type(&self) -> bool {
        matches!(self, ThrushType::Struct(..))
    }

    #[inline(always)]
    pub const fn is_fixed_array_type(&self) -> bool {
        matches!(self, ThrushType::FixedArray(..))
    }

    #[inline(always)]
    pub const fn is_float_type(&self) -> bool {
        matches!(self, ThrushType::F32 | ThrushType::F64)
    }

    #[inline(always)]
    pub const fn is_ptr_type(&self) -> bool {
        matches!(self, ThrushType::Ptr(_))
    }

    #[inline(always)]
    pub const fn is_address_type(&self) -> bool {
        matches!(self, ThrushType::Addr)
    }

    #[inline(always)]
    pub const fn is_mut_type(&self) -> bool {
        matches!(self, ThrushType::Mut(_))
    }

    #[inline(always)]
    pub const fn is_numeric(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_signed_integer_type(&self) -> bool {
        matches!(
            self,
            ThrushType::S8 | ThrushType::S16 | ThrushType::S32 | ThrushType::S64
        )
    }

    #[inline(always)]
    pub const fn is_integer_type(&self) -> bool {
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

pub fn generate_methods(original: Vec<ThrushStatement>) -> Result<Methods, ThrushCompilerIssue> {
    let mut methods: Methods = Vec::with_capacity(original.len());

    for method in original {
        methods.push((
            method.get_method_name()?,
            method.get_method_type()?,
            method.get_method_parameters_types()?,
        ));
    }

    Ok(methods)
}

pub fn decompose_struct_property(
    mut position: usize,
    property_names: Vec<&'_ str>,
    struct_type: ThrushType,
    symbols_table: &SymbolsTable<'_>,
    span: Span,
) -> Result<(ThrushType, Vec<(ThrushType, u32)>), ThrushCompilerIssue> {
    let mut gep_indices: Vec<(ThrushType, u32)> = Vec::with_capacity(10);

    if position >= property_names.len() {
        return Ok((struct_type.clone(), gep_indices));
    }

    if let ThrushType::Struct(name, _) = &struct_type {
        let structure: Struct = symbols_table.get_struct(name, span)?;
        let fields: StructFields = structure.get_fields();

        let field_name: &str = property_names[position];

        let field_with_index = fields
            .1
            .iter()
            .enumerate()
            .find(|field| field.1.0 == field_name);

        if let Some((index, (_, field_type, ..))) = field_with_index {
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

    Ok((struct_type.clone(), gep_indices))
}
