use thrushc_ast::{
    Ast,
    data::{StructDataField, StructureData},
    metadata::PropertyMetadata,
    traits::{AstGetType, AstMemoryExtensions},
};
use thrushc_entities::parser::{FoundSymbolId, Struct};
use thrushc_errors::{CompilationIssue, CompilationIssueCode};
use thrushc_span::Span;
use thrushc_token::{Token, traits::TokenExtensions};
use thrushc_token_type::TokenType;
use thrushc_typesystem::{Type, traits::TypeCodeLocation};

use crate::{
    ParserContext,
    traits::{FoundSymbolEitherExtensions, StructSymbolExtensions},
};

pub fn build_property<'parser>(
    ctx: &mut ParserContext<'parser>,
    source: Ast<'parser>,
) -> Result<Ast<'parser>, CompilationIssue> {
    let source_type: &Type = source.get_value_type()?;
    let metadata: PropertyMetadata = PropertyMetadata::new(source.is_allocated());

    let mut property_names: Vec<&str> = Vec::with_capacity(10);

    let first_property: &Token = ctx.consume(
        TokenType::Identifier,
        CompilationIssueCode::E0001,
        "Expected property name.".into(),
    )?;

    let mut span: Span = first_property.span;

    property_names.push(first_property.get_lexeme());

    while ctx.match_token(TokenType::Dot)? {
        let property: &Token = ctx.consume(
            TokenType::Identifier,
            CompilationIssueCode::E0001,
            "Expected property name.".into(),
        )?;

        span = property.span;

        property_names.push(property.get_lexeme());
    }

    property_names.reverse();

    let decomposed_property: (Type, Vec<(Type, u32)>) =
        self::decompose(ctx, 0, &source, property_names, source_type, span)?;

    let property_type: Type = decomposed_property.0;
    let indexes: Vec<(Type, u32)> = decomposed_property.1;

    Ok(Ast::Property {
        source: source.into(),
        indexes,
        kind: property_type,
        metadata,
        span,
    })
}

fn decompose<'parser>(
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
        let fields: StructureData = structure.get_fields();

        let field: Option<StructDataField> = fields
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
