use std::sync::Arc;

use inkwell::{
    builder::Builder,
    context::Context,
    targets::TargetData,
    values::{BasicValueEnum, PointerValue},
};

use crate::{
    backend::llvm::compiler::{context::LLVMCodeGenContext, typegen},
    frontend::{lexer::span::Span, parser::symbols::SymbolsTable},
    standard::errors::standard::ThrushCompilerIssue,
    types::{
        backend::llvm::traits::LLVMDeallocator,
        frontend::parser::{
            stmts::{stmt::ThrushStatement, traits::StructExtensions, types::StructFields},
            symbols::types::{Methods, Struct},
        },
    },
};

pub type ThrushStructType = (String, Vec<Arc<ThrushType>>);

#[derive(Debug, Clone, Copy)]
pub enum MethodsApplicant {
    Struct,
}

#[derive(Debug, Clone)]
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
    Str,

    // Mutable Type
    Mut(Arc<ThrushType>),

    // Ptr Type
    Ptr(Option<Arc<ThrushType>>),

    // Struct Type
    Struct(String, Vec<Arc<ThrushType>>),

    // Me (Self Type)
    Me(Option<Arc<ThrushType>>),

    // Address
    Address,

    // Void Type
    Void,
}

impl ThrushType {
    #[must_use]
    pub fn precompute_type(&self, other: &ThrushType) -> &ThrushType {
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

            (ThrushType::Mut(a_subtype), ThrushType::Mut(b_subtype)) => {
                a_subtype.precompute_type(b_subtype)
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

    pub fn is_recursive_type(&self) -> bool {
        if let ThrushType::Struct(_, fields) = self {
            return fields.iter().any(|tp| tp.is_recursive_type());
        }

        matches!(self, ThrushType::Me(_))
    }

    pub fn has_any_recursive_type(&self) -> bool {
        let mut indexes: Vec<bool> = Vec::with_capacity(50);

        self.check_recursive_type(&mut indexes);

        indexes.iter().any(|found| *found)
    }

    fn check_recursive_type(&self, indexes: &mut Vec<bool>) {
        if let ThrushType::Struct(_, fields) = self {
            for field in fields.iter() {
                if field.is_recursive_type() {
                    indexes.push(true);
                    field.check_recursive_type(indexes);
                }
            }
        }
    }

    pub fn get_recursive_types(&self) -> Vec<ThrushType> {
        let mut types: Vec<ThrushType> = Vec::with_capacity(50);
        let mut current: ThrushType = ThrushType::Void;

        self.get_exact_recursive_type(&mut types, &mut current);

        types
    }

    fn get_exact_recursive_type(&self, types: &mut Vec<ThrushType>, current: &mut ThrushType) {
        match self {
            ThrushType::Struct(_, fields) => {
                for field in fields.iter() {
                    *current = (**field).clone();

                    field.get_exact_recursive_type(types, current);

                    *current = ThrushType::Void;
                }
            }

            ThrushType::Me(_) => {
                types.push(current.clone());
            }

            _ => (),
        }
    }

    pub fn get_recursive_type_paths(&self) -> Vec<(ThrushType, Vec<u32>)> {
        let mut paths: Vec<(ThrushType, Vec<u32>)> = Vec::with_capacity(50);
        let mut current_path: Vec<u32> = Vec::with_capacity(50);
        let mut current_struct_type: ThrushType = self.clone();

        self.get_recursive_type_path(&mut paths, &mut current_struct_type, &mut current_path);

        paths
    }

    fn get_recursive_type_path(
        &self,
        paths: &mut Vec<(ThrushType, Vec<u32>)>,
        current_struct_type: &mut ThrushType,
        current_path: &mut Vec<u32>,
    ) {
        match self {
            ThrushType::Struct(_, fields) => {
                *current_struct_type = self.clone();

                for (index, field) in fields.iter().enumerate() {
                    current_path.push(index as u32);
                    field.get_recursive_type_path(paths, current_struct_type, current_path);
                    current_path.pop();
                }
            }

            ThrushType::Me(_) => {
                current_path.insert(0, 0);
                paths.push((current_struct_type.clone(), current_path.clone()));
            }

            _ => (),
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
            || self.is_recursive_type()
    }

    pub fn is_mut_ptr_type(&self) -> bool {
        if let ThrushType::Mut(subtype) = self {
            if let ThrushType::Ptr(_) = &**subtype {
                return true;
            }
        }

        false
    }

    pub fn is_mut_numeric_type(&self) -> bool {
        if let ThrushType::Mut(subtype) = self {
            return subtype.is_integer_type() || subtype.is_float_type();
        }

        false
    }

    pub fn into_structure_type(self) -> ThrushStructType {
        if let ThrushType::Struct(name, types) = self {
            return (name, types);
        }

        unreachable!()
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
    pub const fn is_float_type(&self) -> bool {
        matches!(self, ThrushType::F32 | ThrushType::F64)
    }

    #[inline(always)]
    pub const fn is_ptr_type(&self) -> bool {
        matches!(self, ThrushType::Ptr(_))
    }

    #[inline(always)]
    pub const fn is_address_type(&self) -> bool {
        matches!(self, ThrushType::Address)
    }

    #[inline(always)]
    pub const fn is_mut_type(&self) -> bool {
        matches!(self, ThrushType::Mut(_))
    }

    #[inline(always)]
    pub const fn is_str_type(&self) -> bool {
        matches!(self, ThrushType::Str)
    }

    #[inline(always)]
    pub const fn is_numeric(&self) -> bool {
        self.is_integer_type() || self.is_float_type() || self.is_char_type() || self.is_bool_type()
    }

    #[inline(always)]
    pub const fn is_me_type(&self) -> bool {
        matches!(self, ThrushType::Me(_))
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
}

impl LLVMDeallocator for ThrushType {
    fn dealloc(&self, context: &LLVMCodeGenContext<'_, '_>, value: BasicValueEnum<'_>) {
        let llvm_builder: &Builder = context.get_llvm_builder();
        let llvm_context: &Context = context.get_llvm_context();
        let target_data: &TargetData = context.get_target_data();

        if self.is_probably_heap_allocated(llvm_context, target_data) && value.is_pointer_value() {
            let ptr: PointerValue = value.into_pointer_value();

            if self.has_any_recursive_type() {
                if let Some(last_block) = llvm_builder.get_insert_block() {
                    /*let recursive_paths: Vec<Vec<u32>> = self.get_recursive_type_paths();
                    let recursive_types: Vec<ThrushType> = self.get_recursive_types();

                    println!("{:?}", self.get_recursive_types());

                    let deallocator: FunctionValue =
                        memory::create_deallocator(context, self, &recursive_paths);

                    llvm_builder.position_at_end(last_block);

                    let _ = llvm_builder.build_call(deallocator, &[ptr.into()], "");

                    llvm_builder.position_at_end(last_block);*/

                    return;
                }
            }

            let _ = llvm_builder.build_free(ptr);
        }
    }
}

impl PartialEq for ThrushType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ThrushType::Struct(_, fields1), ThrushType::Struct(_, fields2)) => {
                fields1.len() == fields2.len()
                    && fields1
                        .iter()
                        .zip(fields2.iter())
                        .all(|(f1, f2)| f1.as_ref() == f2.as_ref())
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
            (ThrushType::Str, ThrushType::Str) => true,
            (ThrushType::Me(Some(target)), ThrushType::Me(Some(from))) => target == from,
            (ThrushType::Me(None), ThrushType::Me(None)) => true,
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

        let field_with_index: Option<(usize, &(&str, ThrushType, u32, Span))> = fields
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
