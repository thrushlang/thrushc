use crate::{
    core::errors::standard::ThrushCompilerIssue,
    frontends::classical::{
        lexer::span::Span,
        parser::symbols::SymbolsTable,
        types::parser::{
            stmts::{traits::StructExtensions, types::StructFields},
            symbols::types::Struct,
        },
        typesystem::types::Type,
    },
};

pub fn decompose(
    mut position: usize,

    property_names: Vec<&str>,
    base_type: &Type,
    symbols_table: &SymbolsTable,

    span: Span,
) -> Result<(Type, Vec<(Type, u32)>), ThrushCompilerIssue> {
    let mut gep_indices: Vec<(Type, u32)> = Vec::with_capacity(10);

    let mut is_parent_ptr: bool = false;

    if position >= property_names.len() {
        return Ok((base_type.clone(), gep_indices));
    }

    let current_type: &Type = match &base_type {
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

    if let Type::Struct(name, _, _) = current_type {
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

            if is_parent_ptr {
                adjusted_field_type = Type::Ptr(Some(adjusted_field_type.into()));
            }

            gep_indices.push((adjusted_field_type.clone(), index as u32));

            position += 1;

            let (result_type, mut nested_indices) =
                self::decompose(position, property_names, field_type, symbols_table, span)?;

            for (ty, _) in &mut nested_indices {
                let mut adjusted_ty: Type = ty.clone();

                if is_parent_ptr {
                    adjusted_ty = Type::Ptr(Some(adjusted_ty.into()));
                }

                *ty = adjusted_ty;
            }

            gep_indices.append(&mut nested_indices);

            let final_result_type = if is_parent_ptr {
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
