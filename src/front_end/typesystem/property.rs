use crate::core::diagnostic::span::Span;
use crate::core::errors::standard::{CompilationIssue, CompilationIssueCode};

use crate::front_end::parser::ParserContext;
use crate::front_end::types::ast::Ast;
use crate::front_end::types::ast::traits::AstMemoryExtensions;
use crate::front_end::types::parser::stmts::traits::{FoundSymbolEither, StructExtensions};
use crate::front_end::types::parser::stmts::types::{StructField, StructFields};
use crate::front_end::types::parser::symbols::types::{FoundSymbolId, Struct};
use crate::front_end::typesystem::traits::TypeCodeLocation;
use crate::front_end::typesystem::types::Type;

pub fn decompose<'parser>(
    ctx: &mut ParserContext<'parser>,
    mut position: usize,
    source: &Ast,
    property_names: Vec<&str>,
    base_type: &Type,
    span: Span,
) -> Result<(Type, Vec<(Type, u32)>), CompilationIssue> {
    let mut indices: Vec<(Type, u32)> = Vec::with_capacity(50);
    let mut is_parent_ptr: bool = false;

    if position >= property_names.len() {
        return Ok((base_type.clone(), indices));
    }

    let default_ptr_type: &Type = &Type::Ptr(None, base_type.get_span());

    let current_type: &Type = match base_type {
        Type::Ptr(inner_ptr, ..) => {
            is_parent_ptr = true;
            inner_ptr.as_ref().map_or(default_ptr_type, |v| v)
        }
        _ => base_type,
    };

    let field_name: &str = property_names[position];

    if let Type::Struct(name, ..) = current_type {
        let object: FoundSymbolId = ctx.get_symbols().get_symbols_id(name, span)?;

        let structure_id: (&str, usize) = object.expected_struct(span)?;
        let id: &str = structure_id.0;
        let scope_idx: usize = structure_id.1;

        let structure: Struct = ctx.get_symbols().get_struct_by_id(id, scope_idx, span)?;
        let fields: StructFields = structure.get_fields();

        let field: Option<StructField> = fields
            .1
            .iter()
            .enumerate()
            .find(|(_, (name, ..))| *name == field_name);

        if let Some((index, (_, field_type, ..))) = field {
            let adjusted_field_type: Type = if is_parent_ptr || source.is_allocated() {
                Type::Ptr(Some(field_type.clone().into()), field_type.get_span())
            } else {
                field_type.clone()
            };

            indices.push((adjusted_field_type.clone(), index as u32));

            position += 1;

            let (field_inner_type, mut nested_indices) =
                self::decompose(ctx, position, source, property_names, field_type, span)?;

            nested_indices.iter_mut().for_each(|(ty, ..)| {
                *ty = if is_parent_ptr || source.is_allocated() {
                    Type::Ptr(Some(ty.clone().into()), ty.get_span())
                } else {
                    ty.clone()
                };
            });

            indices.append(&mut nested_indices);

            let adjusted_inner_field_type: Type = if is_parent_ptr || source.is_allocated() {
                Type::Ptr(
                    Some(field_inner_type.clone().into()),
                    field_inner_type.get_span(),
                )
            } else {
                field_inner_type
            };

            return Ok((adjusted_inner_field_type, indices));
        }

        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            format!("Expected a property, got '{}'.", field_name),
            None,
            span,
        ));
    }

    if position < property_names.len() {
        return Err(CompilationIssue::Error(
            CompilationIssueCode::E0001,
            format!("Property reference of '{}' isn't a structure.", field_name),
            None,
            span,
        ));
    }

    Ok((base_type.clone(), indices))
}
