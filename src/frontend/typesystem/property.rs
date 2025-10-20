use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontend::{
        lexer::span::Span,
        parser::symbols::SymbolsTable,
        types::{
            ast::Ast,
            parser::{
                stmts::{
                    traits::StructExtensions,
                    types::{StructField, StructFields},
                },
                symbols::types::Struct,
            },
        },
        typesystem::types::Type,
    },
};

pub fn decompose(
    mut position: usize,
    source: &Ast,
    property_names: Vec<&str>,
    base_type: &Type,
    symbols_table: &SymbolsTable,
    span: Span,
) -> Result<(Type, Vec<(Type, u32)>), ThrushCompilerIssue> {
    let mut indices: Vec<(Type, u32)> = Vec::with_capacity(50);
    let mut is_parent_ptr: bool = false;

    if position >= property_names.len() {
        return Ok((base_type.clone(), indices));
    }

    let current_type: &Type = match base_type {
        Type::Ptr(inner_ptr) => {
            is_parent_ptr = true;
            inner_ptr.as_ref().ok_or_else(|| {
                ThrushCompilerIssue::Error(
                    "Type error".into(),
                    "Properties of an non-typed pointer 'ptr' cannot be accessed.".into(),
                    None,
                    span,
                )
            })?
        }
        _ => base_type,
    };

    let field_name: &str = property_names[position];

    if let Type::Struct(name, _, _) = current_type {
        let structure: Struct = symbols_table.get_struct(name, span)?;
        let fields: StructFields = structure.get_fields();

        let field: Option<StructField> = fields
            .1
            .iter()
            .enumerate()
            .find(|(_, (name, ..))| *name == field_name);

        if let Some((index, (_, field_type, ..))) = field {
            let adjusted_field_type: Type = if is_parent_ptr || source.is_allocated() {
                Type::Ptr(Some(field_type.clone().into()))
            } else {
                field_type.clone()
            };

            indices.push((adjusted_field_type.clone(), index as u32));

            position += 1;

            let (field_inner_type, mut nested_indices) = self::decompose(
                position,
                source,
                property_names,
                field_type,
                symbols_table,
                span,
            )?;

            nested_indices.iter_mut().for_each(|(ty, ..)| {
                *ty = if is_parent_ptr || source.is_allocated() {
                    Type::Ptr(Some(ty.clone().into()))
                } else {
                    ty.clone()
                };
            });

            indices.append(&mut nested_indices);

            let adjusted_inner_field_type: Type = if is_parent_ptr || source.is_allocated() {
                Type::Ptr(Some(field_inner_type.into()))
            } else {
                field_inner_type
            };

            return Ok((adjusted_inner_field_type, indices));
        }

        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!("Expected property, not '{}'.", field_name),
            None,
            span,
        ));
    }

    if position < property_names.len() {
        return Err(ThrushCompilerIssue::Error(
            "Syntax error".into(),
            format!("Property source of '{}' isn't a structure.", field_name),
            None,
            span,
        ));
    }

    Ok((base_type.clone(), indices))
}
